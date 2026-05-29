//! Blanket implementations of the traits in `traits` module.
#[cfg(any(feature = "std", feature = "alloc"))]
extern crate alloc;

use crate::traits::sealed::Sealed;
use crate::traits::AbstractSimulator;
use crate::traits::AsPort;
use crate::traits::Bag;
use crate::traits::Component;

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

unsafe impl<T: Component, const N: usize> Component for [T; N] {
    type Input = [T::Input; N];
    type Output = [T::Output; N];

    fn get_t_last(&self) -> f64 {
        self.iter()
            .map(|c| c.get_t_last())
            .fold(f64::INFINITY, f64::min)
    }

    fn set_t_last(&mut self, t_last: f64) {
        self.iter_mut().for_each(|c| c.set_t_last(t_last));
    }

    fn get_t_next(&self) -> f64 {
        self.iter()
            .map(|c| c.get_t_next())
            .fold(f64::INFINITY, f64::min)
    }

    fn set_t_next(&mut self, t_next: f64) {
        self.iter_mut().for_each(|c| c.set_t_next(t_next));
    }
}

unsafe impl<T: AbstractSimulator, const N: usize> AbstractSimulator for [T; N] {
    #[inline]
    fn start(&mut self, t_start: f64) -> f64 {
        self.iter_mut().fold(f64::INFINITY, |t_next, c| {
            f64::min(t_next, c.start(t_start))
        })
    }

    #[inline]
    fn stop(&mut self, t_stop: f64) {
        self.iter_mut().for_each(|c| c.stop(t_stop));
    }

    #[inline]
    fn lambda(&mut self, output: &mut Self::Output, t: f64) {
        self.iter_mut()
            .zip(output.iter_mut())
            .for_each(|(c, out)| c.lambda(out, t));
    }

    #[inline]
    fn delta(&mut self, input: &mut Self::Input, output: &mut Self::Output, t: f64) -> f64 {
        self.iter_mut()
            .zip(input.iter_mut())
            .zip(output.iter_mut())
            .fold(f64::INFINITY, |t_next, ((c, inp), out)| {
                f64::min(t_next, c.delta(inp, out, t))
            })
    }
}

//////////////////////////////////////////////// References //////////////////////////////////////////////

macro_rules! impl_ref {
    ( $ty:ty ) => {
        unsafe impl<T: Component> Component for $ty {
            type Input = T::Input;
            type Output = T::Output;

            fn get_t_last(&self) -> f64 {
                (**self).get_t_last()
            }

            fn set_t_last(&mut self, t_last: f64) {
                (**self).set_t_last(t_last)
            }

            fn get_t_next(&self) -> f64 {
                (**self).get_t_next()
            }

            fn set_t_next(&mut self, t_next: f64) {
                (**self).set_t_next(t_next)
            }
        }

        unsafe impl<T: AbstractSimulator> AbstractSimulator for $ty {
            fn start(&mut self, t_start: f64) -> f64 {
                (**self).start(t_start)
            }

            fn stop(&mut self, t_stop: f64) {
                (**self).stop(t_stop);
            }

            fn lambda(&mut self, output: &mut Self::Output, t: f64) {
                (**self).lambda(output, t);
            }

            fn delta(&mut self, input: &mut Self::Input, output: &mut Self::Output, t: f64) -> f64 {
                (**self).delta(input, output, t)
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
