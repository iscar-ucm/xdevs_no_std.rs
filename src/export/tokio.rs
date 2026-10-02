use crate::{
    rt_engine::{sealed::Sealed, RtEngineInputChannel, RtEngineOutputChannel},
    Duration,
};
use core::future::Future;

pub use tokio::sync::broadcast::error::RecvError;
pub type SubscribeError = core::convert::Infallible;
use tokio::{
    sync::mpsc::error::SendError,
    time::{self, Instant},
};

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

pub struct Clock {
    t0: Instant,
}

impl Clock {
    #[inline(always)]
    pub fn new() -> Self {
        Self {
            // The value here doesn't matter, because start() will overwrite it with the current time when the simulation starts.
            t0: Instant::now(),
        }
    }
}

impl Default for Clock {
    fn default() -> Self {
        Self::new()
    }
}

impl crate::simulation::Clock for Clock {
    #[inline(always)]
    fn start(&mut self) {
        self.t0 = Instant::now();
    }

    async fn wait_until(
        &self,
        t_until: Duration,
        input_handler: impl Future<Output = ()>,
    ) -> Duration {
        let deadline = self.t0 + t_until.into();
        let _ = time::timeout_at(deadline, input_handler).await;
        let now = Instant::now();
        let elapsed =
            u64::try_from(now.saturating_duration_since(self.t0).as_micros()).unwrap_or(u64::MAX);
        Duration::from_micros(elapsed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::simulation::Clock as _;

    #[tokio::test]
    async fn wait_until_returns_early_when_input_is_ready() {
        let wall_t0 = std::time::Instant::now();
        let mut clock = Clock::new();
        clock.start();
        let t = clock
            .wait_until(Duration::from_secs(1), core::future::ready(()))
            .await;
        let elapsed = wall_t0.elapsed();
        assert!(
            elapsed < std::time::Duration::from_millis(10),
            "{elapsed:?}"
        );
        assert!(t < Duration::from_millis(10), "{t:?}");
    }

    #[tokio::test]
    async fn wait_until_waits_requested_wall_time() {
        let fifty_ms = Duration::from_millis(50);
        let wall_t0 = std::time::Instant::now();
        let mut clock = Clock::new();
        clock.start();
        let t = clock.wait_until(fifty_ms, core::future::pending()).await;
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
