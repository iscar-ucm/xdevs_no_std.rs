/// A simple GPB (Generator-Processor-Buffer) model using xDEVS
/// This example shows how to apply Rust's resources for structs
/// (like generics and lifetimes) in DEVS models
mod generator {
    #[xdevs::atomic]
    pub struct Generator {
        #[output]
        pub out_job: xdevs::port::Port<usize, 1>,
        #[state]
        sigma: f64,
        period: f64,
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

        fn ta(state: &Self::State) -> f64 {
            state.sigma
        }

        fn delta_ext(state: &mut Self::State, elapsed: f64, _input: &Self::Input) {
            state.sigma -= elapsed;
        }
    }

    impl Generator {
        pub fn new(period: f64) -> Self {
            Self::build(0.0, period, 0)
        }
    }
}

mod processor {
    #[xdevs::atomic]
    pub struct Processor {
        #[input]
        pub in_job: xdevs::port::Port<usize, 1>,
        #[output]
        pub out_job: xdevs::port::Port<usize, 1>,
        #[state]
        sigma: f64,
        time: f64,
        job: Option<usize>,
    }

    impl xdevs::Atomic for Processor {
        fn delta_int(state: &mut Self::State) {
            state.sigma = f64::INFINITY;
            if let Some(job) = state.job.take() {
                println!("[P] processed job {}", job);
            }
        }

        fn lambda(state: &Self::State, output: &mut Self::Output) {
            if let Some(job) = state.job {
                output.out_job.add_value(job).unwrap();
            }
        }

        fn ta(state: &Self::State) -> f64 {
            state.sigma
        }

        fn delta_ext(state: &mut Self::State, elapsed: f64, input: &Self::Input) {
            state.sigma -= elapsed;
            if let Some(&job) = input.in_job.get_values().last() {
                if state.job.is_none() {
                    state.job = Some(job);
                    state.sigma = state.time;
                }
            }
        }
    }

    impl Processor {
        pub fn new(time: f64) -> Self {
            Self::build(0.0, time, None)
        }
    }
}

mod buffer {
    use core::fmt::Debug;
    use xdevs::port::Port;

    #[xdevs::atomic]
    pub struct Buffer<'a, T: Clone + Debug> {
        #[input]
        pub in_item: Port<T, 1>,
        #[output]
        pub out_item: Port<T, 1>,
        #[state]
        sigma: f64,
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
                f64::INFINITY
            } else {
                1.0
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

        fn ta(state: &Self::State) -> f64 {
            state.sigma
        }

        fn delta_ext(state: &mut Self::State, elapsed: f64, input: &Self::Input) {
            state.sigma -= elapsed;
            for item in input.in_item.get_values() {
                if state.queue.len() < state.capacity {
                    println!("[B] received item {:?}", item);
                    state.queue.push(item.clone()).unwrap();
                } else {
                    println!("[B] buffer full, dropping {:?}", item);
                }
            }
            if !state.queue.is_empty() {
                state.sigma = 1.0;
            }
        }
    }

    impl<'a, T: Clone + Debug> Buffer<'a, T> {
        pub fn new(capacity: usize, config: Option<&'a str>) -> Self {
            Self::build(f64::INFINITY, capacity, heapless::Vec::new(), config)
        }
    }
}

#[xdevs::coupled]
struct GPB<'a> {
    #[components]
    generator: generator::Generator,
    buffer: buffer::Buffer<'a, usize>,
    processor: processor::Processor,
}

impl xdevs::Coupled for GPB<'_> {
    fn ic(from: &Self::ComponentsOutput<'_>, to: &mut Self::ComponentsInput<'_>) {
        from.generator
            .out_job
            .couple(&mut to.buffer.in_item)
            .unwrap();
        from.buffer
            .out_item
            .couple(&mut to.processor.in_job)
            .unwrap();
    }
}

fn main() {
    let generator = generator::Generator::new(1.0);
    let buffer = buffer::Buffer::new(8, Some("FIFO buffer"));
    let processor = processor::Processor::new(1.5);
    let gpb = GPB::build(generator, buffer, processor);

    let mut simulator = xdevs::simulator::Simulator::new(gpb);
    let config = xdevs::simulator::Config::new(0.0, 10.0, 1.0, None);
    simulator.simulate_rt(&config, xdevs::simulator::std::sleep(&config), |_| {});
}
