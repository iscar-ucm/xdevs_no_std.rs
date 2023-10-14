use crate::aux::AbstractSimulator;

#[repr(transparent)]
pub struct Simulator<T>(T);

impl<T: AbstractSimulator> Simulator<T> {
    #[inline]
    pub const fn new(simulator: T) -> Self {
        Self(simulator)
    }

    /// It executes the simulation of the inner DEVS model from `t_start` to `t_stop`.
    #[inline]
    pub fn simulate(&mut self, t_start: f64, t_stop: f64) {
        let mut t_next = self.0.start(t_start);
        while t_next < t_stop {
            self.0.lambda(t_next);
            t_next = self.0.delta(t_next);
        }
        self.0.stop(t_stop);
    }
}
