use crate::traits::{AbstractSimulator, AsyncInput, Bag};
use core::time::Duration;

#[cfg(feature = "std")]
pub mod std;

#[cfg(feature = "embassy")]
pub mod embassy;

/// Configuration for the DEVS simulator.
#[derive(Debug, Clone, Copy)]
pub struct Config {
    /// The start time of the simulation.
    pub t_start: f64,

    /// The stop time of the simulation.
    pub t_stop: f64,

    /// The time scale factor for the simulation.
    ///
    /// If `time_scale` is greater than 1.0, the simulation runs faster than real time.
    /// If `time_scale` is less than 1.0, the simulation runs slower than real time.
    pub time_scale: f64,

    /// The maximum jitter duration allowed in the simulation.
    ///
    /// If `None`, jitter is not checked. If `Some(duration)`, the simulator will panic
    /// if the wall-clock time drift exceeds this duration.
    pub max_jitter: Option<Duration>,
}

impl Config {
    /// Creates a new `SimulatorConfig` with the specified parameters.
    #[inline]
    pub fn new(t_start: f64, t_stop: f64, time_scale: f64, max_jitter: Option<Duration>) -> Self {
        Self {
            t_start,
            t_stop,
            time_scale,
            max_jitter,
        }
    }
}

impl Default for Config {
    /// Default configuration runs from time 0.0 to infinity, with a
    /// time scale of 1.0 (real-time simulation) and no maximum jitter.
    #[inline]
    fn default() -> Self {
        Self::new(0.0, f64::INFINITY, 1.0, None)
    }
}

/// A DEVS simulator.
#[repr(transparent)]
pub struct Simulator<M: AbstractSimulator> {
    model: M,
}

impl<M: AbstractSimulator> Simulator<M> {
    /// Creates a new `Simulator` with the given DEVS model.
    #[inline]
    pub const fn new(model: M) -> Self {
        Self { model }
    }

    /// It executes the simulation of the inner DEVS model from `t_start` to `t_stop`.
    /// It provides support for real time execution via the following arguments:
    ///
    /// - `wait_until`: a closure that is called between state transitions.
    ///   It receives the current time, the time of the next state transition and a
    ///   mutable reference to the input ports. It returns the actual time "waited".
    ///   If the returned time is equal to the input time, an internal/confluent state transition is performed.
    ///   Otherwise, it assumes that an external event happened and executes the external transition function.
    ///
    /// - `propagate_output`: a closure that is called after output functions.
    ///   It receives a mutable reference to the output ports so the closure can access to output events.
    #[inline]
    pub fn simulate_rt(
        &mut self,
        config: &Config,
        mut wait_until: impl FnMut(f64, f64, &mut M::Input) -> f64,
        mut propagate_output: impl FnMut(&M::Output),
    ) {
        let t_start = config.t_start;
        let t_stop = config.t_stop;
        let mut t = t_start;
        let mut t_next_internal = self.model.start(t);
        while t < t_stop {
            let t_until = f64::min(t_next_internal, t_stop);
            t = wait_until(t, t_until, self.model.get_input_mut());
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
    pub fn simulate_vt(&mut self, config: &Config) {
        self.simulate_rt(config, |_, t_until, _| t_until, |_| {});
    }

    /// Asynchronous version of the `simulate_rt` method.
    ///
    /// The main difference is that the `wait_until` function has been replaced with an
    /// [`AsyncInput`] trait, which allows for asynchronous handling of input events.
    pub async fn simulate_rt_async(
        &mut self,
        config: &Config,
        mut input_handler: impl AsyncInput<Input = M::Input>,
        mut propagate_output: impl FnMut(&M::Output),
    ) {
        let mut t = config.t_start;
        let mut t_next_internal = self.model.start(t);
        while t < config.t_stop {
            let t_until = f64::min(t_next_internal, config.t_stop);
            t = input_handler
                .handle(config, t, t_until, self.model.get_input_mut())
                .await;
            if t >= t_next_internal {
                self.model.lambda(t);
                propagate_output(self.model.get_output());
            } else if self.model.get_input().is_empty() {
                continue; // avoid spurious external transitions
            }
            t_next_internal = self.model.delta(t);
        }
        self.model.stop(config.t_stop);
    }
}
