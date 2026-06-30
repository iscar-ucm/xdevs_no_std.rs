/// GPT-like example with an optional processor, using the library gpt module.
use xdevs::{
    gpt::{Generator, Processor, Transducer},
    simulation::{AbstractSimulator, Simulable},
};

/// Coupled model with an optional processor, demonstrates
/// `Option<Processor>` as a component field.
///
/// The coupling code routes unconditionally (same type whether present or not).
/// When the processor is `None`, it silently absorbs any routed events in its `delta`.
#[xdevs::coupled]
pub struct GPTOptional {
    generator: Generator,
    processor: Option<Processor>,
    transducer: Transducer,
}

impl xdevs::Component for GPTOptional {
    type Kind = xdevs::CoupledKind;
    type Input = ();
    type Output = ();
}

impl xdevs::Coupled for GPTOptional {
    fn ic(from: &xdevs::ComponentsOutput<Self>, to: &mut xdevs::ComponentsInput<Self>) {
        from.generator.couple(&mut to.processor).unwrap();
        from.generator
            .couple(&mut to.transducer.in_generator)
            .unwrap();
        from.processor
            .couple(&mut to.transducer.in_processor)
            .unwrap();
        from.transducer.couple(&mut to.generator).unwrap();
    }
}

fn run_gpt(some_processor: bool) {
    const PERIOD: f64 = 1.;
    const PROC_TIME: f64 = 1.1;
    const OBS_TIME: f64 = 10.;

    let processor = if some_processor {
        Some(Processor::new(PROC_TIME))
    } else {
        None
    };
    let label = if some_processor { "some" } else { "no" };
    println!("\n--- GPT with {} processor ---", label);
    let gpt = GPTOptional::build(Generator::new(PERIOD), processor, Transducer::new(OBS_TIME));
    let mut simulator = gpt.to_simulator();
    let config = xdevs::simulation::Config::new(0.0, 14.0, 1.0, None);
    simulator.simulate_rt(&config, xdevs::simulation::std::sleep(&config), |_| {});
}

fn main() {
    run_gpt(true);
    run_gpt(false);
}
