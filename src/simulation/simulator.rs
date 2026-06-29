use crate::{
    component::{atomic::Atomic, AtomicKind},
    port::Bag,
    simulation::{AbstractSimulator, Simulable},
};
use core::ops::{Deref, DerefMut};

/// Processor that wraps a DEVS component and implements the logic for simulating it.
pub struct Simulator<T: Atomic> {
    component: T,
    t_last: f64,
    t_next: f64,
}

impl<T: Atomic> Simulator<T> {
    /// Creates a new processor for the given component.
    #[inline(always)]
    pub const fn new(component: T) -> Self {
        Self {
            component,
            t_last: f64::INFINITY,
            t_next: f64::INFINITY,
        }
    }
}

impl<T: Atomic> Deref for Simulator<T> {
    type Target = T;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.component
    }
}

impl<T: Atomic> DerefMut for Simulator<T> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.component
    }
}

// Atomic models can be simulated using a `Simulator` struct
impl<T: Atomic> Simulable<AtomicKind> for T {
    type Simulator = Simulator<T>;

    fn to_simulator(self) -> Self::Simulator {
        Simulator::new(self)
    }
}

unsafe impl<T: Atomic> AbstractSimulator for Simulator<T> {
    type Input = T::Input;

    type Output = T::Output;

    #[inline(always)]
    fn start(&mut self, t_start: f64) -> f64 {
        self.t_last = t_start;
        self.component.start();
        let t_next = t_start + self.component.ta();
        self.t_next = t_next;
        t_next
    }

    #[inline(always)]
    fn stop(&mut self) {
        self.component.stop();
    }

    #[inline(always)]
    fn lambda(&mut self, output: &mut Self::Output, t: f64) {
        if t >= self.t_next {
            self.component.lambda(output);
        }
    }

    #[inline(always)]
    fn delta(&mut self, input: &mut Self::Input, output: &mut Self::Output, t: f64) -> f64 {
        let t_next = self.t_next;
        if !input.is_empty() {
            if t >= t_next {
                self.component.delta_conf(input);
                output.clear();
            } else {
                let e = t - self.t_last;
                self.component.delta_ext(e, input);
            }
            input.clear();
        } else if t >= t_next {
            self.component.delta_int();
            output.clear();
        } else {
            return t_next;
        }
        let t_next = t + self.component.ta();
        self.t_last = t;
        self.t_next = t_next;
        t_next
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{port::Port, simulation::test_utils::TestAtomic};

    #[test]
    fn start_sets_timing() {
        let mut sim = Simulator::new(TestAtomic::oneshot(3.0));
        let t_next = sim.start(0.0);
        assert_eq!(sim.t_last, 0.0, "t_last = t_start");
        assert_eq!(sim.t_next, 3.0, "t_next = t_start + ta()");
        assert_eq!(t_next, 3.0, "start returns t_next");
    }

    #[test]
    fn stop_called() {
        let mut sim = Simulator::new(TestAtomic::oneshot(5.0));
        sim.start(0.0);
        sim.stop();
        // No panic = pass
    }

    #[test]
    fn lambda_called_on_internal() {
        let mut sim = Simulator::new(TestAtomic::oneshot(3.0));
        sim.start(0.0);
        let mut output = Port::<usize, 1>::new();
        sim.lambda(&mut output, 3.0);
        assert_eq!(output.get_values(), &[99], "lambda called at t = t_next");
    }

    #[test]
    fn lambda_not_called_before_internal() {
        let mut sim = Simulator::new(TestAtomic::oneshot(5.0));
        sim.start(0.0);
        let mut output = Port::<usize, 1>::new();
        sim.lambda(&mut output, 2.0);
        assert!(output.is_empty(), "lambda skipped before t_next");
    }

    #[test]
    fn delta_internal_transition() {
        let mut sim = Simulator::new(TestAtomic::periodic(0.0, 2.0));
        sim.start(0.0);
        let mut output = Port::<usize, 1>::new();
        output.add_value(99).unwrap();
        sim.delta(&mut Port::new(), &mut output, 0.0);
        assert_eq!(sim.component.int_calls, 1, "delta_int called");
        assert!(output.is_empty(), "output cleared after delta_int");
    }

    #[test]
    fn delta_external_transition() {
        let mut sim = Simulator::new(TestAtomic::oneshot(5.0));
        sim.start(0.0);
        let mut input = Port::<usize, 1>::new();
        input.add_value(99).unwrap();
        let mut output = Port::<usize, 1>::new();
        sim.delta(&mut input, &mut output, 2.0);
        assert_eq!(sim.component.ext_calls, 1, "delta_ext called");
        assert_eq!(sim.component.last_elapsed, 2.0, "elapsed = t - t_last");
        assert!(input.is_empty(), "input cleared after delta_ext");
    }

    #[test]
    fn delta_confluent_transition() {
        let mut sim = Simulator::new(TestAtomic::periodic(0.0, 5.0));
        sim.start(0.0);
        let mut input = Port::<usize, 1>::new();
        input.add_value(99).unwrap();
        let mut output = Port::<usize, 1>::new();
        output.add_value(99).unwrap();
        sim.delta(&mut input, &mut output, 0.0);
        assert_eq!(
            sim.component.int_calls, 1,
            "delta_int called (via delta_conf)"
        );
        assert_eq!(
            sim.component.ext_calls, 1,
            "delta_ext called (via delta_conf)"
        );
        assert!(input.is_empty(), "input cleared after delta_conf");
        assert!(output.is_empty(), "output cleared after delta_conf");
    }

    #[test]
    fn delta_no_transition() {
        let mut sim = Simulator::new(TestAtomic::oneshot(5.0));
        sim.start(0.0);
        let t_next = sim.delta(&mut Port::new(), &mut Port::new(), 2.0);
        assert_eq!(t_next, 5.0, "returns unchanged t_next");
        assert_eq!(sim.component.int_calls, 0, "no delta_int");
        assert_eq!(sim.component.ext_calls, 0, "no delta_ext");
    }

    #[test]
    fn delta_updates_timing() {
        let mut sim = Simulator::new(TestAtomic::periodic(0.0, 3.0));
        sim.start(0.0);
        sim.delta(&mut Port::new(), &mut Port::new(), 0.0);
        assert_eq!(sim.t_last, 0.0, "t_last = t");
        assert_eq!(sim.t_next, 3.0, "t_next = t + ta() (= period)");
    }
}
