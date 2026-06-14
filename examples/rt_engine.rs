/// This example demonstrates how the rt_engine can be used to simplify the DEVS simulation
/// interaction with other tasks. An array is used for the input to showcase how the input enum
/// would look like for an input array.

#[derive(xdevs::Bag, xdevs::BagMux)]
pub struct TransparentInput {
    pub in_job: [xdevs::Port<usize, 1>; 3],
}

#[derive(xdevs::Bag, xdevs::BagMux)]
pub struct TransparentOutput {
    pub out_job: xdevs::Port<usize, 1>,
}

pub struct TransparentAtomic {
    next_processor: usize,
    next_value: usize,
    sigma: f64,
}

impl xdevs::Component for TransparentAtomic {
    type Kind = xdevs::AtomicKind;
    type Input = TransparentInput;
    type Output = TransparentOutput;
}

impl xdevs::Atomic for TransparentAtomic {
    fn delta_int(&mut self) {
        self.sigma = f64::INFINITY; // Passive state (wait for external input)
    }

    fn lambda(&self, output: &mut Self::Output) {
        println!(
            "[Model] forwarding job from processor {}",
            self.next_processor
        );
        output.out_job.add_value(self.next_value).unwrap();
    }

    fn ta(&self) -> f64 {
        self.sigma
    }

    fn delta_ext(&mut self, elapsed: f64, input: &Self::Input) {
        self.sigma -= elapsed;
        for i in 0..3 {
            if !input.in_job[i].is_empty() {
                println!("[Model] received job from processor {}", i);
                self.next_processor = i;
                self.next_value = *input.in_job[i].get_values().last().unwrap();
                self.sigma = 0.0; // Immediate output
                break;
            }
        }
    }
}

impl TransparentAtomic {
    fn new() -> Self {
        Self {
            next_processor: 0,
            next_value: 0,
            sigma: f64::INFINITY,
        }
    }
}

#[xdevs::coupled(rt_engine(in_channel_size = 3, out_channel_size = 1))]
pub struct Transparent {
    atomic: TransparentAtomic,
}

impl xdevs::Component for Transparent {
    type Kind = xdevs::CoupledKind;
    type Input = TransparentInput;
    type Output = TransparentOutput;
}

impl xdevs::Coupled for Transparent {
    fn eic(from: &Self::Input, to: &mut Self::ComponentsInput) {
        for (external, internal) in from.in_job.iter().zip(to.atomic.in_job.iter_mut()) {
            external.couple(internal).unwrap();
        }
    }
    fn eoc(from: &Self::ComponentsOutput, to: &mut Self::Output) {
        from.atomic.out_job.couple(&mut to.out_job).unwrap();
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
    let transparent = TransparentAtomic::new();
    let transparent = Transparent::build(transparent);
    let mut engine = transparent.into_rt_engine();
    let config = xdevs::simulator::Config::new(0.0, 15.0, 1.0, None);

    let send = engine.sender();
    let recv = engine.receiver().unwrap();

    tokio::spawn(sender(send));
    tokio::spawn(receiver(recv));

    engine.simulate_rt_async(&config).await;
}
