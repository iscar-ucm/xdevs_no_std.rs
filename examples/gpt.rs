/// A simple DEVS GPT model using the library gpt module.
use xdevs::{
    gpt::{Generator, Processor, Transducer, EF, EFP},
    prelude::*,
    Config, Duration, Instant,
};

fn main() {
    let period = Duration::from_secs(1);
    let proc_time = Duration::from_millis(1100);
    let obs_time = Duration::from_secs(10);

    let generator = Generator::new(period);
    let processor = Processor::new(proc_time);
    let transducer = Transducer::new(obs_time);

    let ef = EF::build(generator, transducer);
    let efp = EFP::build(ef, processor);

    let mut simulator = efp.to_simulator();
    let config = Config::new(Instant::from_secs(0), Instant::from_secs(14), 1, None);
    simulator.simulate_vt(&config);
}
