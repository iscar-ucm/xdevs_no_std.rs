//! Blanket implementations of the traits in `traits` module.
#[cfg(any(feature = "std", feature = "alloc"))]
extern crate alloc;

use crate::{
    traits::{sealed::Sealed, AsPort, AsProcessor, Bag, PartialCoupled},
    Component, Coupled,
};

//////////////////////////////////////////////// Arrays //////////////////////////////////////////////
unsafe impl<T: Bag, const N: usize> Bag for [T; N] {
    fn build() -> Self {
        core::array::from_fn(|_| T::build())
    }

    fn is_empty(&self) -> bool {
        self.iter().all(|bag| bag.is_empty())
    }

    fn clear(&mut self) {
        self.iter_mut().for_each(|bag| bag.clear());
    }
}

impl<T: AsPort, const N: usize> AsPort for [T; N] {
    type Item = (usize, T::Item); // Include index to identify which bag the value came from
}

impl<T: AsPort, const N: usize> Sealed for [T; N] {}

impl<T: Component, const N: usize> Component for [T; N] {
    type Kind = T::Kind;
    type Input = [T::Input; N];
    type Output = [T::Output; N];
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

//////////////////////////////////////////////// References //////////////////////////////////////////////

macro_rules! impl_ref {
    ( $ty:ty ) => {
        impl<T: Component> Component for $ty {
            type Input = T::Input;
            type Output = T::Output;
            type Kind = T::Kind;
        }

        unsafe impl<T: PartialCoupled> PartialCoupled for $ty {
            type Components = T::Components;
            type ComponentsInput = T::ComponentsInput;
            type ComponentsOutput = T::ComponentsOutput;

            #[inline]
            fn components(&mut self) -> &mut Self::Components {
                (**self).components()
            }
            #[inline]
            fn inputs(&mut self) -> &mut Self::ComponentsInput {
                (**self).inputs()
            }
            #[inline]
            fn outputs(&mut self) -> &mut Self::ComponentsOutput {
                (**self).outputs()
            }
            #[inline]
            fn split(
                &mut self,
            ) -> (
                &mut Self::Components,
                &mut Self::ComponentsInput,
                &mut Self::ComponentsOutput,
            ) {
                (**self).split()
            }
        }

        impl<T: Coupled> Coupled for $ty {
            #[inline]
            fn eic(from: &Self::Input, to: &mut Self::ComponentsInput) {
                T::eic(from, to);
            }
            #[inline]
            fn ic(from: &Self::ComponentsOutput, to: &mut Self::ComponentsInput) {
                T::ic(from, to);
            }
            #[inline]
            fn eoc(from: &Self::ComponentsOutput, to: &mut Self::Output) {
                T::eoc(from, to);
            }
        }
    };
}

impl_ref!(&mut T);

#[cfg(any(feature = "std", feature = "alloc"))]
impl_ref!(alloc::boxed::Box<T>);

//////////////////////////////////////////////// Empty Tuple //////////////////////////////////////////////
unsafe impl Bag for () {
    fn build() -> Self {}

    fn is_empty(&self) -> bool {
        true
    }

    fn clear(&mut self) {}
}
