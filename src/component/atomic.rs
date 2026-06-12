/// Interface for DEVS atomic models. All DEVS atomic models must implement this trait.
pub trait Atomic: xdevs::Component<Kind = xdevs::AtomicKind> {
    /// Method for performing any operation before simulating. By default, it does nothing.
    #[allow(unused_variables)]
    #[inline]
    fn start(&mut self) {}

    /// Method for performing any operation after simulating. By default, it does nothing.
    #[allow(unused_variables)]
    #[inline]
    fn stop(&mut self) {}

    /// Internal transition function. It modifies the state of the model when an internal event happens.
    fn delta_int(&mut self);

    /// External transition function. It modifies the state of the model when an external event happens.
    /// The time elapsed since the last state transition is `elapsed`.
    fn delta_ext(&mut self, elapsed: f64, input: &Self::Input);

    /// Confluent transition function. It modifies the state of the model when an external and an internal event occur simultaneously.
    /// By default, it calls [`Atomic::delta_int`] and [`Atomic::delta_ext`] with `elapsed = 0`, in that order.
    #[inline]
    fn delta_conf(&mut self, input: &Self::Input) {
        Self::delta_int(self);
        Self::delta_ext(self, 0., input);
    }

    /// Output function. It triggers output events when an internal event is about to happen.
    fn lambda(&self, output: &mut Self::Output);

    /// Time advance function. It returns the time until the next internal event happens.
    fn ta(&self) -> f64;
}
