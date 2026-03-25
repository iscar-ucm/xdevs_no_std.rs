use crate::{simulator::Config, traits::sealed::Sealed};
use core::future::Future;

/// Trait that defines the methods that a DEVS event bag set must implement.
///
/// # Safety
///
/// This trait must be implemented via macros. Do not implement it manually.
pub unsafe trait Bag {
    /// Returns `true` if the ports are empty.
    fn is_empty(&self) -> bool;

    /// Clears the ports, removing all values.
    fn clear(&mut self);
}

/// Trait that defines the type inside of a Bag for rt_engine enums.
///
/// # Safety
///
/// This trait is implemented internally. Do not implement it manually.
pub unsafe trait AsPort: Bag + Sealed {
    /// The type of the values contained in the bag.
    type Item;
}

/// Trait that defines an enum that can be used for the input or output channel for the rt_engine macro.
///
/// # Safety
///
/// This trait must be implemented via macros. Do not implement it manually.
pub unsafe trait BagMux: Bag {
    /// The enum type that represents the ports of the model. Each variant corresponds to a port.
    type Mux;

    fn enum_to_input(&mut self, input_enum: Self::Mux);
    fn output_to_enum(&self, output_fn: impl FnMut(Self::Mux));
}

/// Interface for DEVS components. All DEVS components must implement this trait.
///
/// # Safety
///
/// This trait must be implemented via macros. Do not implement it manually.
pub unsafe trait Component {
    /// Input event bag of the model.
    type Input: Bag;

    /// Output event bag of the model.
    type Output: Bag;

    /// Reference returned by the `get_ports` method.
    type InputRef<'a>
    where
        Self: 'a;

    /// Reference returned by the `get_ports` method.
    type OutputRef<'a>
    where
        Self: 'a;

    /// Returns the last time the component was updated.
    fn get_t_last(&self) -> f64;

    /// Sets the last time the component was updated.
    fn set_t_last(&mut self, t_last: f64);

    /// Returns the next time the component will be updated.
    fn get_t_next(&self) -> f64;

    /// Sets the next time the component will be updated.
    fn set_t_next(&mut self, t_next: f64);

    /// Returns a reference to the model's input event bag.
    fn get_input(&self) -> &Self::Input;

    /// Returns a mutable reference to the model's input event bag.
    fn get_input_mut(&mut self) -> &mut Self::Input;

    /// Returns a reference to the model's output event bag.
    fn get_output(&self) -> &Self::Output;

    /// Returns a mutable reference to the model's output event bag.
    fn get_output_mut(&mut self) -> &mut Self::Output;

    /// Returns both the input and output event bags as a tuple of references.
    fn get_ports(&mut self) -> (Self::InputRef<'_>, Self::OutputRef<'_>);

    /// Returns only the output event bag reference. Useful for output-only operations like lambda.
    fn get_out_ports(&self) -> Self::OutputRef<'_>;

    /// Clears the input bag, removing all values.
    #[inline]
    fn clear_input(&mut self) {
        self.get_input_mut().clear()
    }

    /// Clears the output bag, removing all values.
    #[inline]
    fn clear_output(&mut self) {
        self.get_output_mut().clear()
    }
}

/// Partial interface for DEVS atomic models.
/// It is used as a helper trait to implement the [`crate::Atomic`] trait.
///
/// # Safety
///
/// This trait must be implemented via macros. Do not implement it manually.
pub unsafe trait PartialAtomic: Component {
    /// The data type used to represent the state of the model.
    type State;
}

/// Partial interface for DEVS coupled models.
/// It is used as a helper trait to implement coupling logic.
///
/// # Safety
///
/// This trait must be implemented via macros. Do not implement it manually.
pub unsafe trait PartialCoupled: Component {
    /// Wrapper type holding references to all inner components' inputs.
    type ComponentsInput<'a>
    where
        Self: 'a;

    /// Wrapper type holding references to all inner components' outputs.
    type ComponentsOutput<'a>
    where
        Self: 'a;
}

