use crate::traits::{AbstractSimulator, AsyncInput, Bag};
//use core::time::Duration;
use embassy_time::{Duration, Instant, Timer};

#[cfg(feature = "std")]
pub mod std;

#[cfg(feature = "embassy")]
pub mod embassy;

/// Configuration for the DEVS simulator.
#[derive(Debug, Clone, Copy)]
pub struct Config {
    /// The start time of the simulation.
    pub t_start: Instant,

    /// The stop time of the simulation.
    pub t_stop: Instant,

    /// The time scale factor for the simulation.
    ///
    /// If `mult` is greater than 1, the simulation runs faster than real time.
    pub mult: u64, //cambio time_scale por mult

    /// The maximum jitter duration allowed in the simulation.
    ///
    /// If `None`, jitter is not checked. If `Some(duration)`, the simulator will panic
    /// if the wall-clock time drift exceeds this duration.
    pub max_jitter: Option<Duration>,
}

impl Config {
    /// Creates a new `SimulatorConfig` with the specified parameters.
    #[inline]
    pub fn new(
        t_start: Instant,
        t_stop: Instant,
        mult: u64, //cambio time_scale por mult
        max_jitter: Option<Duration>,
    ) -> Self {
        Self {
            t_start,
            t_stop,
            mult, //cambio time_scale por mult
            max_jitter,
        }
    }
}

impl Default for Config {
    /// Default configuration runs from time 0.0 to infinity, with a
    /// time scale of 1.0 (real-time simulation) and no maximum jitter.
    #[inline]
    fn default() -> Self {
        Self::new(Instant::from_secs(0), Instant::from_secs(u64::MAX), 1, None)
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

    /// Returns a reference to the inner DEVS model.
    pub fn get_model(&self) -> &M {
        &self.model
    }

    /// It executes the simulation of the inner DEVS model from `t_start` to `t_stop`.
    /// It provides support for real time execution via the following arguments:
    ///
    /// - `wait_until`: a closure that is called between state transitions.
    ///   It receives the current time (NO), the time of the next state transition and a
    ///   mutable reference to the input ports. It returns time until which the simulation should wait.
    ///   If the returned time is equal to the input time, an internal/confluent state transition is performed.
    ///   Otherwise, it assumes that an external event happened and executes the external transition function.
    ///
    /// - `propagate_output`: a closure that is called after output functions.
    ///   It receives a mutable reference to the output ports so the closure can access to output events.
    #[inline]
    pub fn simulate_rt(
        &mut self,
        config: &Config,
        mut wait_until: impl FnMut(Instant, &mut M::Input) -> Instant,
        mut propagate_output: impl FnMut(&M::Output),
    ) {
        let t_start = config.t_start;
        let t_stop = config.t_stop;
        let mut t = t_start;
        //let mut t = Instant::now(); //cambio de t_start a Instant::now()
        let mut t_next_internal = self.model.start(t);
        while t < t_stop {
            let t_until = Instant::min(t_next_internal, t_stop);
            t = wait_until(t_until, self.model.get_input_mut());
            //t = t + elapsed;
            if t >= t_next_internal {
                self.model.lambda(t); //comprobar
                propagate_output(self.model.get_output());
            } else if self.model.get_input().is_empty() {
                continue; // avoid spurious external transitions
            }
            t_next_internal = self.model.delta(t); //comprobar
        }
        self.model.stop(t_stop);
    }

    /// It executes the simulation of the inner DEVS model from `t_start` to `t_stop`.
    /// It uses a virtual clock (i.e., no real time is used).
    #[inline]
    pub fn simulate_vt(&mut self, config: &Config) {
        self.simulate_rt(config, |t_until, _| t_until, |_| {}); //t_until a secas porque no hay eventos externos en el virtual
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
            let t_until = Instant::min(t_next_internal, config.t_stop);
            input_handler
                .handle(config, t_until, self.model.get_input_mut())
                .await; //como ahora input_handler no devuelve nada no modifica t
            t = t_until; //ahora comprobamos que se ha hecho en el tiempo que debería, no nos fiamos del valor que da. En simulate_vt hay que comprobar los tiempos en los que se realizan
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
