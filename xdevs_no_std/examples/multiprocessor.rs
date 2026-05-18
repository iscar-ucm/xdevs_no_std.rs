/// Example demonstrating multiple models with port arrays for coupling.
/// This example shows a load balancer that distributes jobs to multiple processors.
use embassy_time::{Duration as eDuration, Instant as eInstant};

mod processor {
    use crate::eDuration;
    use xdevs::port::Port;

    #[xdevs::atomic]
    pub struct Processor {
        #[input]
        pub in_job: Port<usize, 1>,
        #[output]
        pub out_job: Port<usize, 1>,
        #[state]
        sigma: eDuration,
        time: eDuration,
        id: usize,
        job: Option<usize>,
    }

    impl xdevs::Atomic for Processor {
        fn delta_int(state: &mut Self::State) {
            if let Some(job) = state.job.take() {
                println!("[P{}] processed job {}", state.id, job);
            }
            state.sigma = eDuration::MAX;
        }

        fn lambda(state: &Self::State, output: &mut Self::Output) {
            if let Some(job) = state.job {
                output.out_job.add_value(job).unwrap();
            }
        }

        fn ta(state: &Self::State) -> eDuration {
            state.sigma
        }

        fn delta_ext(state: &mut Self::State, elapsed: eDuration, input: &Self::Input) {
            state.sigma -= elapsed;
            if let Some(&job) = input.in_job.get_values().last() {
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
        pub fn new(id: usize, time: eDuration) -> Self {
            Self::build(eDuration::MAX, time, id, None)
        }
    }
}

mod load_balancer {
    use crate::eDuration;
    use xdevs::port::Port;

    /// A load balancer that receives jobs and distributes them round-robin to 3 output ports.
    #[xdevs::atomic]
    pub struct LoadBalancer {
        #[input]
        pub in_job: Port<usize, 1>,
        #[output]
        pub out_jobs: [Port<usize, 1>; 3],
        #[state]
        sigma: eDuration,
        next_processor: usize,
        pending_job: Option<usize>,
    }

    impl xdevs::Atomic for LoadBalancer {
        fn delta_int(state: &mut Self::State) {
            state.pending_job = None;
            state.sigma = eDuration::MAX;
        }

        fn lambda(state: &Self::State, output: &mut Self::Output) {
            if let Some(job) = state.pending_job {
                let target = state.next_processor;
                println!("[LB] routing job {} to processor {}", job, target);
                output.out_jobs[target].add_value(job).unwrap();
            }
        }

        fn ta(state: &Self::State) -> eDuration {
            state.sigma
        }

        fn delta_ext(state: &mut Self::State, elapsed: eDuration, input: &Self::Input) {
            state.sigma -= elapsed;
            if let Some(&job) = input.in_job.get_values().last() {
                println!("[LB] received job {}", job);
                state.pending_job = Some(job);
                state.next_processor = (state.next_processor + 1) % 3;
                state.sigma = eDuration::from_secs(0); // Immediate output
            }
        }
    }

    impl LoadBalancer {
        pub fn new() -> Self {
            Self::build(eDuration::MAX, 0, None)
        }
    }
}

mod generator {
    use crate::eDuration;
    use xdevs::port::Port;

    #[xdevs::atomic]
    pub struct Generator {
        #[output]
        pub out_job: Port<usize, 1>,
        #[state]
        sigma: eDuration,
        period: eDuration,
        count: usize,
        max_jobs: usize,
    }

    impl xdevs::Atomic for Generator {
        fn delta_int(state: &mut Self::State) {
            state.count += 1;
            if state.count >= state.max_jobs {
                state.sigma = eDuration::MAX;
            } else {
                state.sigma = state.period;
            }
        }

        fn lambda(state: &Self::State, output: &mut Self::Output) {
            println!("[G] generating job {}", state.count);
            output.out_job.add_value(state.count).unwrap();
        }

        fn ta(state: &Self::State) -> eDuration {
            state.sigma
        }

        fn delta_ext(state: &mut Self::State, elapsed: eDuration, _input: &Self::Input) {
            state.sigma -= elapsed;
        }
    }

    impl Generator {
        pub fn new(period: eDuration, max_jobs: usize) -> Self {
            Self::build(eDuration::from_secs(0), period, 0, max_jobs)
        }
    }
}

mod collector {
    use crate::eDuration;
    use xdevs::port::Port;

    /// Collects processed jobs from multiple processors via a single input port.
    #[xdevs::atomic]
    pub struct Collector {
        #[input]
        pub in_jobs: Port<usize, 3>,
        #[state]
        sigma: eDuration,
        total_collected: usize,
    }

    impl xdevs::Atomic for Collector {
        fn delta_int(state: &mut Self::State) {
            state.sigma = eDuration::MAX;
        }

        fn lambda(_state: &Self::State, _output: &mut Self::Output) {}

        fn ta(state: &Self::State) -> eDuration {
            state.sigma
        }

        fn delta_ext(state: &mut Self::State, elapsed: eDuration, input: &Self::Input) {
            state.sigma -= elapsed;
            for &job in input.in_jobs.get_values() {
                state.total_collected += 1;
                println!(
                    "[C] collected job {} (total: {})",
                    job, state.total_collected
                );
            }
        }
    }

    impl Collector {
        pub fn new() -> Self {
            Self::build(eDuration::MAX, 0)
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
    let generator = generator::Generator::new(eDuration::from_secs(1), 10);
    let load_balancer = load_balancer::LoadBalancer::new();
    let processor0 = processor::Processor::new(0, eDuration::from_millis(2500));
    let processor1 = processor::Processor::new(1, eDuration::from_millis(2500));
    let processor2 = processor::Processor::new(2, eDuration::from_millis(2500));
    let collector = collector::Collector::new();

    let model = MultiProcessor::build(
        generator,
        load_balancer,
        [processor0, processor1, processor2],
        collector,
    );

    let mut simulator = xdevs::simulator::Simulator::new(model);
    let config =
        xdevs::simulator::Config::new(eInstant::from_secs(0), eInstant::from_secs(15), 1, None);
    simulator.simulate_rt(&config, xdevs::simulator::std::sleep(&config), |_| {});
}
