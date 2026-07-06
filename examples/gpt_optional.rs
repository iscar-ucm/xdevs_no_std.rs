/// GPT-like example with an optional processor, using the library gpt module.
use xdevs::{
    gpt::{Generator, Processor, Transducer},
    prelude::*,
    ComponentsInput, ComponentsOutput, CoupledKind, Duration, Instant,
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
    type Kind = CoupledKind;
    type Input = ();
    type Output = ();
}

impl xdevs::Coupled for GPTOptional {
    fn ic(from: &ComponentsOutput<Self>, to: &mut ComponentsInput<Self>) {
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
    let period = Duration::from_secs(1);
    let proc_time = Duration::from_millis(1100);
    let obs_time = Duration::from_secs(10);

    let processor = if some_processor {
        Some(Processor::new(proc_time))
    } else {
        None
    };
    let label = if some_processor { "some" } else { "no" };
    println!("\n--- GPT with {} processor ---", label);
    let gpt = GPTOptional::build(Generator::new(period), processor, Transducer::new(obs_time));
    let mut simulator = gpt.to_simulator();
    let config = xdevs::Config::new(Instant::from_secs(0), Instant::from_secs(14), 1, None);
    simulator.simulate_vt(&config);
}

fn main() {
    run_gpt(true);
    run_gpt(false);
}
