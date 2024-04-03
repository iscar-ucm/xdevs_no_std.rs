use crate::traits::{AbstractSimulator, Bag};

#[cfg(feature = "std")]
pub mod std;

/// A DEVS simulator that uses a virtual clock (i.e., no real time is used).
pub struct Simulator<M: AbstractSimulator> {
    model: M,
}

impl<M: AbstractSimulator> Simulator<M> {
    #[inline]
    pub const fn new(model: M) -> Self {
        Self { model }
    }

    /// It executes the simulation of the inner DEVS model from `t_start` to `t_stop`.
    /// It provides support for real time execution via the following arguments:
    ///
    /// - `wait_until`: a closure that is called between state transitions.
    /// It receives the time of the next state transition and a mutable reference to the input ports.
    /// It returns the actual time "waited".
    /// If the returned time is equal to the input time, an internal/confluent state transition is performed.
    /// Otherwise, it assumes that an external event happened and executes the external transition function.
    /// - `propagate_output`: a closure that is called after output functions.
    /// It receives a mutable reference to the output ports so the closure can access to output events.
    /// Feel free to ignore this argument if you do not need to propagate output messages.
    #[inline]
    pub fn simulate_rt(
        &mut self,
        t_start: f64,
        t_stop: f64,
        mut wait_until: impl FnMut(f64, &mut M::Input) -> f64,
        mut propagate_output: impl FnMut(&M::Output),
    ) {
        let mut t = t_start;
        let mut t_next_internal = self.model.start(t);
        while t < t_stop {
            let t_until = f64::min(t_next_internal, t_stop);
            t = wait_until(t_until, self.model.get_input_mut());
            if t >= t_next_internal {
                self.model.lambda(t);
                propagate_output(self.model.get_output());
            } else if self.model.get_input().is_empty() {
                continue; // avoid spurious external transitions
            }
            t_next_internal = self.model.delta(t);
        }
        self.model.stop(t_stop);
    }

    /// It executes the simulation of the inner DEVS model from `t_start` to `t_stop`.
    /// It uses a virtual clock (i.e., no real time is used).
    #[inline]
    pub fn simulate_vt(&mut self, t_start: f64, t_stop: f64) {
        self.simulate_rt(t_start, t_stop, |t, _| t, |_| {});
    }
}
