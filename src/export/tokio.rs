use crate::{
    rt_engine::{sealed::Sealed, RtEngineInputChannel, RtEngineOutputChannel},
    Duration,
};
use core::future::Future;

pub use tokio::sync::broadcast::error::RecvError;
pub type SubscribeError = core::convert::Infallible;
use tokio::sync::mpsc::error::SendError;

#[repr(transparent)]
pub struct Sender<I> {
    sender: tokio::sync::mpsc::Sender<I>,
}
impl<I> Sender<I> {
    pub async fn send(&self, msg: I) -> Result<(), SendError<I>> {
        self.sender.send(msg).await
    }
}

#[repr(transparent)]
pub struct Receiver<O> {
    receiver: tokio::sync::broadcast::Receiver<O>,
}
impl<O: Clone> Receiver<O> {
    pub async fn recv(&mut self) -> Result<O, RecvError> {
        self.receiver.recv().await
    }
}

pub struct InputChannel<I, const N: usize> {
    sender: tokio::sync::mpsc::Sender<I>,
    receiver: tokio::sync::mpsc::Receiver<I>,
}

impl<I, const N: usize> InputChannel<I, N> {
    pub fn new() -> Self {
        let (sender, receiver) = tokio::sync::mpsc::channel(N);
        Self { sender, receiver }
    }
}

impl<I, const N: usize> Default for InputChannel<I, N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<I: Send, const N: usize> RtEngineInputChannel for InputChannel<I, N> {
    type Input = I;
    type Sender = Sender<I>;

    fn sender(&self) -> Self::Sender {
        Sender {
            sender: self.sender.clone(),
        }
    }

    async fn recv(&mut self) -> Self::Input {
        // There will always be a sender, so this should never fail
        self.receiver.recv().await.unwrap()
    }
}

impl<I: Send, const N: usize> Sealed for InputChannel<I, N> {}

pub struct OutputChannel<O: Clone, const N: usize> {
    sender: tokio::sync::broadcast::Sender<O>,
    receiver: tokio::sync::broadcast::Receiver<O>,
}

impl<O: Clone, const N: usize> OutputChannel<O, N> {
    pub fn new() -> Self {
        let (sender, receiver) = tokio::sync::broadcast::channel(N);
        Self { sender, receiver }
    }
}

impl<O: Clone, const N: usize> Default for OutputChannel<O, N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<O: Clone, const N: usize> RtEngineOutputChannel for OutputChannel<O, N> {
    type Output = O;
    type Receiver = Receiver<O>;

    fn receiver(&self) -> Result<Self::Receiver, SubscribeError> {
        Ok(Receiver {
            receiver: self.receiver.resubscribe(),
        })
    }
    fn publish(&self, msg: Self::Output) {
        // There will always be a receiver, so this should never fail
        let _ = self.sender.send(msg);
    }
}

impl<O: Clone, const N: usize> Sealed for OutputChannel<O, N> {}

pub struct Clock;
impl crate::clock::Clock for Clock {
    type Instant = tokio::time::Instant;

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
        let wall_offset = std::time::Duration::from_micros(t_until.as_micros().div_ceil(mult));
        let deadline = *t0 + wall_offset;
        let _ = tokio::time::timeout_at(deadline, input_handler).await;
        let now = Self::Instant::now();
        let elapsed =
            u64::try_from(now.saturating_duration_since(*t0).as_micros()).unwrap_or(u64::MAX);
        Duration::from_micros(elapsed.saturating_mul(mult))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::clock::Clock as _;

    #[tokio::test]
    async fn wait_until_scales_wall_time_by_multiplier() {
        let wall_t0 = std::time::Instant::now();
        let t0 = Clock::now();
        let t = Clock::wait_until(&t0, Duration::from_secs(1), core::future::pending(), 10).await;
        let elapsed = wall_t0.elapsed();
        assert!(
            elapsed >= std::time::Duration::from_millis(100),
            "{elapsed:?}"
        );
        assert!(
            elapsed <= std::time::Duration::from_millis(500),
            "{elapsed:?}"
        );
        assert!(t >= Duration::from_secs(1), "{t:?}");
        assert!(t <= Duration::from_secs(5), "{t:?}");
    }

    #[tokio::test]
    async fn wait_until_returns_early_when_input_is_ready() {
        let wall_t0 = std::time::Instant::now();
        let t0 = Clock::now();
        let t = Clock::wait_until(&t0, Duration::from_secs(1), core::future::ready(()), 1).await;
        let elapsed = wall_t0.elapsed();
        assert!(
            elapsed < std::time::Duration::from_millis(10),
            "{elapsed:?}"
        );
        assert!(t < Duration::from_millis(10), "{t:?}");
    }

    #[tokio::test]
    async fn wait_until_waits_real_time_when_unscaled() {
        let fifty_ms = Duration::from_millis(50);
        let wall_t0 = std::time::Instant::now();
        let t0 = Clock::now();
        let t = Clock::wait_until(&t0, fifty_ms, core::future::pending(), 1).await;
        let elapsed = wall_t0.elapsed();
        assert!(
            elapsed >= std::time::Duration::from_millis(50),
            "{elapsed:?}"
        );
        assert!(
            elapsed <= std::time::Duration::from_millis(250),
            "{elapsed:?}"
        );
        assert!(t >= fifty_ms, "{t:?}");
        assert!(t <= Duration::from_millis(250), "{t:?}");
    }
}
