use crate::traits::{sealed::Sealed, RtEngineInputChannel, RtEngineOutputChannel};

pub use tokio::sync::broadcast::error::RecvError;
pub type SubscribeError = core::convert::Infallible;
use tokio::sync::mpsc::error::SendError;

#[repr(transparent)]
pub struct Sender<T> {
    sender: tokio::sync::mpsc::Sender<T>,
}
impl<T> Sender<T> {
    pub async fn send(&self, msg: T) -> Result<(), SendError<T>> {
        self.sender.send(msg).await
    }
}

#[repr(transparent)]
pub struct Receiver<T> {
    receiver: tokio::sync::broadcast::Receiver<T>,
}
impl<T: Clone> Receiver<T> {
    pub async fn recv(&mut self) -> Result<T, RecvError> {
        self.receiver.recv().await
    }
}

pub struct InputChannel<T, const N: usize> {
    sender: tokio::sync::mpsc::Sender<T>,
    receiver: tokio::sync::mpsc::Receiver<T>,
}
impl<T, const N: usize> InputChannel<T, N> {
    pub fn new() -> Self {
        let (sender, receiver) = tokio::sync::mpsc::channel(N);
        Self { sender, receiver }
    }
}
unsafe impl<T, const N: usize> RtEngineInputChannel for InputChannel<T, N> {
    type Input = T;
    type Sender = Sender<T>;

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
impl<T, const N: usize> Sealed for InputChannel<T, N> {}

pub struct OutputChannel<T: Clone, const N: usize> {
    sender: tokio::sync::broadcast::Sender<T>,
    receiver: tokio::sync::broadcast::Receiver<T>,
}
impl<T: Clone, const N: usize> OutputChannel<T, N> {
    pub fn new() -> Self {
        let (sender, receiver) = tokio::sync::broadcast::channel(N);
        Self { sender, receiver }
    }
}
unsafe impl<T: Clone, const N: usize> RtEngineOutputChannel for OutputChannel<T, N> {
    type Output = T;
    type Receiver = Receiver<T>;

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
impl<T: Clone, const N: usize> Sealed for OutputChannel<T, N> {}
