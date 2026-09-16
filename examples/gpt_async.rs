/// A simple DEVS GPT model using the library gpt module with async simulation.
use xdevs::{
    gpt::{Generator, Processor, Transducer, GPT},
    prelude::*,
    simulation::SleepAsync,
    Config, Duration,
};

#[tokio::main]
async fn main() {
    let period = Duration::from_secs(1);
    let proc_time = Duration::from_millis(1100);
    let obs_time = Duration::from_secs(10);

    let generator = Generator::new(period);
    let processor = Processor::new(proc_time);
    let transducer = Transducer::new(obs_time);

    let gpt = GPT::build(generator, processor, transducer);

    let mut simulator = gpt.to_simulator();
    let config = Config::new(Duration::from_secs(14), 1, None);
    let input_handler = SleepAsync::new();

    simulator.simulate_rt(&config, input_handler, |_| {}).await;
}
