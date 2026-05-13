use crate::traits::{sealed, Bag};
use core::future::Future;

/// Input port interface for DEVS models that can be simulated in real-time using the `RtEngine`.
///
/// # Safety
///
/// This trait must be implemented via macros. Do not implement it manually.
pub unsafe trait InjectInput: Bag {
    /// Input channel for the rt_engine macro.
    type InputChannel;

    /// Maps the input enum to the corresponding input port
    fn map_input(&mut self, in_channel: &mut Self::InputChannel)
        -> impl Future<Output = ()> + Send;
}

/// Output port interface for DEVS models that can be simulated in real-time using the `RtEngine`.
///
/// # Safety
///
/// This trait must be implemented via macros. Do not implement it manually.
pub unsafe trait EjectOutput: Bag {
    /// Output channel for the rt_engine macro.
    type OutputChannel;

    /// Maps the output enum to the corresponding output port
    fn map_output(&self, out_channel: &Self::OutputChannel);
}

/// Input channel for the rt_engine macro.
pub trait RtEngineInputChannel: sealed::Sealed {
    /// Enum representing the input ports of the model. Each variant corresponds to an input port.
    type Input;
    /// Type of the sender used to send input events to the model.
    type Sender;

    /// Returns a sender to the channel. The sender can be used to send input events to the model.
    fn sender(&self) -> Self::Sender;

    fn recv(&mut self) -> impl Future<Output = Self::Input> + Send;
}

/// Output channel for the rt_engine macro.
pub trait RtEngineOutputChannel: sealed::Sealed {
    /// Enum representing the output ports of the model. Each variant corresponds to an output port.
    type Output;

    /// Type of the receiver used to receive output events from the model.
    type Receiver;

    /// Returns a subscriber to the channel. The subscriber can be used to receive output events from the model.
    fn receiver(&self) -> Result<Self::Receiver, crate::rt_engine::SubscribeError>;

    /// Publishes output events from the model to the channel, mapping the model's output ports to the channel's output events.
    fn publish(&self, output: Self::Output);
}
