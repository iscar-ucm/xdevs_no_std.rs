use crate::{component::Component, port::Bag, ComponentsKind};
use core::{future::Future, time::Duration};

pub mod coordinator;
#[cfg(feature = "embassy")]
pub mod embassy;
pub mod simulator;
#[cfg(feature = "std")]
pub mod std;

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

/// Public simulation API for DEVS processors and processor collections.
///
/// This trait provides transition-level methods (`start`, `stop`, `lambda`, `delta`)
/// and high-level default simulation loops (`simulate_vt`, `simulate_rt`,
/// `simulate_rt_async`). It is implemented for every type that implements
/// [`AsProcessor`].
///
/// # Safety
///
/// This trait must be implemented internally or via the [`coupled`](crate::coupled) macro. Do not implement it manually.
pub unsafe trait AbstractSimulator {
    type Input: Bag;

    type Output: Bag;

    fn start(&mut self, t_start: f64) -> f64;

    fn stop(&mut self);

    fn lambda(&mut self, output: &mut Self::Output, t: f64);

    fn delta(&mut self, input: &mut Self::Input, output: &mut Self::Output, t: f64) -> f64;

    /// Executes simulation from `t_start` to `t_stop` using an external wait/input strategy.
    #[inline]
    fn simulate_rt(
        &mut self,
        config: &Config,
        mut wait_until: impl FnMut(f64, f64, &mut Self::Input) -> f64,
        mut propagate_output: impl FnMut(&Self::Output),
    ) {
        let t_start = config.t_start;
        let t_stop = config.t_stop;
        let mut t = t_start;
        let mut t_next_internal = self.start(t);
        let mut component_input = <Self::Input>::build();
        let mut component_output = <Self::Output>::build();
        while t < t_stop {
            let t_until = f64::min(t_next_internal, t_stop);
            t = wait_until(t, t_until, &mut component_input);
            if t >= t_next_internal {
                self.lambda(&mut component_output, t);
                propagate_output(&component_output);
            } else if component_input.is_empty() {
                continue; // avoid spurious external transitions
            }
            t_next_internal = self.delta(&mut component_input, &mut component_output, t);
        }
        self.stop();
    }

    /// Executes simulation from `t_start` to `t_stop` with a virtual clock.
    #[inline]
    fn simulate_vt(&mut self, config: &Config) {
        self.simulate_rt(config, |_, t_until, _| t_until, |_| {});
    }

    /// Asynchronous version of [`AbstractSimulator::simulate_rt`].
    fn simulate_rt_async(
        &mut self,
        config: &Config,
        mut input_handler: impl AsyncInput<Input = Self::Input>,
        mut propagate_output: impl FnMut(&Self::Output),
    ) -> impl Future<Output = ()> {
        async move {
            let mut t = config.t_start;
            let mut t_next_internal = self.start(t);
            let mut component_input = <Self::Input>::build();
            let mut component_output = <Self::Output>::build();
            while t < config.t_stop {
                let t_until = f64::min(t_next_internal, config.t_stop);
                t = input_handler
                    .handle(config, t, t_until, &mut component_input)
                    .await;
                if t >= t_next_internal {
                    self.lambda(&mut component_output, t);
                    propagate_output(&component_output);
                } else if component_input.is_empty() {
                    continue; // avoid spurious external transitions
                }
                t_next_internal = self.delta(&mut component_input, &mut component_output, t);
            }
            self.stop();
        }
    }
}

/// Bridge trait that specifies the simulator type for a given component kind.
pub trait Simulable<K>: Component<Kind = K> {
    /// The concrete simulator type that this component can be converted into.
    type Simulator: AbstractSimulator<Input = Self::Input, Output = Self::Output>;

    /// Converts the component into its corresponding simulator.
    fn to_simulator(self) -> Self::Simulator;
}

/// Helper trait for specifying the simulator type without requiring to be generic over the kind.
pub trait ErasedSimulable: Component {
    type Simulator: AbstractSimulator<Input = Self::Input, Output = Self::Output>;

    fn to_simulator(self) -> Self::Simulator;
}

