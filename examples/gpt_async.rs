/// A simple DEVS GPT model with async simulation
use xdevs::simulator::{std::SleepAsync, Config, Simulator};

mod generator {
    #[xdevs::atomic]
    pub struct Generator {
        #[input]
        pub in_stop: xdevs::port::Port<bool, 1>,
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
            println!("[G] sending job {}", state.count);
            output.out_job.add_value(state.count).unwrap();
        }

        fn ta(state: &Self::State) -> f64 {
            state.sigma
        }

        fn delta_ext(state: &mut Self::State, elapsed: f64, input: &Self::Input) {
            state.sigma -= elapsed;
            if let Some(&stop) = input.in_stop.get_values().last() {
                println!("[G] received stop: {}", stop);
                if stop {
                    state.sigma = f64::INFINITY;
                }
            }
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
            if let Some(job) = state.job {
                println!("[P] processed job {}", job);
                state.job = None;
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
                print!("[P] received job {}", job);
                if state.job.is_none() {
                    println!(" (idle)");
                    state.job = Some(job);
                    state.sigma = state.time;
                } else {
                    println!(" (busy)");
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

mod transducer {
    #[xdevs::atomic]
    pub struct Transducer {
        #[input]
        pub in_generator: xdevs::port::Port<usize, 1>,
        pub in_processor: xdevs::port::Port<usize, 1>,
        #[output]
        pub out_stop: xdevs::port::Port<bool, 1>,
        #[state]
        sigma: f64,
        clock: f64,
        n_generated: usize,
        n_processed: usize,
    }

    impl xdevs::Atomic for Transducer {
        fn delta_int(state: &mut Self::State) {
            state.clock += state.sigma;
            let (acceptance, throughput) = if state.n_processed > 0 {
                (
                    state.n_processed as f64 / state.n_generated as f64,
                    state.n_processed as f64 / state.clock,
                )
            } else {
                (0.0, 0.0)
            };
            println!(
                "[T] acceptance: {:.2}, throughput: {:.2}",
                acceptance, throughput
            );
            state.sigma = f64::INFINITY;
        }

        fn lambda(_state: &Self::State, output: &mut Self::Output) {
            output.out_stop.add_value(true).unwrap();
        }

        fn ta(state: &Self::State) -> f64 {
            state.sigma
        }

        fn delta_ext(state: &mut Self::State, elapsed: f64, input: &Self::Input) {
            state.sigma -= elapsed;
            state.clock += elapsed;
            state.n_generated += input.in_generator.get_values().len();
            state.n_processed += input.in_processor.get_values().len();
        }
    }

    impl Transducer {
        pub fn new(obs_time: f64) -> Self {
            Self::build(obs_time, 0.0, 0, 0)
        }
    }
}

#[xdevs::coupled]
pub struct GPT {
    #[components]
    generator: generator::Generator,
    processor: processor::Processor,
    transducer: transducer::Transducer,
}

impl xdevs::Coupled for GPT {
    fn ic(from: &Self::ComponentsOutput, to: &mut Self::ComponentsInput) {
        from.generator
            .out_job
            .couple(&mut to.processor.in_job)
            .unwrap();
        from.processor
            .out_job
            .couple(&mut to.transducer.in_processor)
            .unwrap();
        from.generator
            .out_job
            .couple(&mut to.transducer.in_generator)
            .unwrap();
        from.transducer
            .out_stop
            .couple(&mut to.generator.in_stop)
            .unwrap();
    }
}

#[xdevs::coupled]
struct EF {
    #[input]
    in_processor: xdevs::port::Port<usize, 1>,
    #[output]
    out_generator: xdevs::port::Port<usize, 1>,
    #[components]
    generator: generator::Generator,
    transducer: transducer::Transducer,
}

impl xdevs::Coupled for EF {
    fn ic(from: &Self::ComponentsOutput, to: &mut Self::ComponentsInput) {
        from.generator
            .out_job
            .couple(&mut to.transducer.in_generator)
            .unwrap();
        from.transducer
            .out_stop
            .couple(&mut to.generator.in_stop)
            .unwrap();
    }
    fn eic(from: &Self::Input, to: &mut Self::ComponentsInput) {
        from.in_processor
            .couple(&mut to.transducer.in_processor)
            .unwrap();
    }
    fn eoc(from: &Self::ComponentsOutput, to: &mut Self::Output) {
        from.generator
            .out_job
            .couple(&mut to.out_generator)
            .unwrap();
    }
}

#[xdevs::coupled]
pub struct EFP {
    #[components]
    ef: EF,
    processor: processor::Processor,
}

impl xdevs::Coupled for EFP {
    fn ic(from: &Self::ComponentsOutput, to: &mut Self::ComponentsInput) {
        from.ef
            .out_generator
            .couple(&mut to.processor.in_job)
            .unwrap();
        from.processor
            .out_job
            .couple(&mut to.ef.in_processor)
            .unwrap();
    }
}

#[tokio::main]
async fn main() {
    let period = 1.;
    let proc_time = 1.1;
    let obs_time = 10.;

    let generator = generator::Generator::new(period);
    let processor = processor::Processor::new(proc_time);
    let transducer = transducer::Transducer::new(obs_time);

    let ef = EF::build(generator, transducer);
    let efp = EFP::build(ef, processor);

    let mut simulator = Simulator::new(efp);
    let config = Config::new(0.0, 14.0, 1.0, None);
    let input_handler = SleepAsync::new();

    simulator
        .simulate_rt_async(&config, input_handler, |_| {})
        .await;
}
