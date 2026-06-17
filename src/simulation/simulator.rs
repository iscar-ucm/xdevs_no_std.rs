use crate::{
    component::{atomic::Atomic, AtomicKind},
    port::Bag,
    simulation::{AbstractSimulator, Simulable},
};
use core::ops::{Deref, DerefMut};

/// Processor that wraps a DEVS component and implements the logic for simulating it.
pub struct Simulator<T: Atomic> {
    pub(crate) component: T,
    pub(crate) t_last: f64,
    pub(crate) t_next: f64,
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
