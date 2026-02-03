/// Example demonstrating multiple models with port arrays for coupling.
/// This example shows a load balancer that distributes jobs to multiple processors.

mod processor {
    use xdevs::port::Port;

    #[xdevs::atomic]
    pub struct Processor {
        #[input]
        pub in_job: Port<usize, 1>,
        #[output]
        pub out_job: Port<usize, 1>,
        #[state]
        sigma: f64,
        time: f64,
        id: usize,
        job: Option<usize>,
    }

    impl xdevs::Atomic for Processor {
        fn delta_int(state: &mut Self::State) {
            if let Some(job) = state.job.take() {
                println!("[P{}] processed job {}", state.id, job);
            }
            state.sigma = f64::INFINITY;
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
                if state.job.is_none() {
                    println!("[P{}] received job {} (idle)", state.id, job);
                    state.job = Some(job);
                    state.sigma = state.time;
                } else {
                    println!("[P{}] received job {} (busy, dropped)", state.id, job);
                }
            }
        }
    }

    impl Processor {
        pub fn new2(id: usize, time: f64) -> Self {
            Self::new(f64::INFINITY, time, id, None)
        }
    }
}

mod load_balancer {
    use xdevs::port::Port;

    /// A load balancer that receives jobs and distributes them round-robin to 3 output ports.
    #[xdevs::atomic]
    pub struct LoadBalancer {
        #[input]
        pub in_job: Port<usize, 1>,
        #[output]
        pub out_jobs: [Port<usize, 1>; 3],
        #[state]
        sigma: f64,
        next_processor: usize,
        pending_job: Option<usize>,
    }

    impl xdevs::Atomic for LoadBalancer {
        fn delta_int(state: &mut Self::State) {
            state.pending_job = None;
            state.sigma = f64::INFINITY;
        }

        fn lambda(state: &Self::State, output: &mut Self::Output) {
            if let Some(job) = state.pending_job {
                let target = state.next_processor;
                println!("[LB] routing job {} to processor {}", job, target);
                output.out_jobs[target].add_value(job).unwrap();
            }
        }

        fn ta(state: &Self::State) -> f64 {
            state.sigma
        }

        fn delta_ext(state: &mut Self::State, e: f64, x: &Self::Input) {
            state.sigma -= e;
            if let Some(&job) = x.in_job.get_values().last() {
                println!("[LB] received job {}", job);
                state.pending_job = Some(job);
                state.next_processor = (state.next_processor + 1) % 3;
                state.sigma = 0.0; // Immediate output
            }
        }
    }

    impl LoadBalancer {
        pub fn new2() -> Self {
            Self::new(f64::INFINITY, 0, None)
        }
    }
}

mod generator {
    use xdevs::port::Port;

    #[xdevs::atomic]
    pub struct Generator {
        #[output]
        pub out_job: Port<usize, 1>,
        #[state]
        sigma: f64,
        period: f64,
        count: usize,
        max_jobs: usize,
    }

    impl xdevs::Atomic for Generator {
        fn delta_int(state: &mut Self::State) {
            state.count += 1;
            if state.count >= state.max_jobs {
                state.sigma = f64::INFINITY;
            } else {
                state.sigma = state.period;
            }
        }

        fn lambda(state: &Self::State, output: &mut Self::Output) {
            println!("[G] generating job {}", state.count);
            output.out_job.add_value(state.count).unwrap();
        }

        fn ta(state: &Self::State) -> f64 {
            state.sigma
        }

        fn delta_ext(state: &mut Self::State, e: f64, _x: &Self::Input) {
            state.sigma -= e;
        }
    }

    impl Generator {
        pub fn new2(period: f64, max_jobs: usize) -> Self {
            Self::new(0.0, period, 0, max_jobs)
        }
    }
}

mod collector {
    use xdevs::port::Port;

    /// Collects processed jobs from multiple processors via a single input port.
    #[xdevs::atomic]
    pub struct Collector {
        #[input]
        pub in_jobs: Port<usize, 3>,
        #[state]
        sigma: f64,
        total_collected: usize,
    }

    impl xdevs::Atomic for Collector {
        fn delta_int(state: &mut Self::State) {
            state.sigma = f64::INFINITY;
        }

        fn lambda(_state: &Self::State, _output: &mut Self::Output) {}

        fn ta(state: &Self::State) -> f64 {
            state.sigma
        }

        fn delta_ext(state: &mut Self::State, e: f64, x: &Self::Input) {
            state.sigma -= e;
            for &job in x.in_jobs.get_values() {
                state.total_collected += 1;
                println!(
                    "[C] collected job {} (total: {})",
                    job, state.total_collected
                );
            }
        }
    }

    impl Collector {
        pub fn new2() -> Self {
            Self::new(f64::INFINITY, 0)
        }
    }
}

/// Coupled model with multiple processors.
/// The load balancer distributes jobs to processors, and the collector gathers results.
#[xdevs::coupled(
    couplings = {
        generator.out_job -> load_balancer.in_job,
        // Connect load balancer outputs to each processor (1-to-1 zipped)
        zip(load_balancer.out_jobs.iter()) -> processor.iter_mut().map(|p| &mut p.input.in_job),
        // Connect all processor outputs to the single collector input
        processor.iter().map(|p| &p.output.out_job) -> collector.in_jobs,
    }
)]
struct MultiProcessor {
    #[components]
    generator: generator::Generator,
    load_balancer: load_balancer::LoadBalancer,
    processor: [processor::Processor; 3],
    collector: collector::Collector,
}

fn main() {
    let generator = generator::Generator::new2(1.0, 10);
    let load_balancer = load_balancer::LoadBalancer::new2();
    let processor0 = processor::Processor::new2(0, 2.5);
    let processor1 = processor::Processor::new2(1, 2.5);
    let processor2 = processor::Processor::new2(2, 2.5);
    let collector = collector::Collector::new2();

    let model = MultiProcessor::new(
        generator,
        load_balancer,
        [processor0, processor1, processor2],
        collector,
    );

    let mut simulator = xdevs::simulator::Simulator::new(model);
    let config = xdevs::simulator::Config::new(0.0, 15.0, 1.0, None);
    simulator.simulate_rt(&config, xdevs::simulator::std::sleep(&config), |_| {});
}
