#![no_std]

pub use embassy_time::{Duration, Instant};
pub use xdevs_no_std_macros::*;

mod impls;
pub mod port;
pub mod simulator;
pub mod traits;

/// Interface for DEVS atomic models. All DEVS atomic models must implement this trait.
pub trait Atomic: traits::PartialAtomic {
    /// Method for performing any operation before simulating. By default, it does nothing.
    #[allow(unused_variables)]
    #[inline]
    fn start(state: &mut Self::State) {}

    /// Method for performing any operation after simulating. By default, it does nothing.
    #[allow(unused_variables)]
    #[inline]
    fn stop(state: &mut Self::State) {}

    /// Internal transition function. It modifies the state of the model when an internal event happens.
    fn delta_int(state: &mut Self::State);

    /// External transition function. It modifies the state of the model when an external event happens.
    /// The time elapsed since the last state transition is `e`.
    fn delta_ext(state: &mut Self::State, e: Duration, x: &Self::Input);

    /// Confluent transition function. It modifies the state of the model when an external and an internal event occur simultaneously.
    /// By default, it calls [`Atomic::delta_int`] and [`Atomic::delta_ext`] with `elapsed = 0`, in that order.
    #[inline]
    fn delta_conf(state: &mut Self::State, input: &Self::Input) {
        Self::delta_int(state);
        Self::delta_ext(state, Duration::from_secs(0), input); //cambiado
    }

    /// Output function. It triggers output events when an internal event is about to happen.
    fn lambda(state: &Self::State, output: &mut Self::Output);

    /// Time advance function. It returns the time until the next internal event happens.
    fn ta(state: &Self::State) -> Duration;
}

/// Interface for DEVS coupled models. All DEVS coupled models must implement this trait.
pub trait Coupled: traits::PartialCoupled {
    /// External Input Coupling. Propagates input events from the coupled model to its inner components.
    #[allow(unused_variables)]
    #[inline]
    fn eic(from: &Self::Input, to: &mut Self::ComponentsInput<'_>) {}

    /// Internal Coupling. Propagates output events from inner components to input events of other inner components.
    #[allow(unused_variables)]
    #[inline]
    fn ic(from: &Self::ComponentsOutput<'_>, to: &mut Self::ComponentsInput<'_>) {}

    /// External Output Coupling. Propagates output events from inner components to the coupled model's output.
    #[allow(unused_variables)]
    #[inline]
    fn eoc(from: &Self::ComponentsOutput<'_>, to: &mut Self::Output) {}
}
