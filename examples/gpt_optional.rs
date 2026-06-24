/// GPT-like example with an optional processor.
use xdevs::{component::coupled::ComponentsOutput, simulation::Simulable, AbstractSimulator};

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
        }
        fn lambda(&self, output: &mut Self::Output) {
            println!("[G] sending job {}", self.count);
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
    pub struct Processor {
        sigma: f64,
        time: f64,
        job: Option<usize>,
    }
    impl xdevs::Component for Processor {
        type Kind = xdevs::AtomicKind;
        type Input = xdevs::Port<usize, 1>;
        type Output = xdevs::Port<usize, 1>;
    }
    impl xdevs::Atomic for Processor {
        fn delta_int(&mut self) {
            self.sigma = f64::INFINITY;
            if let Some(job) = self.job {
                println!("[P] processed job {}", job);
                self.job = None;
            }
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
                println!("[P] received job {}", job);
                if self.job.is_none() {
                    self.job = Some(job);
                    self.sigma = self.time;
                }
            }
        }
    }
    impl Processor {
        pub fn new(time: f64) -> Self {
            Self {
                sigma: 0.0,
                time,
                job: None,
            }
        }
    }
}

mod transducer {
    #[derive(xdevs::Bag)]
    pub struct TransducerInput {
        pub from_generator: xdevs::Port<usize, 1>,
        pub from_processor: xdevs::Port<usize, 1>,
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
            self.n_generated += input.from_generator.get_values().len();
            self.n_processed += input.from_processor.get_values().len();
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

/// Coupled model with an optional processor, demonstrates
/// `Option<Processor>` as a component field.
///
/// The coupling code routes unconditionally (same type whether present or not).
/// When the processor is `None`, it silently absorbs any routed events in its `delta`.
#[xdevs::coupled]
pub struct GPTOptional {
    generator: generator::Generator,
    processor: Option<processor::Processor>,
    transducer: transducer::Transducer,
}

impl xdevs::Component for GPTOptional {
    type Kind = xdevs::CoupledKind;
    type Input = ();
    type Output = ();
}

impl xdevs::Coupled for GPTOptional {
    fn ic(
        from: &ComponentsOutput<Self>,
        to: &mut xdevs::component::coupled::ComponentsInput<Self>,
    ) {
        from.generator.couple(&mut to.processor).unwrap();
        from.generator
            .couple(&mut to.transducer.from_generator)
            .unwrap();
        from.processor
            .couple(&mut to.transducer.from_processor)
            .unwrap();
        from.transducer.couple(&mut to.generator).unwrap();
    }
}

fn run_gpt(some_processor: bool) {
    let period = 1.;
    let proc_time = 1.1;
    let obs_time = 10.;

    let processor = if some_processor {
        Some(processor::Processor::new(proc_time))
    } else {
        None
    };
    let label = if some_processor { "some" } else { "no" };
    println!("\n--- GPT with {} processor ---", label);
    let gpt = GPTOptional::build(
        generator::Generator::new(period),
        processor,
        transducer::Transducer::new(obs_time),
    );
    let mut simulator = gpt.to_simulator();
    let config = xdevs::simulation::Config::new(0.0, 14.0, 1.0, None);
    simulator.simulate_rt(&config, xdevs::simulation::std::sleep(&config), |_| {});
}

fn main() {
    run_gpt(true);
    run_gpt(false);
}
