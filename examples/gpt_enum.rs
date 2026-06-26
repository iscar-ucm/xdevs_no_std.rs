/// Illustrates #[xdevs::to_component] on an enum: a GPT model where the processor is
/// chosen at build time between a fast and a slow variant, without any
/// conditional logic in the coupled model.
use xdevs::{
    gpt::{Generator, Transducer},
    simulation::{AbstractSimulator, Simulable},
};

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
        pub fn new(time: f64) -> Self {
            Self {
                sigma: 0.0,
                time,
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
                    self.sigma = self.time * 2.0;
                }
            }
        }
    }

    impl SlowProcessor {
        pub fn new(time: f64) -> Self {
            Self {
                sigma: 0.0,
                time,
                job: None,
            }
        }
    }

    #[xdevs::to_component]
    pub enum Processor {
        Fast(FastProcessor),
        Slow(SlowProcessor),
    }
}

#[xdevs::to_component]
pub struct GPT {
    generator: Generator,
    processor: processor::Processor,
    transducer: Transducer,
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
    const PERIOD: f64 = 1.;
    const OBS_TIME: f64 = 10.;

    let label = match &processor {
        processor::Processor::Fast(_) => "fast",
        processor::Processor::Slow(_) => "slow",
    };
    println!("\n--- GPT with {} processor ---", label);
    let gpt = GPT::build(Generator::new(PERIOD), processor, Transducer::new(OBS_TIME));
    let mut simulator = gpt.to_simulator();
    let config = xdevs::simulation::Config::new(0.0, 14.0, 1.0, None);
    simulator.simulate_rt(&config, xdevs::simulation::std::sleep(&config), |_| {});
}

fn main() {
    const PROC_TIME: f64 = 1.1;
    let fast = processor::Processor::Fast(processor::FastProcessor::new(PROC_TIME).to_simulator());
    run_gpt(fast);

    let slow = processor::Processor::Slow(processor::SlowProcessor::new(PROC_TIME).to_simulator());
    run_gpt(slow);
}
