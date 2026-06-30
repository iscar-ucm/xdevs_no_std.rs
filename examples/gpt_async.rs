/// A simple DEVS GPT model using the library gpt module with async simulation.
use xdevs::{
    gpt::{Generator, Processor, Transducer, GPT},
    simulation::std::SleepAsync,
    AbstractSimulator, Config, Simulable,
};

#[tokio::main]
async fn main() {
    const PERIOD: f64 = 1.;
    const PROC_TIME: f64 = 1.1;
    const OBS_TIME: f64 = 10.;

    let generator = Generator::new(PERIOD);
    let processor = Processor::new(PROC_TIME);
    let transducer = Transducer::new(OBS_TIME);

    let gpt = GPT::build(generator, processor, transducer);

    let mut simulator = gpt.to_simulator();
    let config = Config::new(0.0, 14.0, 1.0, None);
    let input_handler = SleepAsync::new();

    simulator
        .simulate_rt_async(&config, input_handler, |_| {})
        .await;
}
