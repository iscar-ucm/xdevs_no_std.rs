/// A simple DEVS GPT model using the library gpt module.
use xdevs::{
    gpt::{Generator, Processor, Transducer, EF, EFP},
    simulation::{AbstractSimulator, Simulable},
};

fn main() {
    let period = 1.;
    let proc_time = 1.1;
    let obs_time = 10.;

    let generator = Generator::new(period);
    let processor = Processor::new(proc_time);
    let transducer = Transducer::new(obs_time);

    let ef = EF::build(generator, transducer);
    let efp = EFP::build(ef, processor);

    let mut simulator = efp.to_simulator();
    let config = xdevs::simulation::Config::new(0.0, 14.0, 1.0, None);
    simulator.simulate_vt(&config);
}
