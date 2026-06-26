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
/// `simulate_rt_async`).
///
/// # Safety
///
/// This trait must be implemented internally or via the [`to_component`](crate::to_component) macro. Do not implement it manually.
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

unsafe impl<T: AbstractSimulator> AbstractSimulator for Option<T> {
    type Input = T::Input;
    type Output = T::Output;

    #[inline(always)]
    fn start(&mut self, t_start: f64) -> f64 {
        match self {
            Some(processor) => T::start(processor, t_start),
            None => f64::INFINITY,
        }
    }

    #[inline(always)]
    fn stop(&mut self) {
        if let Some(processor) = self {
            T::stop(processor);
        }
    }

    #[inline(always)]
    fn lambda(&mut self, output: &mut Self::Output, t: f64) {
        if let Some(processor) = self {
            T::lambda(processor, output, t);
        }
    }

    #[inline(always)]
    fn delta(&mut self, input: &mut Self::Input, output: &mut Self::Output, t: f64) -> f64 {
        match self {
            Some(processor) => T::delta(processor, input, output, t),
            None => {
                input.clear();
                f64::INFINITY
            }
        }
    }
}

macro_rules! impl_abstract_simulator_for_tuple {
    ($($idx:tt => $T:ident),+) => {
        unsafe impl<$($T: AbstractSimulator),+> AbstractSimulator for ($($T,)+) {
            type Input = ($($T::Input,)+);
            type Output = ($($T::Output,)+);

            #[inline(always)]
            fn start(&mut self, t_start: f64) -> f64 {
                let mut min_t = f64::INFINITY;
                $(min_t = min_t.min(self.$idx.start(t_start));)+
                min_t
            }

            #[inline(always)]
            fn stop(&mut self) {
                $(self.$idx.stop();)+
            }

            #[inline(always)]
            fn lambda(&mut self, output: &mut Self::Output, t: f64) {
                $(self.$idx.lambda(&mut output.$idx, t);)+
            }

            #[inline(always)]
            fn delta(&mut self, input: &mut Self::Input, output: &mut Self::Output, t: f64) -> f64 {
                let mut min_t = f64::INFINITY;
                $(min_t = min_t.min(self.$idx.delta(&mut input.$idx, &mut output.$idx, t));)+
                min_t
            }
        }
    }
}

impl_abstract_simulator_for_tuple!(0 => T0);
impl_abstract_simulator_for_tuple!(0 => T0, 1 => T1);
impl_abstract_simulator_for_tuple!(0 => T0, 1 => T1, 2 => T2);
impl_abstract_simulator_for_tuple!(0 => T0, 1 => T1, 2 => T2, 3 => T3);
impl_abstract_simulator_for_tuple!(0 => T0, 1 => T1, 2 => T2, 3 => T3, 4 => T4);
impl_abstract_simulator_for_tuple!(0 => T0, 1 => T1, 2 => T2, 3 => T3, 4 => T4, 5 => T5);
impl_abstract_simulator_for_tuple!(0 => T0, 1 => T1, 2 => T2, 3 => T3, 4 => T4, 5 => T5, 6 => T6);
impl_abstract_simulator_for_tuple!(0 => T0, 1 => T1, 2 => T2, 3 => T3, 4 => T4, 5 => T5, 6 => T6, 7 => T7);
impl_abstract_simulator_for_tuple!(0 => T0, 1 => T1, 2 => T2, 3 => T3, 4 => T4, 5 => T5, 6 => T6, 7 => T7, 8 => T8);
impl_abstract_simulator_for_tuple!(0 => T0, 1 => T1, 2 => T2, 3 => T3, 4 => T4, 5 => T5, 6 => T6, 7 => T7, 8 => T8, 9 => T9);
impl_abstract_simulator_for_tuple!(0 => T0, 1 => T1, 2 => T2, 3 => T3, 4 => T4, 5 => T5, 6 => T6, 7 => T7, 8 => T8, 9 => T9, 10 => T10);
impl_abstract_simulator_for_tuple!(0 => T0, 1 => T1, 2 => T2, 3 => T3, 4 => T4, 5 => T5, 6 => T6, 7 => T7, 8 => T8, 9 => T9, 10 => T10, 11 => T11);

