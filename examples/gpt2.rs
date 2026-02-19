use embassy_time::{Duration as eDuration, Instant as eInstant};

mod generator {
    use crate::eDuration;
    #[xdevs::atomic]
    struct Generator {
        #[input]
        in_stop: xdevs::port::Port<bool, 1>,
        #[output]
        out_job: xdevs::port::Port<usize, 1>,
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
    struct Processor {
        #[input]
        in_job: xdevs::port::Port<usize, 1>,
        #[output]
        out_job: xdevs::port::Port<usize, 1>,
        #[state]
        sigma: eDuration,
        time: eDuration,
        job: Option<usize>,
    }

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

    impl Processor {
        pub fn new(time: eDuration) -> Self {
            //cambio new2 por new
            Self::build(eDuration::from_millis(0), time, None) //cambio new por build
        }
    }
}

mod transducer {
    use crate::eDuration;
    #[xdevs::atomic]
    struct Transducer {
        #[input]
        in_generator: xdevs::port::Port<usize, 1>,
        in_processor: xdevs::port::Port<usize, 1>,
        #[output]
        out_stop: xdevs::port::Port<bool, 1>,
        #[state]
        sigma: eDuration,
        clock: eDuration,
        n_generated: usize,
        n_processed: usize,
    }

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
                "[T] acceptance: {:.2}, throughput: {:.2}",
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

    impl Transducer {
        pub fn new(obs_time: eDuration) -> Self {
            //cambio new2 por new
            Self::build(obs_time, eDuration::from_millis(0), 0, 0) //cambio new por build
        }
    }
}

#[xdevs::coupled(
    couplings = {
        generator.out_job -> processor.in_job,
        processor.out_job -> transducer.in_processor,
        generator.out_job -> transducer.in_generator,
        transducer.out_stop -> generator.in_stop,
    }
)]
struct GPT {
    #[components]
    generator: generator::Generator,
    processor: processor::Processor,
    transducer: transducer::Transducer,
}

#[xdevs::coupled(
    couplings = {
        in_processor -> transducer.in_processor,
        generator.out_job -> transducer.in_generator,
        transducer.out_stop -> generator.in_stop,
        generator.out_job -> out_generator,
    }
)]
struct EF {
    #[input]
    in_processor: xdevs::port::Port<usize, 1>,
    #[output]
    out_generator: xdevs::port::Port<usize, 1>,
    #[components]
    generator: generator::Generator,
    transducer: transducer::Transducer,
}

#[xdevs::coupled(
    couplings = {
        ef.out_generator -> processor.in_job,
        processor.out_job -> ef.in_processor,
    }
)]
struct EFP {
    #[components]
    ef: EF,
    processor: processor::Processor,
}

fn main() {
    let period = eDuration::from_millis(1000); //1.0
    let proc_time = eDuration::from_millis(1100); //1.1
    let obs_time = eDuration::from_millis(10000); //10.0

    let generator = generator::Generator::new(period); //cambio new2 por new
    let processor = processor::Processor::new(proc_time); //cambio new2 por new
    let transducer = transducer::Transducer::new(obs_time); //cambio new2 por new

    let ef = EF::new(generator, transducer);
    let efp = EFP::new(ef, processor);

    let mut simulator = xdevs::simulator::Simulator::new(efp);
    let config =
        xdevs::simulator::Config::new(eInstant::from_millis(0), eInstant::from_millis(14), 1, None);
    simulator.simulate_rt(&config, xdevs::simulator::std::sleep(&config), |_| {});

    //simulator.simulate_vt(&config);
}
