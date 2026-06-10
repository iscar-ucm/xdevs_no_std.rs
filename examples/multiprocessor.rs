/// Example demonstrating multiple models with xdevs::port::Port and component arrays for coupling.
/// This example shows a load balancer that distributes jobs to multiple processors.
mod processor {

    pub struct Processor {
        sigma: f64,
        time: f64,
        id: usize,
        job: Option<usize>,
    }

    impl xdevs::traits::Component for Processor {
        type Kind = xdevs::AtomicKind;
        type Input = xdevs::port::Port<usize, 1>;
        type Output = xdevs::port::Port<usize, 1>;
    }
    impl xdevs::Atomic for Processor {
        fn delta_int(&mut self) {
            if let Some(job) = self.job.take() {
                println!("[P{}] processed job {}", self.id, job);
            }
            self.sigma = f64::INFINITY;
        }

        fn lambda(&self, output: &mut Self::Output) {
            if let Some(job) = self.job {
                output.add_value(job).unwrap();
            }
        }

        fn ta(&self) -> f64 {
            self.sigma
        }

        fn delta_ext(&mut self, elapsed: f64, input: &Self::Input) {
            self.sigma -= elapsed;
            if let Some(&job) = input.get_values().last() {
                if self.job.is_none() {
                    println!("[P{}] received job {} (idle)", self.id, job);
                    self.job = Some(job);
                    self.sigma = self.time;
                } else {
                    println!("[P{}] received job {} (busy, dropped)", self.id, job);
                }
            }
        }
    }

    impl Processor {
        pub fn new(id: usize, time: f64) -> Self {
            Self {
                sigma: f64::INFINITY,
                time,
                id,
                job: None,
            }
        }
    }
}

mod load_balancer {
    /// A load balancer that receives jobs and distributes them round-robin to 3 output xdevs::port::Ports.
    pub struct LoadBalancer {
        sigma: f64,
        next_processor: usize,
        pending_job: Option<usize>,
    }

    impl xdevs::traits::Component for LoadBalancer {
        type Kind = xdevs::AtomicKind;
        type Input = xdevs::port::Port<usize, 1>;
        type Output = [xdevs::port::Port<usize, 1>; 3];
    }

    impl xdevs::Atomic for LoadBalancer {
        fn delta_int(&mut self) {
            self.pending_job = None;
            self.sigma = f64::INFINITY;
        }

        fn lambda(&self, output: &mut Self::Output) {
            if let Some(job) = self.pending_job {
                let target = self.next_processor;
                println!("[LB] routing job {} to processor {}", job, target);
                output[target].add_value(job).unwrap();
            }
        }

        fn ta(&self) -> f64 {
            self.sigma
        }

        fn delta_ext(&mut self, elapsed: f64, input: &Self::Input) {
            self.sigma -= elapsed;
            if let Some(&job) = input.get_values().last() {
                println!("[LB] received job {}", job);
                self.pending_job = Some(job);
                self.next_processor = (self.next_processor + 1) % 3;
                self.sigma = 0.0; // Immediate output
            }
        }
    }

    impl LoadBalancer {
        pub fn new() -> Self {
            Self {
                sigma: f64::INFINITY,
                next_processor: 0,
                pending_job: None,
            }
        }
    }
}

mod generator {
    pub struct Generator {
        sigma: f64,
        period: f64,
        count: usize,
        max_jobs: usize,
    }

    impl xdevs::traits::Component for Generator {
        type Kind = xdevs::AtomicKind;
        type Input = ();
        type Output = xdevs::port::Port<usize, 1>;
    }

    impl xdevs::Atomic for Generator {
        fn delta_int(&mut self) {
            self.count += 1;
            if self.count >= self.max_jobs {
                self.sigma = f64::INFINITY;
            } else {
                self.sigma = self.period;
            }
        }

        fn lambda(&self, output: &mut Self::Output) {
            println!("[G] generating job {}", self.count);
            output.add_value(self.count).unwrap();
        }

        fn ta(&self) -> f64 {
            self.sigma
        }

        fn delta_ext(&mut self, elapsed: f64, _input: &Self::Input) {
            self.sigma -= elapsed;
        }
    }

    impl Generator {
        pub fn new(period: f64, max_jobs: usize) -> Self {
            Self {
                sigma: 0.0,
                period,
                count: 0,
                max_jobs,
            }
        }
    }
}

mod collector {
    /// Collects processed jobs from multiple processors via a single input xdevs::port::Port.
    pub struct Collector {
        sigma: f64,
        total_collected: usize,
    }

    impl xdevs::traits::Component for Collector {
        type Kind = xdevs::AtomicKind;
        type Input = xdevs::port::Port<usize, 3>;
        type Output = ();
    }

    impl xdevs::Atomic for Collector {
        fn delta_int(&mut self) {
            self.sigma = f64::INFINITY;
        }

        fn lambda(&self, _output: &mut Self::Output) {}

        fn ta(&self) -> f64 {
            self.sigma
        }

        fn delta_ext(&mut self, elapsed: f64, input: &Self::Input) {
            self.sigma -= elapsed;
            for &job in input.get_values() {
                self.total_collected += 1;
                println!(
                    "[C] collected job {} (total: {})",
                    job, self.total_collected
                );
            }
        }
    }

    impl Collector {
        pub fn new() -> Self {
            Self {
                sigma: f64::INFINITY,
                total_collected: 0,
            }
        }
    }
}

/// Coupled model with multiple processors.
/// The load balancer distributes jobs to processors, and the collector gathers results.
#[xdevs::coupled]
struct MultiProcessor {
    generator: generator::Generator,
    load_balancer: load_balancer::LoadBalancer,
    processor: [processor::Processor; 3],
    collector: collector::Collector,
}

impl xdevs::traits::Component for MultiProcessor {
    type Kind = xdevs::CoupledKind;
    type Input = ();
    type Output = ();
}

impl xdevs::Coupled for MultiProcessor {
    fn eic(_from: &Self::Input, _to: &mut Self::ComponentsInput) {
        // No external input coupling needed, this implementation could be omitted
    }
    fn ic(from: &Self::ComponentsOutput, to: &mut Self::ComponentsInput) {
        from.generator.couple(&mut to.load_balancer).unwrap();
        for (lb_port, proc_port) in from.load_balancer.iter().zip(to.processor.iter_mut()) {
            lb_port.couple(proc_port).unwrap();
        }
        for proc_port in from.processor.iter() {
            proc_port.couple(&mut to.collector).unwrap();
        }
    }
    fn eoc(_from: &Self::ComponentsOutput, _to: &mut Self::Output) {
        // No external output coupling needed, this implementation could be omitted
    }
}

fn main() {
    let generator = generator::Generator::new(1.0, 10);
    let load_balancer = load_balancer::LoadBalancer::new();
    let processor0 = processor::Processor::new(0, 2.5);
    let processor1 = processor::Processor::new(1, 2.5);
    let processor2 = processor::Processor::new(2, 2.5);
    let collector = collector::Collector::new();

    let model = MultiProcessor::build(
        generator,
        load_balancer,
        [processor0, processor1, processor2],
        collector,
    );

    let mut simulator = xdevs::simulator::Simulator::new(model);
    let config = xdevs::simulator::Config::new(0.0, 15.0, 1.0, None);
    simulator.simulate_rt(&config, xdevs::simulator::std::sleep(&config), |_| {});
}