macro_rules! impl_simulable_for_tuple {
    ($($idx:tt => ($T:ident, $K:ident)),+) => {
        impl<$($T, $K),+> Simulable<($($K,)+)> for ($($T,)+)
        where
            $($T: Component<Kind = $K> + Simulable<$K>),+,
            $($K: crate::component::sealed::Sealed),+
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

// Module with models for simulation, simulator and coordinator testing
#[cfg(test)]
pub(crate) mod test_utils {
    use crate::{
        component::coupled::{ComponentsInput, ComponentsOutput, Coupled},
        component::CoupledKind,
        port::Port,
        Atomic, AtomicKind, Component,
    };

    pub(crate) struct TestAtomic {
        pub sigma: f64,
        pub period: f64,
        pub int_calls: usize,
        pub ext_calls: usize,
        pub last_elapsed: f64,
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
        fn delta_ext(&mut self, elapsed: f64, _input: &Self::Input) {
            self.ext_calls += 1;
            self.last_elapsed = elapsed;
            self.sigma = 0.0;
        }
        fn lambda(&self, output: &mut Self::Output) {
            let _ = output.add_value(self.out_val);
        }
        fn ta(&self) -> f64 {
            self.sigma
        }
    }

    impl TestAtomic {
        pub(crate) fn periodic(sigma: f64, period: f64) -> Self {
            Self {
                sigma,
                period,
                int_calls: 0,
                ext_calls: 0,
                last_elapsed: 0.0,
                out_val: 99,
            }
        }
        pub(crate) fn oneshot(sigma: f64) -> Self {
            Self::periodic(sigma, f64::INFINITY)
        }
    }

    #[crate::to_component]
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
            let _ = from.couple(&mut to.a0);
        }
        fn ic(from: &ComponentsOutput<Self>, to: &mut ComponentsInput<Self>) {
            let _ = from.a0.couple(&mut to.a1);
        }
        fn eoc(from: &ComponentsOutput<Self>, to: &mut Self::Output) {
            let _ = from.a1.couple(to);
        }
    }

    #[crate::to_component]
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
            let _ = from.couple(&mut to.a0);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::test_utils::{TestAtomic, TestCoupled, TestCoupledWithOption};
    use crate::{
        component::coupled::PartialCoupled,
        port::Port,
        simulation::{simulator::Simulator, AbstractSimulator, Config, Simulable},
        Component,
    };
    #[test]
    fn simulate_vt_single_event() {
        let mut sim = TestAtomic::oneshot(5.0).to_simulator();
        let config = Config::new(0.0, 20.0, 1.0, None);
        sim.simulate_vt(&config);

        assert_eq!(sim.int_calls, 1, "one internal transition");
        assert_eq!(sim.ext_calls, 0, "no external transitions");
    }

    #[test]
    fn simulate_vt_multiple_events() {
        let mut sim = TestAtomic::periodic(0.0, 2.0).to_simulator();
        let config = Config::new(0.0, 9.0, 1.0, None);
        sim.simulate_vt(&config);

        assert_eq!(sim.int_calls, 5, "expected 5 internal transitions in 9s");
        assert_eq!(sim.ext_calls, 0, "no external transitions");
    }

    #[test]
    fn simulate_vt_no_spurious_transitions() {
        let mut sim = TestAtomic::oneshot(f64::INFINITY).to_simulator();
        let config = Config::new(0.0, 10.0, 1.0, None);
        sim.simulate_vt(&config);

        assert_eq!(sim.int_calls, 0, "no internal events");
        assert_eq!(sim.ext_calls, 0, "no external events");
    }

    #[test]
    fn simulate_rt_single_event() {
        let mut sim = TestAtomic::oneshot(5.0).to_simulator();
        let config = Config::new(0.0, 10.0, 1.0, None);
        sim.simulate_rt(&config, |_, t_until, _| t_until, |_| {});
        assert_eq!(sim.int_calls, 1, "rt single event");
        assert_eq!(sim.ext_calls, 0, "no external transitions");
    }

    #[test]
    fn simulate_rt_injects_external_input() {
        // wait_until injects input at t=0 before the first internal (t=5)
        // external transition triggers. external transition sets sigma=0,
        // so an immediate internal transition follows.
        let mut sim = TestAtomic::oneshot(5.0).to_simulator();
        let config = Config::new(0.0, 10.0, 1.0, None);

        let mut injected = false;
        sim.simulate_rt(
            &config,
            |t, t_until, input| {
                if !injected {
                    injected = true;
                    input.add_value(99).unwrap();
                    t // inject at current time, return same t to process
                } else {
                    t_until // proceed normally afterwards
                }
            },
            |_| {},
        );

        assert_eq!(sim.ext_calls, 1, "external transition via wait_until");
    }

    #[test]
    fn simulate_rt_propagate_output() {
        let mut sim = TestAtomic::oneshot(5.0).to_simulator();
        let config = Config::new(0.0, 10.0, 1.0, None);
        let mut captured = Port::<usize, 1>::new();

        sim.simulate_rt(
            &config,
            |_, t_until, _| t_until,
            |output| {
                for v in output.get_values() {
                    captured.add_value(*v).unwrap();
                }
            },
        );

        assert_eq!(
            captured.get_values(),
            &[99],
            "propagate_output captures lambda output"
        );
    }

    struct IdentityAsyncInput;

    impl crate::simulation::AsyncInput for IdentityAsyncInput {
        type Input = Port<usize, 1>;
        async fn handle(
            &mut self,
            _config: &Config,
            _t_from: f64,
            t_until: f64,
            _input: &mut Self::Input,
        ) -> f64 {
            t_until
        }
    }

    #[tokio::test]
    async fn simulate_rt_async_single_event() {
        let mut sim = TestAtomic::oneshot(5.0).to_simulator();
        let config = Config::new(0.0, 10.0, 1.0, None);
        sim.simulate_rt_async(&config, IdentityAsyncInput, |_| {})
            .await;
        assert_eq!(sim.int_calls, 1, "async single event");
        assert_eq!(sim.ext_calls, 0, "no external transitions");
    }

    #[tokio::test]
    async fn simulate_rt_async_external_input() {
        let mut sim = TestAtomic::oneshot(5.0).to_simulator();
        let config = Config::new(0.0, 10.0, 1.0, None);

        struct InjectInput {
            injected: bool,
        }
        impl crate::simulation::AsyncInput for InjectInput {
            type Input = Port<usize, 1>;
            async fn handle(
                &mut self,
                _config: &Config,
                t_from: f64,
                t_until: f64,
                input: &mut Self::Input,
            ) -> f64 {
                if !self.injected {
                    self.injected = true;
                    input.add_value(99).unwrap();
                    t_from
                } else {
                    t_until
                }
            }
        }

        sim.simulate_rt_async(&config, InjectInput { injected: false }, |_| {})
            .await;
        assert_eq!(sim.ext_calls, 1, "async external input");
    }

    #[tokio::test]
    async fn simulate_rt_async_propagate_output() {
        let mut sim = TestAtomic::oneshot(5.0).to_simulator();
        let config = Config::new(0.0, 10.0, 1.0, None);
        let mut captured = Port::<usize, 1>::new();

        sim.simulate_rt_async(&config, IdentityAsyncInput, |output| {
            for v in output.get_values() {
                let _ = captured.add_value(*v);
            }
        })
        .await;

        assert_eq!(captured.get_values(), &[99], "async propagate_output");
    }

    #[test]
    fn array_start_returns_min() {
        let a0 = TestAtomic::oneshot(3.0);
        let a1 = TestAtomic::oneshot(1.0);
        let a2 = TestAtomic::oneshot(5.0);
        let mut arr = [a0, a1, a2].to_simulator();

        let t = arr.start(0.0);
        assert_eq!(t, 1.0, "min of 3, 1, 5 is 1.0");
    }

    #[test]
    fn array_lambda_iterates_all() {
        let mut arr = [
            TestAtomic::periodic(0.0, 1.0),
            TestAtomic::periodic(0.0, 1.0),
        ]
        .to_simulator();
        arr.start(0.0);

        let mut output = [Port::<usize, 1>::new(), Port::<usize, 1>::new()];
        arr.lambda(&mut output, 0.0);

        assert_eq!(output[0].get_values(), &[99], "first atomic lambda ran");
        assert_eq!(output[1].get_values(), &[99], "second atomic lambda ran");
    }

    #[test]
    fn array_delta_iterates_all() {
        let mut arr = [
            TestAtomic::periodic(0.0, 1.0),
            TestAtomic::periodic(0.0, 1.0),
        ]
        .to_simulator();
        arr.start(0.0);

        let mut input = [Port::<usize, 1>::new(), Port::<usize, 1>::new()];
        let mut output = [Port::<usize, 1>::new(), Port::<usize, 1>::new()];
        let t = arr.delta(&mut input, &mut output, 0.0);

        assert_eq!(arr[0].int_calls, 1, "first atomic delta_int");
        assert_eq!(arr[1].int_calls, 1, "second atomic delta_int");
        assert!(t > 0.0, "t_next should be > 0");
    }

    #[test]
    fn array_stop_iterates_all() {
        let mut arr = [
            TestAtomic::periodic(0.0, 1.0),
            TestAtomic::periodic(0.0, 1.0),
        ]
        .to_simulator();
        arr.start(0.0);
        arr.stop();

        assert_eq!(arr[0].ext_calls, 0, "stop on first array element");
        assert_eq!(arr[1].ext_calls, 0, "stop on second array element");
    }

    #[test]
    fn option_some_delegates() {
        let mut opt = Some(TestAtomic::periodic(0.0, 1.0)).to_simulator();

        let t = opt.start(0.0);
        assert_eq!(t, 0.0, "Some start returns t_next");
        assert_eq!(
            opt.as_ref().unwrap().int_calls,
            0,
            "Some starts with no internal transitions"
        );

        let mut output = Port::<usize, 1>::new();
        opt.lambda(&mut output, 0.0);
        assert_eq!(output.get_values(), &[99], "Some lambda produces output");

        let t = opt.delta(&mut Port::new(), &mut Port::new(), 0.0);
        assert_eq!(
            opt.as_ref().unwrap().int_calls,
            1,
            "Some delta triggers transition"
        );
        assert!(t > 0.0, "Some delta returns next time");

        opt.stop();
    }

    #[test]
    fn option_none_start_infinity() {
        let mut opt: Option<Simulator<TestAtomic>> = None;
        let t = opt.start(0.0);
        assert_eq!(t, f64::INFINITY, "None start returns INFINITY");
    }

    #[test]
    fn option_none_lambda_noop() {
        let mut opt: Option<Simulator<TestAtomic>> = None;
        let mut output = Port::<usize, 1>::new();
        opt.lambda(&mut output, 0.0);
        assert!(output.is_empty(), "None lambda leaves output unchanged");
    }

    #[test]
    fn option_none_delta_clears_input() {
        let mut opt: Option<Simulator<TestAtomic>> = None;
        let mut input = Port::<usize, 1>::new();
        input.add_value(99).unwrap();
        let mut output = Port::<usize, 1>::new();
        let t = opt.delta(&mut input, &mut output, 0.0);
        assert!(input.is_empty(), "None delta clears input");
        assert_eq!(t, f64::INFINITY, "None delta returns INFINITY");
    }

    #[test]
    fn option_none_stop_noop() {
        let mut opt: Option<Simulator<TestAtomic>> = None;
        opt.stop();
        // No panic = pass
    }

    #[test]
    fn tuple_start_returns_min() {
        let mut tup = (TestAtomic::oneshot(3.0), TestAtomic::oneshot(1.0)).to_simulator();
        assert_eq!(tup.start(0.0), 1.0, "min of 3, 1 is 1.0");
    }

    #[test]
    fn tuple_lambda_iterates_all() {
        let mut tup = (
            TestAtomic::periodic(0.0, 1.0),
            TestAtomic::periodic(0.0, 1.0),
        )
            .to_simulator();
        tup.start(0.0);
        let mut out = (Port::<usize, 1>::new(), Port::<usize, 1>::new());
        tup.lambda(&mut out, 0.0);
        assert_eq!(out.0.get_values(), &[99], "lambda on tuple[0]");
        assert_eq!(out.1.get_values(), &[99], "lambda on tuple[1]");
    }

    #[test]
    fn tuple_delta_iterates_all() {
        let mut tup = (
            TestAtomic::periodic(0.0, 1.0),
            TestAtomic::periodic(0.0, 1.0),
        )
            .to_simulator();
        tup.start(0.0);
        let t = tup.delta(
            &mut (Port::new(), Port::new()),
            &mut (Port::new(), Port::new()),
            0.0,
        );
        assert!(t > 0.0, "delta on tuple returns t_next");
    }

    #[test]
    fn tuple_stop_iterates_all() {
        let mut tup = (
            TestAtomic::periodic(0.0, 1.0),
            TestAtomic::periodic(0.0, 1.0),
        )
            .to_simulator();
        tup.start(0.0);
        tup.stop();
        // No panic = pass
    }

    #[test]
    fn ref_mut_delegates_abstract_simulator() {
        let mut raw = TestAtomic::oneshot(5.0).to_simulator();
        let t = <&mut Simulator<TestAtomic> as AbstractSimulator>::start(&mut &mut raw, 0.0);
        assert_eq!(t, 5.0, "start delegates through &mut T");
        <&mut Simulator<TestAtomic> as AbstractSimulator>::stop(&mut &mut raw);
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn box_delegates_abstract_simulator() {
        let mut raw = alloc::boxed::Box::new(TestAtomic::oneshot(3.0).to_simulator());
        let t =
            <alloc::boxed::Box<Simulator<TestAtomic>> as AbstractSimulator>::start(&mut raw, 0.0);
        assert_eq!(t, 3.0, "start delegates through Box<T>");
        <alloc::boxed::Box<Simulator<TestAtomic>> as AbstractSimulator>::stop(&mut raw);
    }

    #[test]
    fn simulate_vt_coupled() {
        let a0 = TestAtomic::oneshot(1.0);
        let a1 = TestAtomic::oneshot(f64::INFINITY); // passive, expects external
        let model = TestCoupled::build(a0, a1);
        let mut coord = model.to_simulator();
        let config = Config::new(0.0, 5.0, 1.0, None);
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
        let a0 = TestAtomic::oneshot(1.0);
        let model = TestCoupledWithOption::build(a0, None);
        let mut coord = model.to_simulator();
        let config = Config::new(0.0, 3.0, 1.0, None);
        coord.simulate_vt(&config);

        let comps = <TestCoupledWithOption as PartialCoupled>::get_components(&coord);
        assert_eq!(comps.a0.int_calls, 1, "atomic[0] fires");
        assert!(comps.opt.is_none(), "optional component is None");
    }

    #[test]
    fn simulate_vt_with_array() {
        // Coupled model with array of atomics
        use crate::component::{
            coupled::{ComponentsInput, ComponentsOutput, Coupled},
            CoupledKind,
        };

        struct ArrayCoupled {
            components: [Simulator<TestAtomic>; 3],
        }

        impl Component for ArrayCoupled {
            type Kind = CoupledKind;
            type Input = Port<usize, 1>;
            type Output = Port<usize, 1>;
        }

        impl crate::component::coupled::PartialCoupled for ArrayCoupled {
            type Components = [TestAtomic; 3];
            fn get_components(&self) -> &crate::component::coupled::Processors<Self> {
                &self.components
            }
            fn get_components_mut(&mut self) -> &mut crate::component::coupled::Processors<Self> {
                &mut self.components
            }
        }

        impl Coupled for ArrayCoupled {
            fn eic(from: &Self::Input, to: &mut ComponentsInput<Self>) {
                let _ = from.couple(&mut to[0]);
            }
            fn ic(from: &ComponentsOutput<Self>, to: &mut ComponentsInput<Self>) {
                let _ = from[0].couple(&mut to[1]);
                let _ = from[1].couple(&mut to[2]);
            }
        }

        let a0 = TestAtomic::periodic(0.0, 2.0);
        let a1 = TestAtomic::oneshot(f64::INFINITY);
        let a2 = TestAtomic::oneshot(f64::INFINITY);
        let model = ArrayCoupled {
            components: [a0.to_simulator(), a1.to_simulator(), a2.to_simulator()],
        };

        let comps =
            <ArrayCoupled as crate::component::coupled::PartialCoupled>::get_components(&model);
        assert_eq!(
            comps as *const _ as usize,
            &model.components as *const _ as usize
        );

        let mut coord = model.to_simulator();
        let config = Config::new(0.0, 5.0, 1.0, None);
        coord.simulate_vt(&config);

        let arr = &coord.components;
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
        assert_eq!(c.t_start, 0.0);
        assert_eq!(c.t_stop, f64::INFINITY);
        assert_eq!(c.time_scale, 1.0);
        assert!(c.max_jitter.is_none());
    }

    #[test]
    fn config_custom() {
        let c = Config::new(1.0, 10.0, 2.0, Some(core::time::Duration::from_millis(100)));
        assert_eq!(c.t_start, 1.0);
        assert_eq!(c.t_stop, 10.0);
        assert_eq!(c.time_scale, 2.0);
        assert_eq!(c.max_jitter, Some(core::time::Duration::from_millis(100)));
    }
}
