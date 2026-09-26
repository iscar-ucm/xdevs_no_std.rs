use crate::{
    rt_engine::{sealed::Sealed, RtEngineInputChannel, RtEngineOutputChannel},
    Duration,
};
use core::{convert::Infallible, future::Future};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex as Mutex;

pub type Channel<T, const N: usize> = embassy_sync::channel::Channel<Mutex, T, N>;
pub type PubSubChannel<T, const CAP: usize, const SUBS: usize> =
    embassy_sync::pubsub::PubSubChannel<Mutex, T, CAP, SUBS, 1>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecvError {
    Lagged(u64),
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum SubscribeError {
    MaximumSubscribersReached,
}

// Simplified Senders/Subscribers
#[repr(transparent)]
pub struct Sender<'a, I, const N: usize> {
    sender: embassy_sync::channel::Sender<'a, Mutex, I, N>,
}

impl<'a, I, const N: usize> Sender<'a, I, N> {
    pub async fn send(&self, msg: I) -> Result<(), Infallible> {
        self.sender.send(msg).await;
        Ok(())
    }
}

#[repr(transparent)]
pub struct Receiver<'a, O: Clone, const CAP: usize, const SUBS: usize> {
    subscriber: embassy_sync::pubsub::Subscriber<'a, Mutex, O, CAP, SUBS, 1>,
}

impl<'a, O: Clone, const CAP: usize, const SUBS: usize> Receiver<'a, O, CAP, SUBS> {
    pub async fn recv(&mut self) -> Result<O, RecvError> {
        use embassy_sync::pubsub::WaitResult;
        match self.subscriber.next_message().await {
            WaitResult::Message(msg) => Ok(msg),
            WaitResult::Lagged(e) => Err(RecvError::Lagged(e)),
        }
    }
}

#[repr(transparent)]
pub struct InputChannel<'a, I, const N: usize> {
    channel: &'a Channel<I, N>,
}

impl<'a, I, const N: usize> InputChannel<'a, I, N> {
    pub fn new(channel: &'a Channel<I, N>) -> Self {
        Self { channel }
    }
}
impl<'a, I: Send, const N: usize> RtEngineInputChannel for InputChannel<'a, I, N> {
    type Input = I;
    type Sender = Sender<'a, I, N>;

    fn sender(&self) -> Self::Sender {
        Sender {
            sender: self.channel.sender(),
        }
    }

    async fn recv(&mut self) -> Self::Input {
        self.channel.receive().await
    }
}

impl<'a, I, const N: usize> Sealed for InputChannel<'a, I, N> {}

pub struct OutputChannel<'a, O: Clone, const CAP: usize, const SUBS: usize> {
    channel: &'a PubSubChannel<O, CAP, SUBS>,
    publisher: embassy_sync::pubsub::Publisher<'a, Mutex, O, CAP, SUBS, 1>,
}

impl<'a, O: Clone, const CAP: usize, const SUBS: usize> OutputChannel<'a, O, CAP, SUBS> {
    pub fn new(channel: &'a PubSubChannel<O, CAP, SUBS>) -> Self {
        Self {
            channel,
            publisher: channel.publisher().unwrap(),
        }
    }
}
impl<'a, O: Clone, const CAP: usize, const SUBS: usize> RtEngineOutputChannel
    for OutputChannel<'a, O, CAP, SUBS>
{
    type Output = O;
    type Receiver = Receiver<'a, Self::Output, CAP, SUBS>;

    fn receiver(&self) -> Result<Self::Receiver, SubscribeError> {
        match self.channel.subscriber() {
            Ok(subscriber) => Ok(Receiver { subscriber }),
            Err(embassy_sync::pubsub::Error::MaximumSubscribersReached) => {
                Err(SubscribeError::MaximumSubscribersReached)
            }
            _ => unreachable!(),
        }
    }

    fn publish(&self, output: Self::Output) {
        self.publisher.publish_immediate(output);
    }
}
impl<'a, O: Clone, const CAP: usize, const SUBS: usize> Sealed for OutputChannel<'a, O, CAP, SUBS> {}

pub struct Clock;
impl crate::clock::Clock for Clock {
    type Instant = embassy_time::Instant;

    #[inline(always)]
    fn now() -> Self::Instant {
        Self::Instant::now()
    }

    async fn wait_until(
        t0: &Self::Instant,
        t_until: Duration,
        input_handler: impl Future<Output = ()>,
        mult: u64,
    ) -> Duration {
        let wall_offset = embassy_time::Duration::from_micros(t_until.as_micros().div_ceil(mult));
        let deadline = t0.saturating_add(wall_offset);
        let _ = embassy_time::with_deadline(deadline, input_handler).await;
        let now = Self::Instant::now();

        Duration::from_micros(
            now.saturating_duration_since(*t0)
                .as_micros()
                .saturating_mul(mult),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::clock::Clock as _;

    #[tokio::test]
    async fn wait_until_scales_wall_time_by_multiplier() {
        let wall_t0 = embassy_time::Instant::now();
        let t0 = Clock::now();
        let t = Clock::wait_until(&t0, Duration::from_secs(1), core::future::pending(), 10).await;
        let elapsed = embassy_time::Instant::now().saturating_duration_since(wall_t0);
        assert!(
            elapsed >= embassy_time::Duration::from_millis(100),
            "{elapsed:?}"
        );
        assert!(
            elapsed <= embassy_time::Duration::from_millis(500),
            "{elapsed:?}"
        );
        assert!(t >= Duration::from_secs(1), "{t:?}");
        assert!(t <= Duration::from_secs(5), "{t:?}");
    }

    #[tokio::test]
    async fn wait_until_returns_early_when_input_is_ready() {
        let wall_t0 = embassy_time::Instant::now();
        let t0 = Clock::now();
        let t = Clock::wait_until(&t0, Duration::from_secs(1), core::future::ready(()), 1).await;
        let elapsed = embassy_time::Instant::now().saturating_duration_since(wall_t0);
        assert!(
            elapsed < embassy_time::Duration::from_millis(10),
            "{elapsed:?}"
        );
        assert!(t < Duration::from_millis(10), "{t:?}");
    }

    #[tokio::test]
    async fn wait_until_waits_real_time_when_unscaled() {
        let fifty_ms = Duration::from_millis(50);
        let wall_t0 = embassy_time::Instant::now();
        let t0 = Clock::now();
        let t = Clock::wait_until(&t0, fifty_ms, core::future::pending(), 1).await;
        let elapsed = embassy_time::Instant::now().saturating_duration_since(wall_t0);
        assert!(
            elapsed >= embassy_time::Duration::from_millis(50),
            "{elapsed:?}"
        );
        assert!(
            elapsed <= embassy_time::Duration::from_millis(250),
            "{elapsed:?}"
        );
        assert!(t >= fifty_ms, "{t:?}");
        assert!(t <= Duration::from_millis(250), "{t:?}");
    }
}
