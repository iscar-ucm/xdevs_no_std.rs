use crate::{bag::Bag, Component, ComponentsKind, Duration};
use core::future::Future;
#[cfg(feature = "rayon")]
use rayon::prelude::*;

/// Helper trait for avoiding verbose trait constraints.
#[cfg(feature = "rayon")]
pub trait SimSend: Send {}
#[cfg(feature = "rayon")]
impl<T: Send> SimSend for T {}

/// Helper trait for avoiding verbose trait constraints.
#[cfg(not(feature = "rayon"))]
pub trait SimSend {}
#[cfg(not(feature = "rayon"))]
impl<T> SimSend for T {}

/// Re-export of `rayon::join` so proc-macro-generated code can reference it
/// through xdevs without requiring the consumer crate to depend on rayon directly.
#[cfg(feature = "rayon")]
pub use rayon::join as parallel_join;

pub mod coordinator;
pub mod simulator;

/// Configuration for the DEVS simulator.
#[derive(Debug, Clone, Copy)]
pub struct Config {
    /// The duration of the simulation.
    pub duration: Duration,

    /// The time multiplier for the simulation.
    ///
    /// Model time advances `mult` times faster than the wall clock. Since
    /// `duration` is measured in model time, the wall-clock run time is
    /// `duration / mult`. A value of 0 is treated as 1. Ignored by `simulate_vt`.
    pub mult: u64,

    /// The maximum jitter duration allowed in the simulation.
    ///
    /// If `None`, jitter is not checked. If `Some(duration)`, the simulator will panic
    /// if the wall-clock time drift exceeds this duration.
    pub max_jitter: Option<Duration>,
}

impl Config {
    /// Creates a new `SimulatorConfig` with the specified parameters.
    #[inline]
    pub fn new(duration: Duration, mult: u64, max_jitter: Option<Duration>) -> Self {
        Self {
            duration,
            mult,
            max_jitter,
        }
    }
}

impl Default for Config {
    /// Default configuration runs for an infinite duration, with a
    /// time scale of 1 (real-time simulation) and no maximum jitter.
    #[inline]
    fn default() -> Self {
        Self::new(Duration::MAX, 1, None)
    }
}

/// Public simulation API for DEVS processors and processor collections.
///
/// This trait provides transition-level methods (`start`, `stop`, `lambda`, `delta`)
/// and high-level default simulation loops (`simulate_vt`, `simulate_rt`).
///
/// # Safety
///
/// This trait must be implemented internally or via the [`coupled`](crate::coupled) macro. Do not implement it manually.
pub unsafe trait AbstractSimulator {
    type Input: Bag;

    type Output: Bag;

    fn start(&mut self) -> Duration;

    fn stop(&mut self);

    fn lambda(&mut self, output: &mut Self::Output, t: Duration);

    fn delta(
        &mut self,
        input: &mut Self::Input,
        output: &mut Self::Output,
        t: Duration,
    ) -> Duration;

    /// Performs a single simulation step up to time `t` and returns the time of
    /// the next transition. This method drives the simulation loop performed by other methods.
    #[inline]
    fn simulate_step(
        &mut self,
        input: &mut Self::Input,
        output: &mut Self::Output,
        t: Duration,
        t_next_internal: Duration,
        propagate: &mut impl FnMut(&Self::Output),
    ) -> Duration {
        let t = if t >= t_next_internal {
            self.lambda(output, t_next_internal);
            propagate(output);
            t_next_internal
        } else if input.is_empty() {
            return t_next_internal; // avoid spurious external transitions
        } else {
            t
        };
        self.delta(input, output, t)
    }

    /// Executes simulation from time 0 to `config.duration` with a virtual clock.
    #[inline]
    fn simulate_vt(&mut self, config: &Config) {
        let t_stop = config.duration;
        let mut t = Duration::ZERO;
        let mut t_next_internal = self.start();
        let mut component_input = <Self::Input>::build();
        let mut component_output = <Self::Output>::build();
        while t < t_stop {
            t = Duration::min(t_next_internal, t_stop);
            t_next_internal = self.simulate_step(
                &mut component_input,
                &mut component_output,
                t,
                t_next_internal,
                &mut |_| {},
            );
        }
        self.stop();
    }

    /// Executes simulation for `config.duration` of model time with a real-time
    /// clock and asynchronous input handling. Model time advances `config.mult`
    /// times faster than the wall clock.
    /// By default, this method uses the `Clock` implementation provided by the `xdevs::export` module.
    #[inline(always)]
    fn simulate_rt(
        &mut self,
        config: &Config,
        input_handler: impl AsyncInput<Input = Self::Input>,
        propagate_output: impl FnMut(&Self::Output),
    ) -> impl Future<Output = ()> {
        self.simulate_rt_clocked::<crate::export::Clock>(config, input_handler, propagate_output)
    }

