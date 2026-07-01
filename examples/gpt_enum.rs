/// Illustrates #[xdevs::to_component] on an enum: a GPT model where the processor is
/// chosen at build time between a fast and a slow variant, without any
/// conditional logic in the coupled model.
use xdevs::{
    gpt::{Generator, Transducer},
    prelude::*,
    CoupledKind, Duration, Instant,
};

mod processor {
    use xdevs::{AtomicKind, Duration, Port};

    pub struct FastProcessor {
        sigma: Duration,
        time: Duration,
        job: Option<usize>,
    }

    impl xdevs::Component for FastProcessor {
        type Kind = AtomicKind;
        type Input = Port<usize, 1>;
        type Output = Port<usize, 1>;
    }

    impl xdevs::Atomic for FastProcessor {
        fn delta_int(&mut self) {
            self.sigma = Duration::MAX;
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
        fn ta(&self) -> Duration {
            self.sigma
        }
        fn delta_ext(&mut self, elapsed: Duration, input: &Self::Input) {
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
        pub fn new(time: Duration) -> Self {
            Self {
                sigma: Duration::from_secs(0),
                time,
                job: None,
            }
        }
    }

    pub struct SlowProcessor {
        sigma: Duration,
        time: Duration,
        job: Option<usize>,
    }

    impl xdevs::Component for SlowProcessor {
        type Kind = AtomicKind;
        type Input = Port<usize, 1>;
        type Output = Port<usize, 1>;
    }

    impl xdevs::Atomic for SlowProcessor {
        fn delta_int(&mut self) {
            self.sigma = Duration::MAX;
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
        fn ta(&self) -> Duration {
            self.sigma
        }
        fn delta_ext(&mut self, elapsed: Duration, input: &Self::Input) {
            self.sigma -= elapsed;
            if let Some(&job) = input.get_values().last() {
                if self.job.is_none() {
                    println!("[P-slow] received job {}", job);
                    self.job = Some(job);
                    self.sigma = self.time * 2;
                }
            }
        }
    }

    impl SlowProcessor {
        pub fn new(time: Duration) -> Self {
            Self {
                sigma: Duration::from_secs(0),
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

#[xdevs::coupled]
pub struct GPT {
    generator: Generator,
    processor: processor::Processor,
    transducer: Transducer,
}

impl xdevs::Component for GPT {
    type Kind = CoupledKind;
    type Input = ();
    type Output = ();
}

impl xdevs::Coupled for GPT {
    fn ic(from: &xdevs::ComponentsOutput<Self>, to: &mut xdevs::ComponentsInput<Self>) {
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
    let period = Duration::from_secs(1);
    let obs_time = Duration::from_secs(10);

    let label = match &processor {
        processor::Processor::Fast(_) => "fast",
        processor::Processor::Slow(_) => "slow",
    };
    println!("\n--- GPT with {} processor ---", label);
    let gpt = GPT::build(Generator::new(period), processor, Transducer::new(obs_time));
    let mut simulator = gpt.to_simulator();
    let config = xdevs::Config::new(Instant::from_secs(0), Instant::from_secs(14), 1, None);
    simulator.simulate_rt(&config, xdevs::simulation::std::sleep(&config), |_| {});
}

fn main() {
    let proc_time = Duration::from_millis(1100);
    let fast = processor::Processor::Fast(processor::FastProcessor::new(proc_time).to_simulator());
    run_gpt(fast);

    let slow = processor::Processor::Slow(processor::SlowProcessor::new(proc_time).to_simulator());
    run_gpt(slow);
}
