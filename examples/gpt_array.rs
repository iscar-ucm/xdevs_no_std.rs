/// Demonstrates arrays of components using GPT models from the library module.
///
/// Topology: one Generator feeds [Processor; N], each with its own Transducer.
/// The last Transducer sends the stop signal back to the Generator.
use xdevs::{
    gpt::{Generator, Processor, Transducer},
    simulation::{AbstractSimulator, Config, Simulable},
};

/// Coupled model with an array of processor-transducer pairs.
#[xdevs::to_component]
struct GPTArray<const N: usize> {
    generator: Generator,
    processors: [Processor; N],
    transducers: [Transducer; N],
}

impl<const N: usize> xdevs::Component for GPTArray<N> {
    type Kind = xdevs::CoupledKind;
    type Input = ();
    type Output = ();
}

impl<const N: usize> xdevs::Coupled for GPTArray<N> {
    fn ic(
        from: &xdevs::component::coupled::ComponentsOutput<Self>,
        to: &mut xdevs::component::coupled::ComponentsInput<Self>,
    ) {
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
    const PERIOD: f64 = 1.;
    const PROC_TIME: f64 = 1.1;
    const OBS_TIME: f64 = 10.;

    let generator = Generator::new(PERIOD);
    let processors = core::array::from_fn(|_| Processor::new(PROC_TIME));
    let transducers = core::array::from_fn(|_| Transducer::new(OBS_TIME));

    let model = GPTArray::<N>::build(generator, processors, transducers);

    let mut simulator = model.to_simulator();
    let config = Config::new(0.0, 14.0, 1.0, None);
    simulator.simulate_vt(&config);
}