/// Interface for simulating DEVS models. All DEVS models must implement this trait.
///
/// # Safety
///
/// This trait must be implemented via macros. Do not implement it manually.
pub unsafe trait AbstractSimulator: Component {
    /// It starts the simulation, setting the initial time to t_start.
    /// It returns the time for the next state transition of the inner DEVS model.
    fn start(&mut self, t_start: f64) -> f64;

    /// It stops the simulation, setting the last time to t_stop.
    fn stop(&mut self, t_stop: f64);

    /// Executes output functions and propagates messages according to EOCs.
    /// Internally, it checks that the model is imminent before executing.
    fn lambda(&mut self, t: f64);

    /// Propagates messages according to ICs and EICs and executes model transition functions.
    /// It also clears all the input and output ports.
    /// Internally, it checks that the model is imminent before executing.
    /// Finally, it returns the time for the next state transition of the inner DEVS model.
    fn delta(&mut self, t: f64) -> f64;
}

/// Input port interface for DEVS models that can be simulated in real-time using the `RtEngine`.
///
/// # Safety
///
/// This trait must be implemented via macros. Do not implement it manually.
pub unsafe trait MapInput: Bag {
    /// Input channel for the rt_engine macro.
    type InputChannel;

    /// Maps the input enum to the corresponding input port
    unsafe fn map_input(&mut self, in_channel: &Self::InputChannel) -> impl Future<Output = ()>;
}

/// Output port interface for DEVS models that can be simulated in real-time using the `RtEngine`.
///
/// # Safety
///
/// This trait must be implemented via macros. Do not implement it manually.
pub unsafe trait MapOutput: Bag {
    /// Output channel for the rt_engine macro.
    type OutputChannel;

    /// Maps the output enum to the corresponding output port
    unsafe fn map_output(&self, out_channel: &Self::OutputChannel);
}

/// Input channel for the rt_engine macro.
///
/// # Safety
///
/// This trait is implemented internally. Do not implement it manually.
pub unsafe trait RtEngineInputChannel: sealed::Sealed {
    /// Enum representing the input ports of the model. Each variant corresponds to an input port.
    type InputEnum;
    /// Type of the sender used to send input events to the model.
    type Sender;

    /// Returns a sender to the channel. The sender can be used to send input events to the model.
    fn sender(&self) -> Self::Sender;

    /// Receives from the channel and maps the received input event to the model's input ports.
    fn recv(&mut self) -> impl Future<Output = Self::InputEnum>;
}

/// Output channel for the rt_engine macro.
///
/// # Safety
///
/// This trait is implemented internally. Do not implement it manually.
pub unsafe trait RtEngineOutputChannel: sealed::Sealed {
    /// Enum representing the output ports of the model. Each variant corresponds to an output port.
    type OutputEnum;

    /// Type of the subscriber used to receive output events from the model.
    type Subscriber;

    /// Returns a subscriber to the channel. The subscriber can be used to receive output events from the model.
    fn subscriber(&self) -> Result<Self::Subscriber, crate::rt_engine::SubscribeError>;

    /// Publishes output events from the model to the channel, mapping the model's output ports to the channel's output events.
    fn publish(&self, output: Self::OutputEnum);
}

/// Interface for handling input events in an asynchronous DEVS simulation.
///
/// Unlike other traits, this trait must be implemented by the user, as it is not generated by macros.
/// It allows the model to handle input events asynchronously, waiting for external events without blocking the simulation.
pub trait AsyncInput {
    /// Set this to the input event bag type of your model under study.
    type Input: Bag;

    /// Handles input events asynchronously.
    ///
    /// It receives the time interval `[t_from, t_until]` and a mutable reference to the input event bag.
    /// It returns the time of the next event, which is usually the time of the next state transition.
    /// If an external event occurs, it should inject the event to the input and return the time at which the event happened.
    fn handle(
        &mut self,
        config: &Config,
        t_from: f64,
        t_until: f64,
        input: &mut Self::Input,
    ) -> impl Future<Output = f64>;
}

pub(crate) mod sealed {
    pub trait Sealed {}
}
