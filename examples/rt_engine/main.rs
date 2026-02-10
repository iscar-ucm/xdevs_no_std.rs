use xdevs::port::Port;

#[xdevs::rt_engine(in_size = 3, out_size = 1, max_out_subs = 1)]
#[xdevs::atomic]
pub struct Transparent {
    #[input]
    pub in_job: [Port<usize, 1>; 3],
    #[output]
    pub out_job: Port<usize, 1>,
    #[state]
    next_processor: usize,
    next_value: usize,
    sigma: f64,
}

impl xdevs::Atomic for Transparent {
    fn delta_int(state: &mut Self::State) {
        state.sigma = f64::INFINITY; // Immediate output
    }

    fn lambda(state: &Self::State, output: &mut Self::Output) {
        println!("[T] forwarding job from processor {}", state.next_processor);
        output.out_job.add_value(state.next_value).unwrap();
    }

    fn ta(state: &Self::State) -> f64 {
        state.sigma
    }

    fn delta_ext(state: &mut Self::State, elapsed: f64, input: &Self::Input) {
        state.sigma -= elapsed;
        for i in 0..3 {
            if !input.in_job[i].is_empty() {
                println!("[T] received job from processor {}", i);
                state.next_processor = i;
                state.next_value = *input.in_job[i].get_values().last().unwrap();
                state.sigma = 0.0; // Immediate output
            }
        }
    }
}

impl Transparent {
    fn new() -> Self {
        Self::build(0, 0, f64::INFINITY)
    }
}

async fn sender(sender: xdevs::Sender<'static, TransparentInputEnum, 3>) {
    let mut input = 0;
    let mut index = 0;
    loop {
        sender
            .send(TransparentInputEnum::InJob((index, input)))
            .await;
        input += 1;
        index = (index + 1) % 3;
        //index += 1;
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }
}

async fn receiver(mut receiver: xdevs::Subscriber<'static, TransparentOutputEnum, 1, 1>) {
    loop {
        match receiver.recv().await {
            Ok(TransparentOutputEnum::OutJob(value)) => {
                println!("[Receiver] got value {}", value);
            }
            Err(xdevs::PubSubError::Lagged(u64)) => {
                println!("[Receiver] lagged by {} messages", u64);
            }
        }
    }
}

#[tokio::main]
async fn main() {
    let transparent = Transparent::new();
    let engine = transparent.into_rt_engine();
    let config = xdevs::simulator::Config::new(0.0, 15.0, 1.0, None);

    let send = engine.sender();
    let recv = engine.subscriber().unwrap();

    tokio::spawn(sender(send));
    tokio::spawn(receiver(recv));

    engine.simulate_rt_async(&config).await;
}
