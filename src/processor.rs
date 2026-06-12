use core::ops::{Deref, DerefMut};

use crate::{
    traits::{AbstractSimulator, AsProcessor, Bag},
    Atomic, AtomicKind, Component, Coupled, CoupledKind,
};

/// Processor that wraps a DEVS component and implements the logic for simulating it.
pub struct Processor<T: Component> {
    pub(crate) component: T,
    t_last: f64,
    t_next: f64,
}

impl<T: Component> Processor<T> {
    /// Creates a new processor for the given component.
    pub const fn new(component: T) -> Self {
        Self {
            component,
            t_last: f64::INFINITY,
            t_next: f64::INFINITY,
        }
    }
}

impl<T: Component> Component for Processor<T> {
    type Input = T::Input;
    type Output = T::Output;
    type Kind = T::Kind;
}

unsafe impl<T, K> AsProcessor for Processor<T>
where
    T: Component<Kind = K> + AbstractSimulator<K>,
{
    #[inline(always)]
    fn starts(&mut self, t_start: f64) -> f64 {
        <T as AbstractSimulator<K>>::start(self, t_start)
    }

    #[inline(always)]
    fn stops(&mut self) {
        <T as AbstractSimulator<K>>::stop(self)
    }

    #[inline(always)]
    fn lambdas(&mut self, output: &mut T::Output, t: f64) {
        <T as AbstractSimulator<K>>::lambda(self, output, t)
    }

    #[inline(always)]
    fn deltas(&mut self, input: &mut T::Input, output: &mut T::Output, t: f64) -> f64 {
        <T as AbstractSimulator<K>>::delta(self, input, output, t)
    }
}

unsafe impl<T: Atomic> AbstractSimulator<AtomicKind> for T {
    #[inline(always)]
    fn start(processor: &mut Processor<Self>, t_start: f64) -> f64 {
        processor.t_last = t_start;
        processor.component.start();
        let t_next = t_start + processor.component.ta();
        processor.t_next = t_next;
        t_next
    }
    #[inline(always)]
    fn stop(processor: &mut Processor<Self>) {
        processor.component.stop();
    }
    #[inline(always)]
    fn lambda(processor: &mut Processor<Self>, output: &mut Self::Output, t: f64) {
        if t >= processor.t_next {
            processor.component.lambda(output);
        }
    }
    #[inline(always)]
    fn delta(
        processor: &mut Processor<Self>,
        input: &mut Self::Input,
        output: &mut Self::Output,
        t: f64,
    ) -> f64 {
        let t_next = processor.t_next;
        if !input.is_empty() {
            if t >= t_next {
                processor.component.delta_conf(input);
                output.clear();
            } else {
                let e = t - processor.t_last;
                processor.component.delta_ext(e, input);
            }
            input.clear();
        } else if t >= t_next {
            processor.component.delta_int();
            output.clear();
        } else {
            return t_next;
        }
        let t_next = t + processor.component.ta();
        processor.t_last = t;
        processor.t_next = t_next;
        t_next
    }
}

unsafe impl<T: Coupled> AbstractSimulator<CoupledKind> for T {
    #[inline(always)]
    fn start(processor: &mut Processor<Self>, t_start: f64) -> f64 {
        processor.t_last = t_start;
        let t_next = processor.component.components().starts(t_start);
        processor.t_next = t_next;
        t_next
    }
    #[inline(always)]
    fn stop(processor: &mut Processor<Self>) {
        processor.component.components().stops();
    }
    #[inline(always)]
    fn lambda(processor: &mut Processor<Self>, output: &mut Self::Output, t: f64) {
        if t >= processor.t_next {
            let (components, _, outputs) = processor.component.split();
            components.lambdas(outputs, t);
            Self::eoc(outputs, output);
        }
    }
    #[inline(always)]
    fn delta(
        processor: &mut Processor<Self>,
        input: &mut Self::Input,
        output: &mut Self::Output,
        t: f64,
    ) -> f64 {
        let t_next = processor.t_next;
        if t < t_next && input.is_empty() {
            return t_next;
        }
        let (components, inputs, outputs) = processor.component.split();
        Self::eic(input, inputs);
        Self::ic(outputs, inputs);
        let t_next = components.deltas(inputs, outputs, t);
        processor.t_last = t;
        processor.t_next = t_next;

        input.clear();
        output.clear();

        t_next
    }
}

impl<T: Component> Deref for Processor<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.component
    }
}
impl<T: Component> DerefMut for Processor<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.component
    }
}
