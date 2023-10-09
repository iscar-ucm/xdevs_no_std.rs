use crate::port::UnsafePort;

pub struct Component<X: UnsafePort, Y: UnsafePort> {
    pub(crate) input: X,
    pub(crate) output: Y,
    pub(crate) t_last: f64,
    pub(crate) t_next: f64,
}

impl<X: UnsafePort, Y: UnsafePort> Component<X, Y> {
    #[inline]
    pub const fn new(input: X, output: Y) -> Self {
        Self {
            input,
            output,
            t_last: 0.0,
            t_next: f64::INFINITY,
        }
    }

    /// Sets the last and next times the component experienced an event.
    #[inline]
    pub(crate) fn set_sim_t(&mut self, t_last: f64, t_next: f64) {
        self.t_last = t_last;
        self.t_next = t_next;
    }

    /// Clears all the input and output ports.
    #[inline]
    pub(crate) fn clear_ports(&mut self) {
        self.input.clear();
        self.output.clear();
    }

    /// Returns true if the input port is empty.
    #[inline]
    pub(crate) fn is_input_empty(&self) -> bool {
        self.input.is_empty()
    }
}

pub unsafe trait UnsafeComponent {
    fn start(&mut self, t_start: f64) -> f64;
    fn stop(&mut self, t_stop: f64);
    fn lambda(&mut self, t: f64);
    fn delta(&mut self, t: f64) -> f64;
}
