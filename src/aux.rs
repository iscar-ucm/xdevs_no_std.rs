/// Trait that defines the methods that a DEVS component port set must implement.
///
/// # Safety
///
/// This trait must be implemented via macros. Do not implement it manually.
pub unsafe trait Port {
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
    /// Input port set of the model.
    type Input: Port;

    /// Output port set of the model.
    type Output: Port;

    /// Returns the last time the component was updated.
    fn get_t_last(&self) -> f64;

    /// Sets the last time the component was updated.
    fn set_t_last(&mut self, t_last: f64);

    /// Returns the next time the component will be updated.
    fn get_t_next(&self) -> f64;

    /// Sets the next time the component will be updated.
    fn set_t_next(&mut self, t_next: f64);

    /// Clears the input ports, removing all values.
    fn clear_input(&mut self);

    /// Clears the output ports, removing all values.
    fn clear_output(&mut self);
}

/// Partial interface for DEVS atomic models.
/// It is used as a helper trait to implement the [`Atomic`] trait.
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
