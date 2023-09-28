use crate::component::Component;
use crate::port::UnsafePort;

/// Interface for DEVS atomic models.
///
/// # Safety
///
/// This trait must be implemented via the [`atomic!`] macro. Do not implement it manually.
pub unsafe trait UnsafeAtomic {
    /// The data type used to represent the state of the model.
    type State;

    /// The data type used to represent the input of the model.
    type Input: UnsafePort;

    /// The data type used to represent the output of the model.
    type Output: UnsafePort;

    /// Returns a tuple containing a reference to the state and a reference to the component.
    fn divide(&self) -> (&Self::State, &Component<Self::Input, Self::Output>);

    /// Returns a tuple containing a mutable reference to the state and a mutable reference to the component.
    fn divide_mut(&mut self) -> (&mut Self::State, &mut Component<Self::Input, Self::Output>);

    /// Returns a reference to the state.
    #[inline]
    fn get_state(&self) -> &Self::State {
        let (state, _) = self.divide();
        state
    }

    /// Returns a mutable reference to the state.
    #[inline]
    fn get_state_mut(&mut self) -> &mut Self::State {
        let (state, _) = self.divide_mut();
        state
    }

    /// Returns a reference to the component.
    #[inline]
    fn get_component(&self) -> &Component<Self::Input, Self::Output> {
        let (_, component) = self.divide();
        component
    }

    /// Returns a mutable reference to the component.
    #[inline]
    fn get_component_mut(&mut self) -> &mut Component<Self::Input, Self::Output> {
        let (_, component) = self.divide_mut();
        component
    }
}

pub trait Atomic: UnsafeAtomic {
    /// Method for performing any operation before simulating. By default, it does nothing.
    #[allow(unused_variables)]
    #[inline]
    fn start(state: &mut Self::State) {}

    /// Method for performing any operation after simulating. By default, it does nothing.
    #[allow(unused_variables)]
    #[inline]
    fn stop(state: &mut Self::State) {}

    /// Internal transition function.
    /// It modifies the state of the model when an internal event happens.
    fn delta_int(state: &mut Self::State);

    /// External transition function.
    /// It modifies the state of the model when an external event happens.
    /// The time elapsed since the last state transition is `e`.
    fn delta_ext(state: &mut Self::State, e: f64, x: &Self::Input);

    /// Confluent transition function.
    /// It modifies the state of the model when an external and an internal event occur simultaneously.
    /// By default, it calls [`Atomic::delta_int`] and [`Atomic::delta_ext`] with `e=0`, in that order.
    #[inline]
    fn delta_conf(state: &mut Self::State, x: &Self::Input) {
        Self::delta_int(state);
        Self::delta_ext(state, 0., x);
    }

    /// Output function.
    /// It triggers output events when an internal event is about to happen.
    fn lambda(state: &Self::State, output: &mut Self::Output);

    /// Time advance function.
    /// It returns the time until the next internal event happens.
    fn ta(state: &Self::State) -> f64;
}
