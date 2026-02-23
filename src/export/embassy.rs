use crate::traits::{sealed::Sealed, RtEngineInputChannel, RtEngineOutputChannel};
pub use embassy_sync::{
    channel::{Channel, Sender as eSender},
    pubsub::{
        Error as SubscribeError, PubSubChannel, Publisher as ePublisher, Subscriber as eSubscriber,
        WaitResult,
    },
};
pub use embassy_time::with_deadline;

#[cfg(feature = "embassy-noop")]
pub use embassy_sync::blocking_mutex::raw::NoopRawMutex as Mutex;

#[cfg(feature = "embassy-cs")]
pub use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex as Mutex;

pub enum RecvError {
    Lagged(u64),
}

// Simplified Senders/Subscribers with 'static hardcoded
pub struct Sender<'a, I, const N: usize> {
    sender: eSender<'a, Mutex, I, N>,
}

impl<'a, I, const N: usize> Sender<'a, I, N> {
    pub async fn send(&self, msg: I) {
        self.sender.send(msg).await;
    }
}

pub struct Subscriber<'a, O: Clone, const CAP: usize, const SUBS: usize> {
    subscriber: eSubscriber<'a, Mutex, O, CAP, SUBS, 1>,
}

impl<'a, O: Clone, const CAP: usize, const SUBS: usize> Subscriber<'a, O, CAP, SUBS> {
    pub async fn recv(&mut self) -> Result<O, RecvError> {
        match self.subscriber.next_message().await {
            WaitResult::Message(msg) => Ok(msg),
            WaitResult::Lagged(e) => Err(RecvError::Lagged(e)),
        }
    }
}

pub struct InputChannel<'a, I, const N: usize> {
    channel: &'a Channel<Mutex, I, N>,
}

impl<'a, I, const N: usize> InputChannel<'a, I, N> {
    pub fn new(channel: &'a Channel<Mutex, I, N>) -> Self {
        Self { channel }
    }
}
unsafe impl<'a, I, const N: usize> RtEngineInputChannel for InputChannel<'a, I, N> {
    type InputEnum = I;
    type Sender = Sender<'a, I, N>;

    fn sender(&self) -> Self::Sender {
        Sender {
            sender: self.channel.sender(),
        }
    }

    async fn recv(&self) -> Self::InputEnum {
        self.channel.receive().await
    }
}

impl<'a, I, const N: usize> Sealed for InputChannel<'a, I, N> {}

pub struct OutputChannel<'a, O: Clone, const CAP: usize, const SUBS: usize> {
    channel: &'a PubSubChannel<Mutex, O, CAP, SUBS, 1>,
    publisher: ePublisher<'a, Mutex, O, CAP, SUBS, 1>,
}

impl<'a, O: Clone, const CAP: usize, const SUBS: usize> OutputChannel<'a, O, CAP, SUBS> {
    pub fn new(channel: &'a PubSubChannel<Mutex, O, CAP, SUBS, 1>) -> Self {
        Self {
            channel,
            // SAFETY: This is the only publisher that will be created for this channel
            publisher: channel.publisher().unwrap(),
        }
    }
}
unsafe impl<'a, O: Clone, const CAP: usize, const SUBS: usize> RtEngineOutputChannel
    for OutputChannel<'a, O, CAP, SUBS>
{
    type OutputEnum = O;
    type Subscriber = Subscriber<'a, Self::OutputEnum, CAP, SUBS>;

    fn subscriber(&self) -> Result<Self::Subscriber, SubscribeError> {
        match self.channel.subscriber() {
            Ok(subscriber) => Ok(Subscriber { subscriber }),
            Err(e) => Err(e),
        }
    }

    fn publish(&self, output: Self::OutputEnum) {
        self.publisher.publish_immediate(output);
    }
}
impl<'a, O: Clone, const CAP: usize, const SUBS: usize> Sealed for OutputChannel<'a, O, CAP, SUBS> {}
