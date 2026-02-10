pub use {
    embassy_sync::channel::{Channel, Sender as eSender},
    embassy_sync::pubsub::{
        Error as SubscribeError, PubSubChannel, Subscriber as eSubscriber, WaitResult,
    },
    embassy_time::with_deadline,
};

#[cfg(feature = "embassy-noop")]
pub use embassy_sync::blocking_mutex::raw::NoopRawMutex as Mutex;

#[cfg(feature = "embassy-cs")]
pub use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex as Mutex;

pub enum PubSubError {
    Lagged(u64),
}

pub struct Sender<'a, I, const N: usize> {
    channel: eSender<'a, Mutex, I, N>,
}

impl<'a, I, const N: usize> Sender<'a, I, N> {
    pub async fn send(&self, msg: I) {
        self.channel.send(msg).await;
    }
}

pub struct Subscriber<'a, O: Clone, const CAP: usize, const SUBS: usize> {
    subscriber: eSubscriber<'a, Mutex, O, CAP, SUBS, 1>,
}

impl<'a, O: Clone, const CAP: usize, const SUBS: usize> Subscriber<'a, O, CAP, SUBS> {
    pub async fn recv(&mut self) -> Result<O, PubSubError> {
        match self.subscriber.next_message().await {
            WaitResult::Message(msg) => Ok(msg),
            WaitResult::Lagged(e) => Err(PubSubError::Lagged(e)),
        }
    }
}

pub struct RtEngine<
    'a,
    M: crate::traits::AbstractSimulator,
    I,
    O: Clone,
    IH: crate::traits::AsyncInput,
    PO: FnMut(&M::Output),
    const N: usize,
    const CAP: usize,
    const SUBS: usize,
> {
    simulator: crate::Simulator<M>,
    input_channel: &'a Channel<Mutex, I, N>,
    input_handler: IH,
    output_channel: &'a PubSubChannel<Mutex, O, CAP, SUBS, 1>,
    propagate_output: PO,
}

impl<
        'a,
        M: crate::traits::AbstractSimulator,
        I,
        O: Clone,
        IH: crate::traits::AsyncInput<Input = M::Input>,
        PO: FnMut(&M::Output),
        const N: usize,
        const CAP: usize,
        const SUBS: usize,
    > RtEngine<'a, M, I, O, IH, PO, N, CAP, SUBS>
{
    pub fn new(
        model: M,
        input_channel: &'a Channel<Mutex, I, N>,
        input_handler: IH,
        output_channel: &'a PubSubChannel<Mutex, O, CAP, SUBS, 1>,
        propagate_output: PO,
    ) -> Self {
        Self {
            simulator: crate::Simulator::new(model),
            input_channel,
            input_handler,
            output_channel,
            propagate_output,
        }
    }

    pub async fn simulate_rt_async(mut self, config: &crate::Config) {
        self.simulator
            .simulate_rt_async(config, self.input_handler, self.propagate_output)
            .await;
    }

    pub fn sender(&self) -> Sender<'a, I, N> {
        Sender {
            channel: self.input_channel.sender(),
        }
    }

    pub fn subscriber(&self) -> Result<Subscriber<'a, O, CAP, SUBS>, SubscribeError> {
        match self.output_channel.subscriber() {
            Ok(subscriber) => Ok(Subscriber { subscriber }),
            Err(e) => Err(e),
        }
    }
}
