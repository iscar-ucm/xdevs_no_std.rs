use embassy_time::{Duration as eDuration, Instant as eInstant};

mod generator {
    use crate::eDuration;
    #[xdevs::atomic]
    pub struct Generator {
        #[output]
        pub out_job: xdevs::port::Port<usize, 1>,
        #[state]
        sigma: eDuration,
        period: eDuration,
        count: usize,
    }

    impl xdevs::Atomic for Generator {
        fn delta_int(state: &mut Self::State) {
            state.count += 1;
            state.sigma = state.period;
        }

        fn lambda(state: &Self::State, output: &mut Self::Output) {
            output.out_job.add_value(state.count).unwrap();
        }

        fn ta(state: &Self::State) -> eDuration {
            state.sigma
        }

        fn delta_ext(state: &mut Self::State, e: eDuration, _x: &Self::Input) {
            state.sigma -= e;
        }
    }

    impl Generator {
        pub fn new(period: eDuration) -> Self {
            //cambio new2 por new
            Self::build(eDuration::from_millis(0), period, 0) //cambio new por build
        }
    }
}

mod processor {
    use crate::eDuration;
    #[xdevs::atomic]
    pub struct Processor {
        #[input]
        pub in_job: xdevs::port::Port<usize, 1>,
        #[output]
        pub out_job: xdevs::port::Port<usize, 1>,
        #[state]
        sigma: eDuration,
        time: eDuration,
        job: Option<usize>,
    }

    impl xdevs::Atomic for Processor {
        fn delta_int(state: &mut Self::State) {
            state.sigma = eDuration::MAX;
            if let Some(job) = state.job.take() {
                println!("[P] processed job {}", job);
            }
        }

        fn lambda(state: &Self::State, output: &mut Self::Output) {
            if let Some(job) = state.job {
                output.out_job.add_value(job).unwrap();
            }
        }

        fn ta(state: &Self::State) -> eDuration {
            state.sigma
        }

        fn delta_ext(state: &mut Self::State, e: eDuration, x: &Self::Input) {
            state.sigma -= e;
            if let Some(&job) = x.in_job.get_values().last() {
                if state.job.is_none() {
                    state.job = Some(job);
                    state.sigma = state.time;
                }
            }
        }
    }

    impl Processor {
        pub fn new(time: eDuration) -> Self {
            Self::build(eDuration::from_millis(0), time, None)
        }
    }
}

mod buffer {
    use crate::eDuration;
    use core::fmt::Debug;
    use xdevs::port::Port;

    #[xdevs::atomic]
    pub struct Buffer<'a, T: Clone + Debug> {
        #[input]
        pub in_item: Port<T, 1>,
        #[output]
        pub out_item: Port<T, 1>,
        #[state]
        sigma: eDuration,
        capacity: usize,
        queue: heapless::Vec<T, 16>,
        config: Option<&'a str>,
    }

    impl<'a, T: Clone + Debug> xdevs::Atomic for Buffer<'a, T> {
        fn delta_int(state: &mut Self::State) {
            if !state.queue.is_empty() {
                state.queue.remove(0);
            }
            state.sigma = if state.queue.is_empty() {
                eDuration::MAX
            } else {
                eDuration::from_millis(1)
            };
        }

        fn lambda(state: &Self::State, output: &mut Self::Output) {
            if let Some(item) = state.queue.first() {
                if let Some(cfg) = state.config {
                    println!("[B:{}] sending item {:?}", cfg, item);
                } else {
                    println!("[B] sending item {:?}", item);
                }
                output.out_item.add_value(item.clone()).unwrap();
            }
        }

        fn ta(state: &Self::State) -> eDuration {
            state.sigma
        }

        fn delta_ext(state: &mut Self::State, e: eDuration, x: &Self::Input) {
            state.sigma -= e;
            for item in x.in_item.get_values() {
                if state.queue.len() < state.capacity {
                    println!("[B] received item {:?}", item);
                    state.queue.push(item.clone()).unwrap();
                } else {
                    println!("[B] buffer full, dropping {:?}", item);
                }
            }
            if !state.queue.is_empty() {
                state.sigma = eDuration::from_millis(1);
            }
        }
    }

    impl<'a, T: Clone + Debug> Buffer<'a, T> {
        pub fn new(capacity: usize, config: Option<&'a str>) -> Self {
            //cambiio new2 por new
            Self::build(eDuration::MAX, capacity, heapless::Vec::new(), config)
            //cambio new por build
        }
    }
}

#[xdevs::coupled(
    couplings = {
        generator.out_job -> buffer.in_item,
        buffer.out_item -> processor.in_job,
    }
)]
struct GPB<'a> {
    #[components]
    generator: generator::Generator,
    buffer: buffer::Buffer<'a, usize>,
    processor: processor::Processor,
}

fn main() {
    let generator = generator::Generator::new(eDuration::from_millis(1000)); //cambio new2 por new
    let buffer = buffer::Buffer::new(8, Some("FIFO buffer")); //cambio new2 por new
    let processor = processor::Processor::new(eDuration::from_millis(1500)); //previamente 1.5 //cambio new2 por new

    let gpb = GPB::new(generator, buffer, processor);

    let mut simulator = xdevs::simulator::Simulator::new(gpb);
    let config = xdevs::simulator::Config::new(
        eInstant::from_millis(0),
        eInstant::from_millis(10000),
        1,
        None,
    );
    //simulator.simulate_rt(&config, xdevs::simulator::std::sleep(&config), |_| {});

    simulator.simulate_vt(&config);
}
