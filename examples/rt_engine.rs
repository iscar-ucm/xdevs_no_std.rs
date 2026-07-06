/// This example demonstrates how the rt_engine can be used to simplify the DEVS simulation
/// interaction with other tasks. An array is used for the input to showcase how the input enum
/// would look like for an input array.
use xdevs::{prelude::*, AtomicKind, Config, Duration, Instant, Port};

#[derive(xdevs::Bag)]
pub struct TransparentInput {
    pub in_job: [Port<usize, 1>; 3],
}

#[derive(xdevs::Bag)]
pub struct TransparentOutput {
    pub out_job: Port<usize, 1>,
}

pub struct Transparent {
    next_processor: usize,
    next_value: usize,
    sigma: Duration,
}

#[xdevs::rt_engine(in_channel_size = 3, out_channel_size = 1)]
impl xdevs::Component for Transparent {
    type Kind = AtomicKind;
    type Input = TransparentInput;
    type Output = TransparentOutput;
}

impl xdevs::Atomic for Transparent {
    fn delta_int(&mut self) {
        self.sigma = Duration::MAX; // Passive state (wait for external input)
    }

    fn lambda(&self, output: &mut Self::Output) {
        println!(
            "[Model] forwarding job from processor {}",
            self.next_processor
        );
        output.out_job.add_value(self.next_value).unwrap();
    }

    fn ta(&self) -> Duration {
        self.sigma
    }

    fn delta_ext(&mut self, elapsed: Duration, input: &Self::Input) {
        self.sigma -= elapsed;
        for i in 0..3 {
            if !input.in_job[i].is_empty() {
                println!("[Model] received job from processor {}", i);
                self.next_processor = i;
                self.next_value = *input.in_job[i].get_values().last().unwrap();
                self.sigma = Duration::from_secs(0); // Immediate output
                break;
            }
        }
    }
}

impl Transparent {
    fn new() -> Self {
        Self {
            next_processor: 0,
            next_value: 0,
            sigma: Duration::MAX,
        }
    }
}

async fn sender(sender: TransparentSender) {
    let mut input = 0;
    let mut index = 0;
    loop {
        println!("[Sender] sending value {} to processor {}", input, index);
        sender
            .send(TransparentInputEnum::InJob((index, input)))
            .await
            .unwrap();
        input += 1;
        index = (index + 1) % 3;
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }
}

async fn receiver(mut receiver: TransparentReceiver) {
    loop {
        match receiver.recv().await {
            Ok(TransparentOutputEnum::OutJob(value)) => {
                println!("[Receiver] got value {}", value);
            }
            Err(xdevs::rt_engine::RecvError::Lagged(count)) => {
                println!("[Receiver] lagged by {} messages", count);
            }
            // This error exists only in std
            Err(xdevs::rt_engine::RecvError::Closed) => {
                println!("[Receiver] receive channel closed");
                break;
            }
        }
    }
}

#[tokio::main]
async fn main() {
    let transparent = Transparent::new();
    let mut engine = transparent.into_rt_engine();
    let config = Config::new(Instant::from_secs(0), Instant::from_secs(15), 1, None);

    let send = engine.sender();
    let recv = engine.receiver().unwrap();

    tokio::spawn(sender(send));
    tokio::spawn(receiver(recv));

    engine.simulate_rt(&config).await;
}