impl<T, K> ErasedSimulable for T
where
    T: Component<Kind = K> + Simulable<K>,
{
    type Simulator = <T as Simulable<K>>::Simulator;

    #[inline(always)]
    fn to_simulator(self) -> Self::Simulator {
        <T as Simulable<K>>::to_simulator(self)
    }
}

/// Interface for handling input events in an asynchronous DEVS simulation.
///
/// Unlike other traits, this trait must be implemented by the user, as it is not generated by macros.
/// It allows the model to handle input events asynchronously, waiting for external events without blocking the simulation.
pub trait AsyncInput {
    /// Set this to the input event bag type of your model under study.
    type Input: Bag;

    /// Handles input events asynchronously.
    ///
    /// It receives the time interval `[t_from, t_until]` and a mutable reference to the input event bag.
    /// It returns the time of the next event, which is usually the time of the next state transition.
    /// If an external event occurs, it should inject the event to the input and return the time at which the event happened.
    fn handle(
        &mut self,
        config: &Config,
        t_from: f64,
        t_until: f64,
        input: &mut Self::Input,
    ) -> impl Future<Output = f64>;
}

unsafe impl<T: AbstractSimulator> AbstractSimulator for &mut T {
    type Input = T::Input;
    type Output = T::Output;

    #[inline(always)]
    fn start(&mut self, t_start: f64) -> f64 {
        T::start(self, t_start)
    }

    #[inline(always)]
    fn stop(&mut self) {
        T::stop(self)
    }

    #[inline(always)]
    fn lambda(&mut self, output: &mut Self::Output, t: f64) {
        T::lambda(self, output, t)
    }

    #[inline(always)]
    fn delta(&mut self, input: &mut Self::Input, output: &mut Self::Output, t: f64) -> f64 {
        T::delta(self, input, output, t)
    }
}

#[cfg(feature = "alloc")]
unsafe impl<T: AbstractSimulator> AbstractSimulator for alloc::boxed::Box<T> {
    type Input = T::Input;
    type Output = T::Output;

    #[inline(always)]
    fn start(&mut self, t_start: f64) -> f64 {
        T::start(self, t_start)
    }

    #[inline(always)]
    fn stop(&mut self) {
        T::stop(self)
    }

    #[inline(always)]
    fn lambda(&mut self, output: &mut Self::Output, t: f64) {
        T::lambda(self, output, t)
    }

    #[inline(always)]
    fn delta(&mut self, input: &mut Self::Input, output: &mut Self::Output, t: f64) -> f64 {
        T::delta(self, input, output, t)
    }
}

unsafe impl<T: AbstractSimulator, const N: usize> AbstractSimulator for [T; N] {
    type Input = [T::Input; N];
    type Output = [T::Output; N];

    #[inline(always)]
    fn start(&mut self, t_start: f64) -> f64 {
        self.iter_mut()
            .map(|processor| T::start(processor, t_start))
            .fold(f64::INFINITY, f64::min)
    }

    #[inline(always)]
    fn stop(&mut self) {
        self.iter_mut().for_each(|processor| T::stop(processor));
    }

    #[inline(always)]
    fn lambda(&mut self, output: &mut Self::Output, t: f64) {
        for (processor, output) in self.iter_mut().zip(output.iter_mut()) {
            T::lambda(processor, output, t);
        }
    }

    #[inline(always)]
    fn delta(&mut self, input: &mut Self::Input, output: &mut Self::Output, t: f64) -> f64 {
        self.iter_mut()
            .zip(input.iter_mut())
            .zip(output.iter_mut())
            .map(|((processor, input), output)| T::delta(processor, input, output, t))
            .fold(f64::INFINITY, f64::min)
    }
}

impl<T> Simulable<ComponentsKind> for T
where
    T: Component<Kind = ComponentsKind>,
    T: AbstractSimulator<Input = <T as Component>::Input, Output = <T as Component>::Output>,
{
    type Simulator = Self;

    #[inline(always)]
    fn to_simulator(self) -> Self::Simulator {
        self
    }
}
