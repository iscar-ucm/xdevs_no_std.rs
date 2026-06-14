use core::ops::{Deref, DerefMut};

use crate::{component::AbstractSimulator, Component};

/// Processor that wraps a DEVS component and implements the logic for simulating it.
pub struct Processor<T: Component> {
    pub(crate) component: T,
    pub(crate) t_last: f64,
    pub(crate) t_next: f64,
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

/// Interface for handling collections of DEVS models during simulation.
///
/// # Safety
///
/// This trait must be implemented via macros. Do not implement it manually.
pub unsafe trait AsProcessor: Component {
    fn starts(&mut self, t_start: f64) -> f64;
    fn stops(&mut self);
    fn lambdas(&mut self, output: &mut Self::Output, t: f64);
    fn deltas(&mut self, input: &mut Self::Input, output: &mut Self::Output, t: f64) -> f64;
}

unsafe impl<T: AsProcessor, const N: usize> AsProcessor for [T; N] {
    #[inline(always)]
    fn starts(&mut self, t_start: f64) -> f64 {
        self.iter_mut()
            .map(|component| component.starts(t_start))
            .fold(f64::INFINITY, f64::min)
    }

    #[inline(always)]
    fn stops(&mut self) {
        self.iter_mut().for_each(|component| component.stops());
    }

    #[inline(always)]
    fn lambdas(&mut self, output: &mut Self::Output, t: f64) {
        for (component, output_bag) in self.iter_mut().zip(output.iter_mut()) {
            component.lambdas(output_bag, t);
        }
    }

    #[inline(always)]
    fn deltas(&mut self, input: &mut Self::Input, output: &mut Self::Output, t: f64) -> f64 {
        self.iter_mut()
            .zip(input.iter_mut())
            .zip(output.iter_mut())
            .map(|((component, input_bag), output_bag)| component.deltas(input_bag, output_bag, t))
            .fold(f64::INFINITY, f64::min)
    }
}