    /// Executes simulation for `config.duration` of model time with a real-time
    /// clock and asynchronous input handling. Model time advances `config.mult`
    /// times faster than the wall clock.
    fn simulate_rt_clocked<C: crate::clock::Clock>(
        &mut self,
        config: &Config,
        mut input_handler: impl AsyncInput<Input = Self::Input>,
        mut propagate_output: impl FnMut(&Self::Output),
    ) -> impl Future<Output = ()> {
        async move {
            let t0 = C::now();
            let mult = config.mult.max(1);
            let t_stop = config.duration;
            let mut t = Duration::ZERO;
            let mut t_next_internal = self.start();
            let mut component_input = <Self::Input>::build();
            let mut component_output = <Self::Output>::build();
            while t < t_stop {
                let t_until = Duration::min(t_next_internal, t_stop);
                let future = input_handler.handle(&mut component_input);
                t = C::wait_until(&t0, t_until, future, mult).await;
                if t >= t_next_internal {
                    if let Some(max_jitter) = config.max_jitter {
                        let jitter =
                            Duration::from_micros(t.saturating_sub(t_until).as_micros() / mult);
                        if jitter > max_jitter {
                            panic!("Jitter too high: {:?} > {:?}", jitter, max_jitter);
                        }
                    }
                }
                t_next_internal = self.simulate_step(
                    &mut component_input,
                    &mut component_output,
                    t,
                    t_next_internal,
                    &mut propagate_output,
                );
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
pub trait SimpleSimulable: Component {
    type Simulator: AbstractSimulator<Input = Self::Input, Output = Self::Output>;

    fn to_simulator(self) -> Self::Simulator;
}

impl<T, K> SimpleSimulable for T
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
    /// It receives a mutable reference to the input event bag.
    /// The deadline is managed externally by the simulator via `embassy_time::with_deadline`.
    /// When called, it should wait for external input events.
    fn handle(&mut self, input: &mut Self::Input) -> impl Future<Output = ()>;
}

unsafe impl<T: AbstractSimulator> AbstractSimulator for &mut T {
    type Input = T::Input;
    type Output = T::Output;

    #[inline(always)]
    fn start(&mut self) -> Duration {
        T::start(self)
    }

    #[inline(always)]
    fn stop(&mut self) {
        T::stop(self)
    }

    #[inline(always)]
    fn lambda(&mut self, output: &mut Self::Output, t: Duration) {
        T::lambda(self, output, t)
    }

    #[inline(always)]
    fn delta(
        &mut self,
        input: &mut Self::Input,
        output: &mut Self::Output,
        t: Duration,
    ) -> Duration {
        T::delta(self, input, output, t)
    }
}

#[cfg(feature = "alloc")]
unsafe impl<T: AbstractSimulator> AbstractSimulator for alloc::boxed::Box<T> {
    type Input = T::Input;
    type Output = T::Output;

    #[inline(always)]
    fn start(&mut self) -> Duration {
        T::start(self)
    }

    #[inline(always)]
    fn stop(&mut self) {
        T::stop(self)
    }

    #[inline(always)]
    fn lambda(&mut self, output: &mut Self::Output, t: Duration) {
        T::lambda(self, output, t)
    }

    #[inline(always)]
    fn delta(
        &mut self,
        input: &mut Self::Input,
        output: &mut Self::Output,
        t: Duration,
    ) -> Duration {
        T::delta(self, input, output, t)
    }
}

unsafe impl<T: AbstractSimulator + SimSend, const N: usize> AbstractSimulator for [T; N]
where
    T::Input: SimSend,
    T::Output: SimSend,
{
    type Input = [T::Input; N];
    type Output = [T::Output; N];

    #[inline(always)]
    fn start(&mut self) -> Duration {
        #[cfg(feature = "rayon")]
        {
            self.par_iter_mut()
                .map(|processor| T::start(processor))
                .reduce(|| Duration::MAX, Duration::min)
        }
        #[cfg(not(feature = "rayon"))]
        {
            self.iter_mut()
                .map(|processor| T::start(processor))
                .fold(Duration::MAX, Duration::min)
        }
    }

    #[inline(always)]
    fn stop(&mut self) {
        #[cfg(feature = "rayon")]
        {
            self.par_iter_mut().for_each(|processor| T::stop(processor));
        }
        #[cfg(not(feature = "rayon"))]
        {
            self.iter_mut().for_each(|processor| T::stop(processor));
        }
    }

    #[inline(always)]
    fn lambda(&mut self, output: &mut Self::Output, t: Duration) {
        #[cfg(feature = "rayon")]
        {
            self.par_iter_mut()
                .zip(output.par_iter_mut())
                .for_each(|(processor, output)| T::lambda(processor, output, t));
        }
        #[cfg(not(feature = "rayon"))]
        {
            self.iter_mut()
                .zip(output.iter_mut())
                .for_each(|(processor, output)| T::lambda(processor, output, t));
        }
    }

    #[inline(always)]
    fn delta(
        &mut self,
        input: &mut Self::Input,
        output: &mut Self::Output,
        t: Duration,
    ) -> Duration {
        #[cfg(feature = "rayon")]
        {
            self.par_iter_mut()
                .zip(input.par_iter_mut())
                .zip(output.par_iter_mut())
                .map(|((processor, input), output)| T::delta(processor, input, output, t))
                .reduce(|| Duration::MAX, Duration::min)
        }
        #[cfg(not(feature = "rayon"))]
        {
            self.iter_mut()
                .zip(input.iter_mut())
                .zip(output.iter_mut())
                .map(|((processor, input), output)| T::delta(processor, input, output, t))
                .fold(Duration::MAX, Duration::min)
        }
    }
}

unsafe impl<T: AbstractSimulator> AbstractSimulator for Option<T> {
    type Input = T::Input;
    type Output = T::Output;

    #[inline(always)]
    fn start(&mut self) -> Duration {
        match self {
            Some(processor) => T::start(processor),
            None => Duration::MAX,
        }
    }

    #[inline(always)]
    fn stop(&mut self) {
        if let Some(processor) = self {
            T::stop(processor);
        }
    }

    #[inline(always)]
    fn lambda(&mut self, output: &mut Self::Output, t: Duration) {
        if let Some(processor) = self {
            T::lambda(processor, output, t);
        }
    }

    #[inline(always)]
    fn delta(
        &mut self,
        input: &mut Self::Input,
        output: &mut Self::Output,
        t: Duration,
    ) -> Duration {
        match self {
            Some(processor) => T::delta(processor, input, output, t),
            None => {
                input.clear();
                Duration::MAX
            }
        }
    }
}

#[cfg(feature = "rayon")]
mod tuple_macros {
    // Splitter that deinterleaves a list of tts into even-indexed and odd-indexed halves.
    macro_rules! split_even_odd {
    ([$($even:tt)*] [$($odd:tt)*] [] $cont:ident $ctx:tt) => {
        tuple_macros::$cont!([$($even)*] [$($odd)*] $ctx)
    };
    ([$($even:tt)*] [$($odd:tt)*] [$a:tt] $cont:ident $ctx:tt) => {
        tuple_macros::$cont!([$($even)* $a] [$($odd)*] $ctx)
    };
    ([$($even:tt)*] [$($odd:tt)*] [$a:tt $b:tt $($rest:tt)*] $cont:ident $ctx:tt) => {
        tuple_macros::split_even_odd!([$($even)* $a] [$($odd)* $b] [$($rest)*] $cont $ctx)
    };
}

    // Balanced rayon::join tree for tuple start (returns Duration::min).
    macro_rules! par_start {
    ($self:expr, [$idx:tt]) => {
        $crate::simulation::AbstractSimulator::start(&mut $self.$idx)
    };
    ($self:expr, [$idx:tt $($rest:tt)+]) => {
        tuple_macros::split_even_odd!([] [] [$idx $($rest)+] par_start_node [$self])
    };
}

    macro_rules! par_start_node {
    ([$($even:tt)*] [$($odd:tt)*] [$self:expr]) => {{
        let (a, b) = ::rayon::join(
            || tuple_macros::par_start!($self, [$($even)*]),
            || tuple_macros::par_start!($self, [$($odd)*]),
        );
        Duration::min(a, b)
    }};
}

    // Balanced rayon::join tree for tuple stop (returns ()).
    macro_rules! par_stop {
    ($self:expr, [$idx:tt]) => {
        $crate::simulation::AbstractSimulator::stop(&mut $self.$idx)
    };
    ($self:expr, [$idx:tt $($rest:tt)+]) => {
        tuple_macros::split_even_odd!([] [] [$idx $($rest)+] par_stop_node [$self])
    };
}

    macro_rules! par_stop_node {
    ([$($even:tt)*] [$($odd:tt)*] [$self:expr]) => {{
        ::rayon::join(
            || tuple_macros::par_stop!($self, [$($even)*]),
            || tuple_macros::par_stop!($self, [$($odd)*]),
        );
    }};
}

    // Balanced rayon::join tree for tuple lambda (returns ()).
    macro_rules! par_lambda {
    ($self:expr, $output:expr, $t:expr, [$idx:tt]) => {
        $crate::simulation::AbstractSimulator::lambda(&mut $self.$idx, &mut $output.$idx, $t)
    };
    ($self:expr, $output:expr, $t:expr, [$idx:tt $($rest:tt)+]) => {
        tuple_macros::split_even_odd!([] [] [$idx $($rest)+] par_lambda_node [$self, $output, $t])
    };
}

    macro_rules! par_lambda_node {
    ([$($even:tt)*] [$($odd:tt)*] [$self:expr, $output:expr, $t:expr]) => {{
        ::rayon::join(
            || tuple_macros::par_lambda!($self, $output, $t, [$($even)*]),
            || tuple_macros::par_lambda!($self, $output, $t, [$($odd)*]),
        );
    }};
}

    // Balanced rayon::join tree for tuple delta (returns Duration::min).
    macro_rules! par_delta {
    ($self:expr, $input:expr, $output:expr, $t:expr, [$idx:tt]) => {
        $crate::simulation::AbstractSimulator::delta(
            &mut $self.$idx, &mut $input.$idx, &mut $output.$idx, $t)
    };
    ($self:expr, $input:expr, $output:expr, $t:expr, [$idx:tt $($rest:tt)+]) => {
        tuple_macros::split_even_odd!([] [] [$idx $($rest)+] par_delta_node [$self, $input, $output, $t])
    };
}

    macro_rules! par_delta_node {
    ([$($even:tt)*] [$($odd:tt)*] [$self:expr, $input:expr, $output:expr, $t:expr]) => {{
        let (a, b) = ::rayon::join(
            || tuple_macros::par_delta!($self, $input, $output, $t, [$($even)*]),
            || tuple_macros::par_delta!($self, $input, $output, $t, [$($odd)*]),
        );
        Duration::min(a, b)
    }};
}

    macro_rules! impl_abstract_simulator_for_tuple {
    ($($idx:tt => $T:ident),+) => {
        unsafe impl<$($T: AbstractSimulator + SimSend),+> AbstractSimulator for ($($T,)+)
        where
            $($T::Input: SimSend, $T::Output: SimSend),+
        {
            type Input = ($($T::Input,)+);
            type Output = ($($T::Output,)+);

            #[inline(always)]
            fn start(&mut self) -> Duration {
                tuple_macros::par_start!(self, [$($idx)+])
            }

            #[inline(always)]
            fn stop(&mut self) {
                tuple_macros::par_stop!(self, [$($idx)+])
            }

            #[inline(always)]
            fn lambda(&mut self, output: &mut Self::Output, t: Duration) {
                tuple_macros::par_lambda!(self, output, t, [$($idx)+])
            }

            #[inline(always)]
            fn delta(&mut self, input: &mut Self::Input, output: &mut Self::Output, t: Duration) -> Duration {
                tuple_macros::par_delta!(self, input, output, t, [$($idx)+])
            }
        }
    }
    }
    pub(crate) use impl_abstract_simulator_for_tuple;
    pub(crate) use par_delta;
    pub(crate) use par_delta_node;
    pub(crate) use par_lambda;
    pub(crate) use par_lambda_node;
    pub(crate) use par_start;
    pub(crate) use par_start_node;
    pub(crate) use par_stop;
    pub(crate) use par_stop_node;
    pub(crate) use split_even_odd;
}

#[cfg(not(feature = "rayon"))]
mod tuple_macros {
    macro_rules! impl_abstract_simulator_for_tuple {
    ($($idx:tt => $T:ident),+) => {
        unsafe impl<$($T: AbstractSimulator + SimSend),+> AbstractSimulator for ($($T,)+)
        where
            $($T::Input: SimSend, $T::Output: SimSend),+
        {
            type Input = ($($T::Input,)+);
            type Output = ($($T::Output,)+);

            #[inline(always)]
            fn start(&mut self) -> Duration {
                let mut min_t = Duration::MAX;
                $(min_t = Duration::min(min_t, self.$idx.start());)+
                min_t

            }

            #[inline(always)]
            fn stop(&mut self) {
                $(self.$idx.stop();)+

            }

            #[inline(always)]
            fn lambda(&mut self, output: &mut Self::Output, t: Duration) {
                $(self.$idx.lambda(&mut output.$idx, t);)+
            }

            #[inline(always)]
            fn delta(&mut self, input: &mut Self::Input, output: &mut Self::Output, t: Duration) -> Duration {
                let mut min_t = Duration::MAX;
                $(min_t = Duration::min(min_t, self.$idx.delta(&mut input.$idx, &mut output.$idx, t));)+
                min_t
            }
        }
    }
}
    pub(crate) use impl_abstract_simulator_for_tuple;
}

tuple_macros::impl_abstract_simulator_for_tuple!(0 => T0);
tuple_macros::impl_abstract_simulator_for_tuple!(0 => T0, 1 => T1);
tuple_macros::impl_abstract_simulator_for_tuple!(0 => T0, 1 => T1, 2 => T2);
tuple_macros::impl_abstract_simulator_for_tuple!(0 => T0, 1 => T1, 2 => T2, 3 => T3);
tuple_macros::impl_abstract_simulator_for_tuple!(0 => T0, 1 => T1, 2 => T2, 3 => T3, 4 => T4);
tuple_macros::impl_abstract_simulator_for_tuple!(0 => T0, 1 => T1, 2 => T2, 3 => T3, 4 => T4, 5 => T5);
tuple_macros::impl_abstract_simulator_for_tuple!(0 => T0, 1 => T1, 2 => T2, 3 => T3, 4 => T4, 5 => T5, 6 => T6);
tuple_macros::impl_abstract_simulator_for_tuple!(0 => T0, 1 => T1, 2 => T2, 3 => T3, 4 => T4, 5 => T5, 6 => T6, 7 => T7);
tuple_macros::impl_abstract_simulator_for_tuple!(0 => T0, 1 => T1, 2 => T2, 3 => T3, 4 => T4, 5 => T5, 6 => T6, 7 => T7, 8 => T8);
tuple_macros::impl_abstract_simulator_for_tuple!(0 => T0, 1 => T1, 2 => T2, 3 => T3, 4 => T4, 5 => T5, 6 => T6, 7 => T7, 8 => T8, 9 => T9);
tuple_macros::impl_abstract_simulator_for_tuple!(0 => T0, 1 => T1, 2 => T2, 3 => T3, 4 => T4, 5 => T5, 6 => T6, 7 => T7, 8 => T8, 9 => T9, 10 => T10);
tuple_macros::impl_abstract_simulator_for_tuple!(0 => T0, 1 => T1, 2 => T2, 3 => T3, 4 => T4, 5 => T5, 6 => T6, 7 => T7, 8 => T8, 9 => T9, 10 => T10, 11 => T11);

macro_rules! impl_simulable_for_tuple {
    ($($idx:tt => ($T:ident, $K:ident)),+) => {
        impl<$($T, $K),+> Simulable<($($K,)+)> for ($($T,)+)
        where
            $($T: Component<Kind = $K> + Simulable<$K>),+,
            $($K: crate::component::sealed::Sealed),+,
            $($T::Simulator: SimSend, $T::Input: SimSend, $T::Output: SimSend),+
        {
            type Simulator = ($($T::Simulator,)+);

            #[inline(always)]
            fn to_simulator(self) -> Self::Simulator {
                ($(self.$idx.to_simulator(),)+)
            }
        }
    }
}

impl_simulable_for_tuple!(0 => (T0, K0));
impl_simulable_for_tuple!(0 => (T0, K0), 1 => (T1, K1));
impl_simulable_for_tuple!(0 => (T0, K0), 1 => (T1, K1), 2 => (T2, K2));
impl_simulable_for_tuple!(0 => (T0, K0), 1 => (T1, K1), 2 => (T2, K2), 3 => (T3, K3));
impl_simulable_for_tuple!(0 => (T0, K0), 1 => (T1, K1), 2 => (T2, K2), 3 => (T3, K3), 4 => (T4, K4));
impl_simulable_for_tuple!(0 => (T0, K0), 1 => (T1, K1), 2 => (T2, K2), 3 => (T3, K3), 4 => (T4, K4), 5 => (T5, K5));
impl_simulable_for_tuple!(0 => (T0, K0), 1 => (T1, K1), 2 => (T2, K2), 3 => (T3, K3), 4 => (T4, K4), 5 => (T5, K5), 6 => (T6, K6));
impl_simulable_for_tuple!(0 => (T0, K0), 1 => (T1, K1), 2 => (T2, K2), 3 => (T3, K3), 4 => (T4, K4), 5 => (T5, K5), 6 => (T6, K6), 7 => (T7, K7));
impl_simulable_for_tuple!(0 => (T0, K0), 1 => (T1, K1), 2 => (T2, K2), 3 => (T3, K3), 4 => (T4, K4), 5 => (T5, K5), 6 => (T6, K6), 7 => (T7, K7), 8 => (T8, K8));
impl_simulable_for_tuple!(0 => (T0, K0), 1 => (T1, K1), 2 => (T2, K2), 3 => (T3, K3), 4 => (T4, K4), 5 => (T5, K5), 6 => (T6, K6), 7 => (T7, K7), 8 => (T8, K8), 9 => (T9, K9));
impl_simulable_for_tuple!(0 => (T0, K0), 1 => (T1, K1), 2 => (T2, K2), 3 => (T3, K3), 4 => (T4, K4), 5 => (T5, K5), 6 => (T6, K6), 7 => (T7, K7), 8 => (T8, K8), 9 => (T9, K9), 10 => (T10, K10));
impl_simulable_for_tuple!(0 => (T0, K0), 1 => (T1, K1), 2 => (T2, K2), 3 => (T3, K3), 4 => (T4, K4), 5 => (T5, K5), 6 => (T6, K6), 7 => (T7, K7), 8 => (T8, K8), 9 => (T9, K9), 10 => (T10, K10), 11 => (T11, K11));

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

impl<T, K, const N: usize> Simulable<[K; N]> for [T; N]
where
    T: Component<Kind = K>,
    T: Simulable<K>,
    K: crate::component::sealed::Sealed,
    T::Simulator: SimSend,
    T::Input: SimSend,
    T::Output: SimSend,
{
    type Simulator = [T::Simulator; N];

    #[inline(always)]
    fn to_simulator(self) -> Self::Simulator {
        self.map(|component| component.to_simulator())
    }
}

impl<T, K> Simulable<Option<K>> for Option<T>
where
    T: Component<Kind = K>,
    T: Simulable<K>,
    K: crate::component::sealed::Sealed,
{
    type Simulator = Option<T::Simulator>;

    #[inline(always)]
    fn to_simulator(self) -> Self::Simulator {
        self.map(|component| component.to_simulator())
    }
}

/// A simple asynchronous input handler that sleeps until the next state transition of the model.
#[derive(Default)]
pub struct SleepAsync<T: Bag> {
    /// Phantom data to associate with the input bag type.
    input: core::marker::PhantomData<T>,
}

impl<T: Bag> SleepAsync<T> {
    /// Creates a new `SleepAsync` instance.
    pub fn new() -> Self {
        Self {
            input: core::marker::PhantomData,
        }
    }
}

impl<T: Bag> AsyncInput for SleepAsync<T> {
    type Input = T;

    async fn handle(&mut self, _input: &mut Self::Input) {
        core::future::pending::<()>().await
    }
}

// Module with models for simulation, simulator and coordinator testing
#[cfg(test)]
pub(crate) mod test_utils {
    use crate::{
        couple, Atomic, AtomicKind, Bag, Component, ComponentsInput, ComponentsOutput, Coupled,
        CoupledKind, Duration, Port,
    };

    pub(crate) struct TestAtomic {
        pub sigma: Duration,
        pub period: Duration,
        pub int_calls: usize,
        pub ext_calls: usize,
        pub last_elapsed: Duration,
        pub out_val: usize,
    }

    impl Component for TestAtomic {
        type Kind = AtomicKind;
        type Input = Port<usize, 1>;
        type Output = Port<usize, 1>;
    }

    impl Atomic for TestAtomic {
        fn delta_int(&mut self) {
            self.int_calls += 1;
            self.sigma = self.period;
        }
        fn delta_ext(&mut self, elapsed: Duration, _input: &Self::Input) {
            self.ext_calls += 1;
            self.last_elapsed = elapsed;
            self.sigma = Duration::from_secs(0);
        }
        fn lambda(&self, output: &mut Self::Output) {
            let _ = output.add_value(self.out_val);
        }
        fn ta(&self) -> Duration {
            self.sigma
        }
    }

    impl TestAtomic {
        pub(crate) fn periodic(sigma: Duration, period: Duration) -> Self {
            Self {
                sigma,
                period,
                int_calls: 0,
                ext_calls: 0,
                last_elapsed: Duration::from_secs(0),
                out_val: 99,
            }
        }
        pub(crate) fn oneshot(sigma: Duration) -> Self {
            Self::periodic(sigma, Duration::MAX)
        }
    }

    #[crate::coupled]
    pub(crate) struct TestCoupled {
        pub a0: TestAtomic,
        pub a1: TestAtomic,
    }

    impl Component for TestCoupled {
        type Kind = CoupledKind;
        type Input = Port<usize, 1>;
        type Output = Port<usize, 1>;
    }

    impl Coupled for TestCoupled {
        fn eic(from: &Self::Input, to: &mut ComponentsInput<Self>) {
            let _ = couple(from, &mut to.a0);
        }
        fn ic(from: &ComponentsOutput<Self>, to: &mut ComponentsInput<Self>) {
            let _ = couple(&from.a0, &mut to.a1);
        }
        fn eoc(from: &ComponentsOutput<Self>, to: &mut Self::Output) {
            let _ = couple(&from.a1, to);
        }
    }

    #[crate::coupled]
    pub(crate) struct TestCoupledWithOption {
        pub a0: TestAtomic,
        pub opt: Option<TestAtomic>,
    }

    impl Component for TestCoupledWithOption {
        type Kind = CoupledKind;
        type Input = Port<usize, 1>;
        type Output = ();
    }

    impl Coupled for TestCoupledWithOption {
        fn eic(from: &Self::Input, to: &mut ComponentsInput<Self>) {
            let _ = couple(from, &mut to.a0);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::test_utils::{TestAtomic, TestCoupled, TestCoupledWithOption};
    use crate::{
        component::coupled::PartialCoupled,
        prelude::*,
        simulation::{simulator::Simulator, Config},
        Component, Duration, Instant, Port,
    };
    #[test]
    fn step_returns_next_transition_time() {
        let mut sim =
            TestAtomic::periodic(Duration::from_secs(0), Duration::from_secs(2)).to_simulator();
        let mut input = Port::<usize, 1>::new();
        let mut output = Port::<usize, 1>::new();
        let t_next = sim.start();
        let next = sim.simulate_step(&mut input, &mut output, t_next, t_next, &mut |_| {});

        assert_eq!(next, Duration::from_secs(2), "next transition after delta");
        assert_eq!(sim.int_calls, 1, "internal transition at t_next");
    }

    #[test]
    fn step_without_transition_returns_same_time() {
        let mut sim = TestAtomic::oneshot(Duration::from_secs(5)).to_simulator();
        let mut input = Port::<usize, 1>::new();
        let mut output = Port::<usize, 1>::new();
        let t_next = sim.start();
        let next = sim.simulate_step(
            &mut input,
            &mut output,
            Duration::from_secs(2),
            t_next,
            &mut |_| {},
        );

        assert_eq!(next, t_next, "no transition before t_next");
        assert_eq!(sim.int_calls, 0, "no internal transition");
        assert_eq!(sim.ext_calls, 0, "no external transition");
    }

    #[test]
    fn simulate_vt_single_event() {
        let mut sim = TestAtomic::oneshot(Duration::from_secs(5)).to_simulator();
        let config = Config::new(Duration::from_secs(20), 1, None);
        sim.simulate_vt(&config);

        assert_eq!(sim.int_calls, 1, "one internal transition");
        assert_eq!(sim.ext_calls, 0, "no external transitions");
    }

    #[test]
    fn simulate_vt_multiple_events() {
        let mut sim =
            TestAtomic::periodic(Duration::from_secs(0), Duration::from_secs(2)).to_simulator();
        let config = Config::new(Duration::from_secs(9), 1, None);
        sim.simulate_vt(&config);

        assert_eq!(sim.int_calls, 5, "expected 5 internal transitions in 9s");
        assert_eq!(sim.ext_calls, 0, "no external transitions");
    }

    #[test]
    fn simulate_vt_no_spurious_transitions() {
        let mut sim = TestAtomic::oneshot(Duration::MAX).to_simulator();
        let config = Config::new(Duration::from_secs(10), 1, None);
        sim.simulate_vt(&config);

        assert_eq!(sim.int_calls, 0, "no internal events");
        assert_eq!(sim.ext_calls, 0, "no external events");
    }

    #[tokio::test]
    async fn simulate_rt_single_event() {
        let mut sim = TestAtomic::oneshot(Duration::from_millis(5)).to_simulator();
        let config = Config::new(Duration::from_millis(10), 1, None);
        sim.simulate_rt(&config, IdentityAsyncInput, |_| {}).await;
        assert_eq!(sim.int_calls, 1, "rt single event");
        assert_eq!(sim.ext_calls, 0, "no external transitions");
    }

    #[tokio::test]
    async fn simulate_rt_injects_external_input() {
        let mut sim = TestAtomic::oneshot(Duration::from_millis(5)).to_simulator();
        let config = Config::new(Duration::from_millis(10), 1, None);

        struct InjectInput {
            injected: bool,
        }
        impl crate::simulation::AsyncInput for InjectInput {
            type Input = Port<usize, 1>;
            async fn handle(&mut self, input: &mut Self::Input) {
                if !self.injected {
                    self.injected = true;
                    input.add_value(99).unwrap();
                } else {
                    core::future::pending::<()>().await
                }
            }
        }

        sim.simulate_rt(&config, InjectInput { injected: false }, |_| {})
            .await;

        assert_eq!(sim.ext_calls, 1, "external transition via input_handler");
    }

    #[tokio::test]
    async fn simulate_rt_propagate_output() {
        let mut sim = TestAtomic::oneshot(Duration::from_millis(5)).to_simulator();
        let config = Config::new(Duration::from_millis(10), 1, None);
        let mut captured = Port::<usize, 1>::new();

        sim.simulate_rt(&config, IdentityAsyncInput, |output| {
            for v in output.get_values() {
                let _ = captured.add_value(v);
            }
        })
        .await;

        assert_eq!(
            captured.as_slice(),
            &[99],
            "propagate_output captures lambda output"
        );
    }

    struct IdentityAsyncInput;

    impl crate::simulation::AsyncInput for IdentityAsyncInput {
        type Input = Port<usize, 1>;
        async fn handle(&mut self, _input: &mut Self::Input) {
            core::future::pending::<()>().await
        }
    }

    #[tokio::test]
    async fn simulate_rt_single_event_async() {
        let mut sim = TestAtomic::oneshot(Duration::from_millis(5)).to_simulator();
        let config = Config::new(Duration::from_millis(10), 1, None);
        sim.simulate_rt(&config, IdentityAsyncInput, |_| {}).await;
        assert_eq!(sim.int_calls, 1, "async single event");
        assert_eq!(sim.ext_calls, 0, "no external transitions");
    }

    #[tokio::test]
    async fn simulate_rt_external_input_async() {
        let mut sim = TestAtomic::oneshot(Duration::from_millis(5)).to_simulator();
        let config = Config::new(Duration::from_millis(10), 1, None);

        struct InjectInput {
            injected: bool,
        }
        impl crate::simulation::AsyncInput for InjectInput {
            type Input = Port<usize, 1>;
            async fn handle(&mut self, input: &mut Self::Input) {
                if !self.injected {
                    self.injected = true;
                    input.add_value(99).unwrap();
                } else {
                    core::future::pending::<()>().await
                }
            }
        }

        sim.simulate_rt(&config, InjectInput { injected: false }, |_| {})
            .await;
        assert_eq!(sim.ext_calls, 1, "async external input");
    }

    #[tokio::test]
    async fn simulate_rt_propagate_output_async() {
        let mut sim = TestAtomic::oneshot(Duration::from_millis(5)).to_simulator();
        let config = Config::new(Duration::from_millis(10), 1, None);
        let mut captured = Port::<usize, 1>::new();

        sim.simulate_rt(&config, IdentityAsyncInput, |output| {
            for v in output.get_values() {
                let _ = captured.add_value(v);
            }
        })
        .await;

        assert_eq!(captured.as_slice(), &[99], "async propagate_output");
    }

    #[tokio::test]
    async fn simulate_rt_respects_duration() {
        let mut sim = TestAtomic::oneshot(Duration::from_millis(5)).to_simulator();
        let config = Config::new(Duration::from_millis(20), 1, None);

        let start = Instant::now();
        sim.simulate_rt(&config, IdentityAsyncInput, |_| {}).await;
        let elapsed = Duration::from_micros(Instant::now().duration_since(start).as_micros());

        assert!(
            elapsed >= Duration::from_millis(20) && elapsed < Duration::from_millis(100),
            "rt simulation must run for the whole duration, elapsed: {} ms",
            elapsed.as_millis(),
        );
    }

    #[tokio::test]
    async fn simulate_rt_mult_speeds_up_wall_time() {
        let mut sim = TestAtomic::oneshot(Duration::from_millis(5)).to_simulator();
        let config = Config::new(Duration::from_millis(200), 4, None);

        let start = Instant::now();
        sim.simulate_rt(&config, IdentityAsyncInput, |_| {}).await;
        let elapsed = Duration::from_micros(Instant::now().duration_since(start).as_micros());

        assert_eq!(sim.int_calls, 1, "internal event fires");
        assert!(
            elapsed >= Duration::from_millis(50) && elapsed < Duration::from_millis(150),
            "mult 4 must run 200ms of model time in ~50ms wall time, elapsed: {} ms",
            elapsed.as_millis(),
        );
    }

    #[tokio::test]
    async fn simulate_rt_mult_zero_clamped_to_one() {
        let mut sim = TestAtomic::oneshot(Duration::from_millis(5)).to_simulator();
        let config = Config::new(Duration::from_millis(20), 0, None);

        let start = Instant::now();
        sim.simulate_rt(&config, IdentityAsyncInput, |_| {}).await;
        let elapsed = Duration::from_micros(Instant::now().duration_since(start).as_micros());

        assert_eq!(sim.int_calls, 1, "internal event fires");
        assert!(
            elapsed >= Duration::from_millis(20),
            "mult 0 must behave as real time, elapsed: {} ms",
            elapsed.as_millis(),
        );
    }

    #[test]
    fn array_start_returns_min() {
        let a0 = TestAtomic::oneshot(Duration::from_secs(3));
        let a1 = TestAtomic::oneshot(Duration::from_secs(1));
        let a2 = TestAtomic::oneshot(Duration::from_secs(5));
        let mut arr = [a0, a1, a2].to_simulator();

        let t = arr.start();
        assert_eq!(t, Duration::from_secs(1), "min of 3, 1, 5 is 1");
    }

    #[test]
    fn array_lambda_iterates_all() {
        let mut arr = [
            TestAtomic::periodic(Duration::from_secs(0), Duration::from_secs(1)),
            TestAtomic::periodic(Duration::from_secs(0), Duration::from_secs(1)),
        ]
        .to_simulator();
        arr.start();

        let mut output = [Port::<usize, 1>::new(), Port::<usize, 1>::new()];
        arr.lambda(&mut output, Duration::from_secs(0));

        assert_eq!(output[0].as_slice(), &[99], "first atomic lambda ran");
        assert_eq!(output[1].as_slice(), &[99], "second atomic lambda ran");
    }

    #[test]
    fn array_delta_iterates_all() {
        let mut arr = [
            TestAtomic::periodic(Duration::from_secs(0), Duration::from_secs(1)),
            TestAtomic::periodic(Duration::from_secs(0), Duration::from_secs(1)),
        ]
        .to_simulator();
        arr.start();

        let mut input = [Port::<usize, 1>::new(), Port::<usize, 1>::new()];
        let mut output = [Port::<usize, 1>::new(), Port::<usize, 1>::new()];
        let t = arr.delta(&mut input, &mut output, Duration::from_secs(0));

        assert_eq!(arr[0].int_calls, 1, "first atomic delta_int");
        assert_eq!(arr[1].int_calls, 1, "second atomic delta_int");
        assert!(t > Duration::from_secs(0), "t_next should be > 0");
    }

    #[test]
    fn array_stop_iterates_all() {
        let mut arr = [
            TestAtomic::periodic(Duration::from_secs(0), Duration::from_secs(1)),
            TestAtomic::periodic(Duration::from_secs(0), Duration::from_secs(1)),
        ]
        .to_simulator();
        arr.start();
        arr.stop();

        assert_eq!(arr[0].ext_calls, 0, "stop on first array element");
        assert_eq!(arr[1].ext_calls, 0, "stop on second array element");
    }

    #[test]
    fn option_some_delegates() {
        let mut opt = Some(TestAtomic::periodic(
            Duration::from_secs(0),
            Duration::from_secs(1),
        ))
        .to_simulator();

        let t = opt.start();
        assert_eq!(t, Duration::from_secs(0), "Some start returns t_next");
        assert_eq!(
            opt.as_ref().unwrap().int_calls,
            0,
            "Some starts with no internal transitions"
        );

        let mut output = Port::<usize, 1>::new();
        opt.lambda(&mut output, Duration::from_secs(0));
        assert_eq!(output.as_slice(), &[99], "Some lambda produces output");

        let t = opt.delta(&mut Port::new(), &mut Port::new(), Duration::from_secs(0));
        assert_eq!(
            opt.as_ref().unwrap().int_calls,
            1,
            "Some delta triggers transition"
        );
        assert!(t > Duration::from_secs(0), "Some delta returns next time");

        opt.stop();
    }

    #[test]
    fn option_none_start_infinity() {
        let mut opt: Option<Simulator<TestAtomic>> = None;
        let t = opt.start();
        assert_eq!(t, Duration::MAX, "None start returns Duration::MAX");
    }

    #[test]
    fn option_none_lambda_noop() {
        let mut opt: Option<Simulator<TestAtomic>> = None;
        let mut output = Port::<usize, 1>::new();
        opt.lambda(&mut output, Duration::from_secs(0));
        assert!(output.is_empty(), "None lambda leaves output unchanged");
    }

    #[test]
    fn option_none_delta_clears_input() {
        let mut opt: Option<Simulator<TestAtomic>> = None;
        let mut input = Port::<usize, 1>::new();
        input.add_value(99).unwrap();
        let mut output = Port::<usize, 1>::new();
        let t = opt.delta(&mut input, &mut output, Duration::from_secs(0));
        assert!(input.is_empty(), "None delta clears input");
        assert_eq!(t, Duration::MAX, "None delta returns Duration::MAX");
    }

    #[test]
    fn option_none_stop_noop() {
        let mut opt: Option<Simulator<TestAtomic>> = None;
        opt.stop();
        // No panic = pass
    }

    #[test]
    fn tuple_start_returns_min() {
        let mut tup = (
            TestAtomic::oneshot(Duration::from_secs(3)),
            TestAtomic::oneshot(Duration::from_secs(1)),
        )
            .to_simulator();
        assert_eq!(tup.start(), Duration::from_secs(1), "min of 3, 1 is 1");
    }

    #[test]
    fn tuple_lambda_iterates_all() {
        let mut tup = (
            TestAtomic::periodic(Duration::from_secs(0), Duration::from_secs(1)),
            TestAtomic::periodic(Duration::from_secs(0), Duration::from_secs(1)),
        )
            .to_simulator();
        tup.start();
        let mut out = (Port::<usize, 1>::new(), Port::<usize, 1>::new());
        tup.lambda(&mut out, Duration::from_secs(0));
        assert_eq!(out.0.as_slice(), &[99], "lambda on tuple[0]");
        assert_eq!(out.1.as_slice(), &[99], "lambda on tuple[1]");
    }

    #[test]
    fn tuple_delta_iterates_all() {
        let mut tup = (
            TestAtomic::periodic(Duration::from_secs(0), Duration::from_secs(1)),
            TestAtomic::periodic(Duration::from_secs(0), Duration::from_secs(1)),
        )
            .to_simulator();
        tup.start();
        let t = tup.delta(
            &mut (Port::new(), Port::new()),
            &mut (Port::new(), Port::new()),
            Duration::from_secs(0),
        );
        assert!(t > Duration::from_secs(0), "delta on tuple returns t_next");
    }

    #[test]
    fn tuple_stop_iterates_all() {
        let mut tup = (
            TestAtomic::periodic(Duration::from_secs(0), Duration::from_secs(1)),
            TestAtomic::periodic(Duration::from_secs(0), Duration::from_secs(1)),
        )
            .to_simulator();
        tup.start();
        tup.stop();
        // No panic = pass
    }

    #[test]
    fn ref_mut_delegates_abstract_simulator() {
        let mut raw = TestAtomic::oneshot(Duration::from_secs(5)).to_simulator();
        let t = <&mut Simulator<TestAtomic> as AbstractSimulator>::start(&mut &mut raw);
        assert_eq!(t, Duration::from_secs(5), "start delegates through &mut T");
        <&mut Simulator<TestAtomic> as AbstractSimulator>::stop(&mut &mut raw);
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn box_delegates_abstract_simulator() {
        let mut raw =
            alloc::boxed::Box::new(TestAtomic::oneshot(Duration::from_secs(3)).to_simulator());
        let t = <alloc::boxed::Box<Simulator<TestAtomic>> as AbstractSimulator>::start(&mut raw);
        assert_eq!(t, Duration::from_secs(3), "start delegates through Box<T>");
        <alloc::boxed::Box<Simulator<TestAtomic>> as AbstractSimulator>::stop(&mut raw);
    }

    #[test]
    fn simulate_vt_coupled() {
        let a0 = TestAtomic::oneshot(Duration::from_secs(1));
        let a1 = TestAtomic::oneshot(Duration::MAX); // passive, expects external
        let model = TestCoupled::build(a0, a1);
        let mut coord = model.to_simulator();
        let config = Config::new(Duration::from_secs(5), 1, None);
        coord.simulate_vt(&config);

        let comps = <TestCoupled as PartialCoupled>::get_components(&coord);
        assert_eq!(comps.a0.int_calls, 1, "atomic[0] fires once");
        assert_eq!(
            comps.a1.ext_calls, 1,
            "atomic[1] receives external from a0's lambda"
        );
    }

    #[test]
    fn simulate_vt_with_option_none() {
        let a0 = TestAtomic::oneshot(Duration::from_secs(1));
        let model = TestCoupledWithOption::build(a0, None);
        let mut coord = model.to_simulator();
        let config = Config::new(Duration::from_secs(3), 1, None);
        coord.simulate_vt(&config);

        let comps = <TestCoupledWithOption as PartialCoupled>::get_components(&coord);
        assert_eq!(comps.a0.int_calls, 1, "atomic[0] fires");
        assert!(comps.opt.is_none(), "optional component is None");
    }

    #[test]
    fn simulate_vt_with_array() {
        // Coupled model with array of atomics
        use crate::{couple, ComponentsInput, ComponentsOutput, Coupled, CoupledKind};

        #[crate::to_component]
        struct ArrayCoupledComponents {
            inner: [TestAtomic; 3],
        }

        struct ArrayCoupled {
            components: ArrayCoupledComponents,
        }

        impl Component for ArrayCoupled {
            type Kind = CoupledKind;
            type Input = Port<usize, 1>;
            type Output = Port<usize, 1>;
        }

        impl crate::component::coupled::PartialCoupled for ArrayCoupled {
            type Components = ArrayCoupledComponents;
            fn get_components(&self) -> &Self::Components {
                &self.components
            }
            fn get_components_mut(&mut self) -> &mut Self::Components {
                &mut self.components
            }
        }

        impl Coupled for ArrayCoupled {
            fn eic(from: &Self::Input, to: &mut ComponentsInput<Self>) {
                let _ = couple(from, &mut to.inner[0]);
            }
            fn ic(from: &ComponentsOutput<Self>, to: &mut ComponentsInput<Self>) {
                let _ = couple(&from.inner[0], &mut to.inner[1]);
                let _ = couple(&from.inner[1], &mut to.inner[2]);
            }
        }

        let a0 = TestAtomic::periodic(Duration::from_secs(0), Duration::from_secs(2));
        let a1 = TestAtomic::oneshot(Duration::MAX);
        let a2 = TestAtomic::oneshot(Duration::MAX);
        let model = ArrayCoupled {
            components: ArrayCoupledComponents {
                inner: [a0.to_simulator(), a1.to_simulator(), a2.to_simulator()],
            },
        };

        let comps =
            <ArrayCoupled as crate::component::coupled::PartialCoupled>::get_components(&model);
        assert_eq!(
            comps as *const _ as usize,
            &model.components as *const _ as usize
        );

        let mut coord = model.to_simulator();
        let config = Config::new(Duration::from_secs(5), 1, None);
        coord.simulate_vt(&config);

        let arr = &coord.components.inner;
        assert_eq!(arr[0].int_calls, 3, "a0 fires 3 times (t=0,2,4)");
        assert_eq!(
            arr[1].ext_calls, 3,
            "a1 receives external from a0 each time"
        );
        assert_eq!(
            arr[2].ext_calls, 3,
            "a2 receives external from a1 each time"
        );
    }

    #[test]
    fn config_default() {
        let c = Config::default();
        assert_eq!(c.duration, Duration::MAX);
        assert_eq!(c.mult, 1);
        assert!(c.max_jitter.is_none());
    }

    #[test]
    fn config_custom() {
        let c = Config::new(Duration::from_secs(10), 2, Some(Duration::from_millis(100)));
        assert_eq!(c.duration, Duration::from_secs(10));
        assert_eq!(c.mult, 2);
        assert_eq!(c.max_jitter, Some(Duration::from_millis(100)));
    }
}
