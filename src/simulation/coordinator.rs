use crate::{
    component::{
        coupled::{ComponentsInput, ComponentsOutput, Coupled},
        CoupledKind,
    },
    port::Bag,
    simulation::{AbstractSimulator, Simulable},
};
use core::ops::{Deref, DerefMut};

/// Coordinator that encapsulates coupled-model simulation state.
pub struct Coordinator<T: Coupled> {
    component: T,
    components_input: ComponentsInput<T>,
    components_output: ComponentsOutput<T>,
    t_next: f64,
}

impl<T: Coupled> Coordinator<T> {
    /// Creates a new coordinator for the given coupled model.
    #[inline(always)]
    pub fn new(component: T) -> Self {
        Self {
            component,
            components_input: ComponentsInput::<T>::build(),
            components_output: ComponentsOutput::<T>::build(),
            t_next: f64::INFINITY,
        }
    }
}

impl<T: Coupled> Deref for Coordinator<T> {
    type Target = T;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.component
    }
}

impl<T: Coupled> DerefMut for Coordinator<T> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.component
    }
}

// Coupled models can be simulated using a `Coupled` struct
impl<T: Coupled> Simulable<CoupledKind> for T {
    type Simulator = Coordinator<T>;

    fn to_simulator(self) -> Self::Simulator {
        Coordinator::new(self)
    }
}

unsafe impl<T: Coupled> AbstractSimulator for Coordinator<T> {
    type Input = T::Input;
    type Output = T::Output;

    #[inline(always)]
    fn start(&mut self, t_start: f64) -> f64 {
        let t_next = self.component.get_components_mut().start(t_start);
        self.t_next = t_next;
        t_next
    }

    #[inline(always)]
    fn stop(&mut self) {
        self.component.get_components_mut().stop();
    }

    #[inline(always)]
    fn lambda(&mut self, output: &mut Self::Output, t: f64) {
        if t >= self.t_next {
            self.component
                .get_components_mut()
                .lambda(&mut self.components_output, t);
            T::eoc(&self.components_output, output);
        }
    }

    #[inline(always)]
    fn delta(&mut self, input: &mut Self::Input, output: &mut Self::Output, t: f64) -> f64 {
        let t_next = self.t_next;
        if t < t_next && input.is_empty() {
            return t_next;
        }

        T::eic(input, &mut self.components_input);
        T::ic(&self.components_output, &mut self.components_input);
        let t_next = self.component.get_components_mut().delta(
            &mut self.components_input,
            &mut self.components_output,
            t,
        );

        self.t_next = t_next;

        input.clear();
        output.clear();

        t_next
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        component::coupled::PartialCoupled,
        port::Port,
        simulation::test_utils::{TestAtomic, TestCoupled},
    };

    #[test]
    fn start_delegates() {
        let a0 = TestAtomic::oneshot(3.0);
        let a1 = TestAtomic::oneshot(7.0);
        let model = TestCoupled::build(a0, a1);
        let mut coord = Coordinator::new(model);
        let t = coord.start(0.0);
        assert_eq!(t, 3.0, "start returns min t_next");
    }

    #[test]
    fn stop_called() {
        let model = TestCoupled::build(TestAtomic::oneshot(1.0), TestAtomic::oneshot(1.0));
        let mut coord = Coordinator::new(model);
        coord.start(0.0);
        coord.stop();
        // No panic = pass
    }

    #[test]
    fn lambda_calls_eoc() {
        let model = TestCoupled::build(
            TestAtomic::periodic(0.0, 1.0),
            TestAtomic::periodic(0.0, 1.0),
        );
        let mut coord = Coordinator::new(model);
        coord.start(0.0);
        let mut output = Port::<usize, 1>::new();
        coord.lambda(&mut output, 0.0);
        assert_eq!(output.get_values(), &[99], "eoc copies a1 output");
    }

    #[test]
    fn lambda_noop_before_internal() {
        let model = TestCoupled::build(
            TestAtomic::oneshot(f64::INFINITY),
            TestAtomic::oneshot(f64::INFINITY),
        );
        let mut coord = Coordinator::new(model);
        coord.start(0.0);
        let mut output = Port::<usize, 1>::new();
        coord.lambda(&mut output, 0.0);
        assert!(output.is_empty(), "lambda no-op before t_next");
    }

    #[test]
    fn delta_early_return() {
        let model = TestCoupled::build(
            TestAtomic::oneshot(f64::INFINITY),
            TestAtomic::oneshot(f64::INFINITY),
        );
        let mut coord = Coordinator::new(model);
        coord.start(0.0);
        let t = coord.delta(&mut Port::new(), &mut Port::new(), 0.0);
        assert_eq!(t, f64::INFINITY, "early return when no work");
    }

    #[test]
    fn delta_eic_propagation() {
        let a0 = TestAtomic::oneshot(f64::INFINITY);
        let a1 = TestAtomic::oneshot(f64::INFINITY);
        let model = TestCoupled::build(a0, a1);
        let mut coord = Coordinator::new(model);
        coord.start(0.0);

        let mut input = Port::<usize, 1>::new();
        input.add_value(99).unwrap();
        coord.delta(&mut input, &mut Port::new(), 3.0);

        let comps = <TestCoupled as PartialCoupled>::get_components(&coord);
        assert_eq!(comps.a0.ext_calls, 1, "eic copies external input to a0");
        assert_eq!(comps.a0.last_elapsed, 3.0, "elapsed = t - t_last");
        assert_eq!(
            comps.a1.ext_calls, 0,
            "a1 receives nothing (ic from empty output)"
        );
    }

    #[test]
    fn delta_ic_propagation() {
        // a0 fires immediately, a1 is passive, ic copies a0's output to a1
        let a0 = TestAtomic::periodic(0.0, 2.0);
        let a1 = TestAtomic::oneshot(f64::INFINITY);
        let model = TestCoupled::build(a0, a1);
        let mut coord = Coordinator::new(model);
        coord.start(0.0);

        // Lambda: a0 writes 99 to components_output[0]
        coord.lambda(&mut Port::new(), 0.0);
        // Delta: ic copies components_output[0] → components_input[1] → a1 delta_ext
        coord.delta(&mut Port::new(), &mut Port::new(), 0.0);

        let comps = <TestCoupled as PartialCoupled>::get_components(&coord);
        assert_eq!(comps.a1.ext_calls, 1, "ic routes a0's output to a1's input");
    }
}
