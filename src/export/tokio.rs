use crate::traits::{sealed::Sealed, RtEngineInputChannel, RtEngineOutputChannel};

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
