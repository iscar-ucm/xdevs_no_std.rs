use embassy_time::{Duration as eDuration, Instant as eInstant};

mod generator {
    use crate::eDuration;
    pub struct GeneratorState {
        sigma: eDuration,
        period: eDuration,
        count: usize,
    }

    impl GeneratorState {
        pub fn new(period: eDuration) -> Self {
            Self {
                sigma: eDuration::from_millis(0),
                period,
                count: 0,
            }
        }
    }

    xdevs::component!(
        ident = Generator,
        input = {
            in_stop<bool>,
        },
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

        fn ta(state: &Self::State) -> eDuration {
            state.sigma
        }

        fn delta_ext(state: &mut Self::State, e: eDuration, x: &Self::Input) {
            state.sigma -= e;
            if let Some(&stop) = x.in_stop.get_values().last() {
                println!("[G] received stop: {}", stop);
                if stop {
                    state.sigma = eDuration::MAX;
                }
            }
        }
    }
}

mod processor {
    use crate::eDuration;
    pub struct ProcessorState {
        sigma: eDuration,
        time: eDuration,
        job: Option<usize>,
    }

    impl ProcessorState {
        pub fn new(time: eDuration) -> Self {
            Self {
                sigma: eDuration::from_millis(0),
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
            state.sigma = eDuration::MAX;
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

        fn ta(state: &Self::State) -> eDuration {
            state.sigma
        }

        fn delta_ext(state: &mut Self::State, e: eDuration, x: &Self::Input) {
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

mod transducer {
    use crate::eDuration;
    pub struct TransducerState {
        sigma: eDuration,
        clock: eDuration,
        n_generated: usize,
        n_processed: usize,
    }

    impl TransducerState {
        pub fn new(obs_time: eDuration) -> Self {
            Self {
                sigma: obs_time,
                clock: eDuration::from_millis(0),
                n_generated: 0,
                n_processed: 0,
            }
        }
    }

    xdevs::component!(
        ident = Transducer,
        input = {
            in_generator<usize, 1>,
            in_processor<usize, 1>,
        },
        output = {
            out_stop<bool>
        },
        state = TransducerState,
    );

    impl xdevs::Atomic for Transducer {
        fn delta_int(state: &mut Self::State) {
            state.clock += state.sigma;
            let (acceptance, throughput) = if state.n_processed > 0 {
                (
                    state.n_processed as f64 / state.n_generated as f64,
                    state.n_processed as f64 / state.clock.as_millis() as f64,
                )
            } else {
                (0.0, 0.0)
            };
            println!(
                "[T] acceptance: {:.2}, throughput: {:.5}",
                acceptance, throughput
            );
            state.sigma = eDuration::MAX;
        }

        fn lambda(_state: &Self::State, output: &mut Self::Output) {
            output.out_stop.add_value(true).unwrap();
        }

        fn ta(state: &Self::State) -> eDuration {
            state.sigma
        }

        fn delta_ext(state: &mut Self::State, e: eDuration, x: &Self::Input) {
            state.sigma -= e;
            state.clock += e;
            state.n_generated += x.in_generator.get_values().len();
            state.n_processed += x.in_processor.get_values().len();
        }
    }
}

xdevs::component!(
    ident = GPT,
    components = {
        generator: generator::Generator,
        processor: processor::Processor,
        transducer: transducer::Transducer,
    },
    couplings = {
        generator.out_job -> processor.in_job,
        processor.out_job -> transducer.in_processor,
        generator.out_job -> transducer.in_generator,
        transducer.out_stop -> generator.in_stop,
    }
);

xdevs::component!(
    ident = EF,
    input = {
        in_processor<usize, 1>,
    },
    output = {
        out_generator<usize, 1>,
    },
    components = {
        generator: generator::Generator,
        transducer: transducer::Transducer,
    },
    couplings = {
        in_processor -> transducer.in_processor,
        generator.out_job -> transducer.in_generator,
        transducer.out_stop -> generator.in_stop,
        generator.out_job -> out_generator,
    }
);

xdevs::component!(
    ident = EFP,
    components = {
        ef: EF,
        processor: processor::Processor,
    },
    couplings = {
        ef.out_generator -> processor.in_job,
        processor.out_job -> ef.in_processor,
    }
);

fn main() {
    let period = eDuration::from_millis(1000); //1.0
    let proc_time = eDuration::from_millis(1100); //1.1
    let obs_time = eDuration::from_millis(10000); //10.0

    let generator = generator::Generator::new(generator::GeneratorState::new(period));
    let processor = processor::Processor::new(processor::ProcessorState::new(proc_time));
    let transducer = transducer::Transducer::new(transducer::TransducerState::new(obs_time));

    let ef = EF::new(generator, transducer);
    let efp = EFP::new(ef, processor);

    let mut simulator = xdevs::simulator::Simulator::new(efp);
    let config =
        xdevs::simulator::Config::new(eInstant::from_millis(0), eInstant::from_secs(20), 1, None);
    simulator.simulate_rt(&config, xdevs::simulator::std::sleep(&config), |_| {});

    //simulator.simulate_vt(&config);
}
