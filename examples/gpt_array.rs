/// Demonstrates arrays of components using GPT models from the library module.
///
/// Topology: one Generator feeds [Processor; N], each with its own Transducer.
/// The last Transducer sends the stop signal back to the Generator.
use xdevs::{
    gpt::{Generator, Processor, Transducer},
    prelude::*,
    ComponentsInput, ComponentsOutput, Config, CoupledKind, Duration, Instant,
};

/// Coupled model with an array of processor-transducer pairs.
#[xdevs::coupled]
struct GPTArray<const N: usize> {
    generator: Generator,
    processors: [Processor; N],
    transducers: [Transducer; N],
}

impl<const N: usize> xdevs::Component for GPTArray<N> {
    type Kind = CoupledKind;
    type Input = ();
    type Output = ();
}

impl<const N: usize> xdevs::Coupled for GPTArray<N> {
    fn ic(from: &ComponentsOutput<Self>, to: &mut ComponentsInput<Self>) {
        for i in 0..N {
            from.generator.couple(&mut to.processors[i]).unwrap();
            from.processors[i]
                .couple(&mut to.transducers[i].in_processor)
                .unwrap();
            from.generator
                .couple(&mut to.transducers[i].in_generator)
                .unwrap();
        }
        if N > 0 {
            from.transducers[N - 1].couple(&mut to.generator).unwrap();
        }
    }
}

fn main() {
    const N: usize = 3;
    let period = Duration::from_secs(1);
    let proc_time = Duration::from_millis(1100);
    let obs_time = Duration::from_secs(10);

    let generator = Generator::new(period);
    let processors = core::array::from_fn(|_| Processor::new(proc_time));
    let transducers = core::array::from_fn(|_| Transducer::new(obs_time));

    let model = GPTArray::<N>::build(generator, processors, transducers);

    let mut simulator = model.to_simulator();
    let config = Config::new(Instant::from_secs(0), Instant::from_secs(14), 1, None);
    simulator.simulate_vt(&config);
}
