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