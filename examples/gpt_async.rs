/// A simple DEVS GPT model with async simulation
use xdevs::simulation::{std::SleepAsync, Config};
use xdevs::{simulation::Simulable, AbstractSimulator};

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
                println!("[G] received stop: {}", stop);
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
                print!("[P] received job {}", job);
                if self.job.is_none() {
                    println!(" (idle)");
                    self.job = Some(job);
                    self.sigma = self.time;
                } else {
                    println!(" (busy)");
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
                "[T] acceptance: {:.2}, throughput: {:.2}",
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

#[xdevs::coupled]
struct EF {
    generator: generator::Generator,
    transducer: transducer::Transducer,
}

impl xdevs::Component for EF {
    type Kind = xdevs::CoupledKind;
    type Input = xdevs::Port<usize, 1>;
    type Output = xdevs::Port<usize, 1>;
}

impl xdevs::Coupled for EF {
    fn ic(
        from: &xdevs::component::coupled::ComponentsOutput<Self>,
        to: &mut xdevs::component::coupled::ComponentsInput<Self>,
    ) {
        from.generator
            .couple(&mut to.transducer.in_generator)
            .unwrap();
        from.transducer.couple(&mut to.generator).unwrap();
    }
    fn eic(from: &Self::Input, to: &mut xdevs::component::coupled::ComponentsInput<Self>) {
        from.couple(&mut to.transducer.in_processor).unwrap();
    }
    fn eoc(from: &xdevs::component::coupled::ComponentsOutput<Self>, to: &mut Self::Output) {
        from.generator.couple(to).unwrap();
    }
}

#[xdevs::coupled]
pub struct EFP {
    ef: EF,
    processor: processor::Processor,
}

impl xdevs::Component for EFP {
    type Kind = xdevs::CoupledKind;
    type Input = ();
    type Output = ();
}
impl xdevs::Coupled for EFP {
    fn ic(
        from: &xdevs::component::coupled::ComponentsOutput<Self>,
        to: &mut xdevs::component::coupled::ComponentsInput<Self>,
    ) {
        from.ef.couple(&mut to.processor).unwrap();
        from.processor.couple(&mut to.ef).unwrap();
    }
}
#[tokio::main]
async fn main() {
    let period = 1.;
    let proc_time = 1.1;
    let obs_time = 10.;

    let generator = generator::Generator::new(period);
    let processor = processor::Processor::new(proc_time);
    let transducer = transducer::Transducer::new(obs_time);

    let ef = EF::build(generator, transducer);
    let efp = EFP::build(ef, processor);

    let mut simulator = efp.to_simulator();
    let config = Config::new(0.0, 14.0, 1.0, None);
    let input_handler = SleepAsync::new();

    simulator
        .simulate_rt_async(&config, input_handler, |_| {})
        .await;
}
