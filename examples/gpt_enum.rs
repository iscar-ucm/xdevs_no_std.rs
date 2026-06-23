/// Illustrates #[xdevs::modelenum]: a GPT model where the processor is
/// chosen at build time between a fast and a slow variant, without any
/// conditional logic in the coupled model.
use xdevs::simulation::Simulable;
use xdevs::AbstractSimulator;

mod generator {
    pub struct Generator {
        sigma: f64,
        period: f64,
        count: usize,
    }

    impl xdevs::Component for Generator {
        type Kind = xdevs::AtomicKind;
        type Input = xdevs::Port<bool, 1>;
        type Output = xdevs::Port<usize, 1>;
    }

    impl xdevs::Atomic for Generator {
        fn delta_int(&mut self) {
            self.count += 1;
            self.sigma = self.period;
            println!("[G] generated job {}", self.count);
        }
        fn lambda(&self, output: &mut Self::Output) {
            output.add_value(self.count).unwrap();
        }
        fn ta(&self) -> f64 {
            self.sigma
        }
        fn delta_ext(&mut self, elapsed: f64, input: &Self::Input) {
            self.sigma -= elapsed;
            if let Some(&stop) = input.get_values().last() {
                if stop {
                    self.sigma = f64::INFINITY;
                }
            }
        }
    }

    impl Generator {
        pub fn new(period: f64) -> Self {
            Self {
                sigma: 0.0,
                period,
                count: 0,
            }
        }
    }
}

mod processor {
    pub struct FastProcessor {
        sigma: f64,
        time: f64,
        job: Option<usize>,
    }

    impl xdevs::Component for FastProcessor {
        type Kind = xdevs::AtomicKind;
        type Input = xdevs::Port<usize, 1>;
        type Output = xdevs::Port<usize, 1>;
    }

    impl xdevs::Atomic for FastProcessor {
        fn delta_int(&mut self) {
            self.sigma = f64::INFINITY;
            if let Some(job) = self.job {
                println!("[P-fast] processed job {}", job);
            }
            self.job = None;
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
                    println!("[P-fast] received job {}", job);
                    self.job = Some(job);
                    self.sigma = self.time;
                }
            }
        }
    }

    impl FastProcessor {
        pub fn new() -> Self {
            Self {
                sigma: f64::INFINITY,
                time: 0.5,
                job: None,
            }
        }
    }

    pub struct SlowProcessor {
        sigma: f64,
        time: f64,
        job: Option<usize>,
    }

    impl xdevs::Component for SlowProcessor {
        type Kind = xdevs::AtomicKind;
        type Input = xdevs::Port<usize, 1>;
        type Output = xdevs::Port<usize, 1>;
    }

    impl xdevs::Atomic for SlowProcessor {
        fn delta_int(&mut self) {
            self.sigma = f64::INFINITY;
            if let Some(job) = self.job {
                println!("[P-slow] processed job {}", job);
            }
            self.job = None;
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
                    println!("[P-slow] received job {}", job);
                    self.job = Some(job);
                    self.sigma = self.time;
                }
            }
        }
    }

    impl SlowProcessor {
        pub fn new() -> Self {
            Self {
                sigma: f64::INFINITY,
                time: 2.0,
                job: None,
            }
        }
    }

    #[xdevs::modelenum]
    pub enum Processor {
        Fast(FastProcessor),
        Slow(SlowProcessor),
    }
}

mod transducer {
    #[derive(xdevs::Bag)]
    pub struct TransducerInput {
        pub in_generator: xdevs::Port<usize, 1>,
        pub in_processor: xdevs::Port<usize, 1>,
    }

    pub struct Transducer {
        sigma: f64,
        clock: f64,
        n_generated: usize,
        n_processed: usize,
    }

    impl xdevs::Component for Transducer {
        type Kind = xdevs::AtomicKind;
        type Input = TransducerInput;
        type Output = xdevs::Port<bool, 1>;
    }

    impl xdevs::Atomic for Transducer {
        fn delta_int(&mut self) {
            self.clock += self.sigma;
            let (acceptance, throughput) = if self.n_processed > 0 {
                (
                    self.n_processed as f64 / self.n_generated as f64,
                    self.n_processed as f64 / self.clock,
                )
            } else {
                (0.0, 0.0)
            };
            println!(
                "[T] acceptance: {:.2}, throughput: {:.2} jobs/s",
                acceptance, throughput
            );
            self.sigma = f64::INFINITY;
        }
        fn lambda(&self, output: &mut Self::Output) {
            output.add_value(true).unwrap();
        }
        fn ta(&self) -> f64 {
            self.sigma
        }
        fn delta_ext(&mut self, elapsed: f64, input: &Self::Input) {
            self.sigma -= elapsed;
            self.clock += elapsed;
            self.n_generated += input.in_generator.get_values().len();
            self.n_processed += input.in_processor.get_values().len();
        }
    }

    impl Transducer {
        pub fn new(obs_time: f64) -> Self {
            Self {
                sigma: obs_time,
                clock: 0.0,
                n_generated: 0,
                n_processed: 0,
            }
        }
    }
}

#[xdevs::coupled]
pub struct GPT {
    generator: generator::Generator,
    processor: processor::Processor,
    transducer: transducer::Transducer,
}

impl xdevs::Component for GPT {
    type Kind = xdevs::CoupledKind;
    type Input = ();
    type Output = ();
}

impl xdevs::Coupled for GPT {
    fn ic(
        from: &xdevs::component::coupled::ComponentsOutput<Self>,
        to: &mut xdevs::component::coupled::ComponentsInput<Self>,
    ) {
        from.generator.couple(&mut to.processor).unwrap();
        from.processor
            .couple(&mut to.transducer.in_processor)
            .unwrap();
        from.generator
            .couple(&mut to.transducer.in_generator)
            .unwrap();
        from.transducer.couple(&mut to.generator).unwrap();
    }
}

fn run_gpt(processor: processor::Processor) {
    let label = match &processor {
        processor::Processor::Fast(_) => "fast",
        processor::Processor::Slow(_) => "slow",
    };
    println!("\n--- GPT with {} processor ---", label);
    let gpt = GPT::build(
        generator::Generator::new(1.0),
        processor,
        transducer::Transducer::new(10.0),
    );
    let mut simulator = gpt.to_simulator();
    let config = xdevs::simulation::Config::new(0.0, 14.0, 1.0, None);
    simulator.simulate_rt(&config, xdevs::simulation::std::sleep(&config), |_| {});
}

fn main() {
    let fast = processor::Processor::Fast(processor::FastProcessor::new().to_simulator());
    run_gpt(fast);

    let slow = processor::Processor::Slow(processor::SlowProcessor::new().to_simulator());
    run_gpt(slow);
}
