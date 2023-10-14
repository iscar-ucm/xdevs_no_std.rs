mod generator {
    pub struct GeneratorState {
        sigma: f64,
        period: f64,
        count: usize,
    }

    impl GeneratorState {
        pub fn new(period: f64) -> Self {
            Self {
                sigma: 0.0,
                period,
                count: 0,
            }
        }
    }

    xdevs::component!(
        ident = Generator,
        input = {},
        output = {
            out_job<usize>,
        },
        state = GeneratorState,
    );

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

        fn delta_ext(state: &mut Self::State, e: f64, _x: &Self::Input) {
            state.sigma -= e;
        }
    }
}

mod processor {
    pub struct ProcessorState {
        sigma: f64,
        time: f64,
        job: Option<usize>,
    }

    impl ProcessorState {
        pub fn new(time: f64) -> Self {
            Self {
                sigma: 0.0,
                time,
                job: None,
            }
        }
    }

    xdevs::component!(
        ident = Processor,
        input = {
            in_job<usize, 1>
        },
        output = {
            out_job<usize>
        },
        state = ProcessorState,
    );

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

        fn delta_ext(state: &mut Self::State, e: f64, x: &Self::Input) {
            state.sigma -= e;
            if let Some(&job) = x.in_job.get_values().last() {
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
}

xdevs::component!(
    ident = GPT,
    components = {
        generator: generator::Generator,
        processor: processor::Processor,
    },
    couplings = {
        generator.out_job -> processor.in_job,
    }
);

fn main() {
    let period = 1.;
    let time = 1.5;

    let generator = generator::Generator::new(generator::GeneratorState::new(period));
    let processor = processor::Processor::new(processor::ProcessorState::new(time));

    let model = GPT::new(generator, processor);

    let mut simulator = xdevs::simulator::Simulator::new(model);

    simulator.simulate(0.0, 10.0);
}
