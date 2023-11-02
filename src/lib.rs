#![no_std]

pub use xdevs_no_std_macros::*;

pub mod aux;
pub mod port;
pub mod simulator;

/// Interface for DEVS atomic models. All DEVS atomic models must implement this trait.
pub trait Atomic: aux::PartialAtomic {
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
    fn delta_ext(state: &mut Self::State, e: f64, x: &Self::Input);

    /// Confluent transition function. It modifies the state of the model when an external and an internal event occur simultaneously.
    /// By default, it calls [`Atomic::delta_int`] and [`Atomic::delta_ext`] with `e = 0`, in that order.
    #[inline]
    fn delta_conf(state: &mut Self::State, x: &Self::Input) {
        Self::delta_int(state);
        Self::delta_ext(state, 0., x);
    }

    /// Output function. It triggers output events when an internal event is about to happen.
    fn lambda(state: &Self::State, output: &mut Self::Output);

    /// Time advance function. It returns the time until the next internal event happens.
    fn ta(state: &Self::State) -> f64;
}
