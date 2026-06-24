/// A simple GPB (Generator-Processor-Buffer) model using xDEVS
/// This example shows how to apply Rust's resources for structs
/// (like generics and lifetimes) in DEVS models
use xdevs::{simulation::Simulable, AbstractSimulator};

mod generator {
    pub struct Generator {
        sigma: f64,
        period: f64,
        count: usize,
    }

    impl xdevs::Component for Generator {
        type Input = ();
        type Output = xdevs::Port<usize, 1>;
        type Kind = xdevs::AtomicKind;
    }

    impl xdevs::Atomic for Generator {
        fn delta_int(&mut self) {
            self.count += 1;
            self.sigma = self.period;
        }

        fn lambda(&self, output: &mut Self::Output) {
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
        type Input = xdevs::Port<usize, 1>;
        type Output = xdevs::Port<usize, 1>;
        type Kind = xdevs::AtomicKind;
    }

    impl xdevs::Atomic for Processor {
        fn delta_int(&mut self) {
            self.sigma = f64::INFINITY;
            if let Some(job) = self.job.take() {
                println!("[P] processed job {}", job);
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

mod buffer {
    use core::fmt::Debug;

    pub struct Buffer<'a, T: Clone + Debug> {
        sigma: f64,
        capacity: usize,
        queue: heapless::Vec<T, 16>,
        config: Option<&'a str>,
    }

    impl<'a, T: Clone + Debug> xdevs::Component for Buffer<'a, T> {
        type Input = xdevs::Port<T, 1>;
        type Output = xdevs::Port<T, 1>;
        type Kind = xdevs::AtomicKind;
    }
    impl<'a, T: Clone + Debug> xdevs::Atomic for Buffer<'a, T> {
        fn delta_int(&mut self) {
            if !self.queue.is_empty() {
                self.queue.remove(0);
            }
            self.sigma = if self.queue.is_empty() {
                f64::INFINITY
            } else {
                1.0
            };
        }

        fn lambda(&self, output: &mut Self::Output) {
            if let Some(item) = self.queue.first() {
                if let Some(cfg) = self.config {
                    println!("[B:{}] sending item {:?}", cfg, item);
                } else {
                    println!("[B] sending item {:?}", item);
                }
                output.add_value(item.clone()).unwrap();
            }
        }

        fn ta(&self) -> f64 {
            self.sigma
        }

        fn delta_ext(&mut self, elapsed: f64, input: &Self::Input) {
            self.sigma -= elapsed;
            for item in input.get_values() {
                if self.queue.len() < self.capacity {
                    println!("[B] received item {:?}", item);
                    self.queue.push(item.clone()).unwrap();
                } else {
                    println!("[B] buffer full, dropping {:?}", item);
                }
            }
            if !self.queue.is_empty() {
                self.sigma = 1.0;
            }
        }
    }

    impl<'a, T: Clone + Debug> Buffer<'a, T> {
        pub fn new(capacity: usize, config: Option<&'a str>) -> Self {
            Self {
                sigma: f64::INFINITY,
                capacity,
                queue: heapless::Vec::new(),
                config,
            }
        }
    }
}

#[xdevs::coupled]
struct GPB<'a> {
    generator: generator::Generator,
    buffer: buffer::Buffer<'a, usize>,
    processor: processor::Processor,
}

impl xdevs::Component for GPB<'_> {
    type Kind = xdevs::CoupledKind;
    type Input = ();
    type Output = ();
}

impl<'a> xdevs::Coupled for GPB<'a> {
    fn ic(
        from: &xdevs::component::coupled::ComponentsOutput<Self>,
        to: &mut xdevs::component::coupled::ComponentsInput<Self>,
    ) {
        from.generator.couple(&mut to.buffer).unwrap();
        from.buffer.couple(&mut to.processor).unwrap();
    }
}

fn main() {
    let generator = generator::Generator::new(1.0);
    let buffer = buffer::Buffer::new(8, Some("FIFO buffer"));
    let processor = processor::Processor::new(1.5);
    let gpb = GPB::build(generator, buffer, processor);

    let mut simulator = gpb.to_simulator();
    let config = xdevs::simulation::Config::new(0.0, 10.0, 1.0, None);
    simulator.simulate_rt(&config, xdevs::simulation::std::sleep(&config), |_| {});
}
