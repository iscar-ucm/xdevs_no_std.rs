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
    type Input = ();
    type Output = ();
    type InputRef<'a>
        = [T::InputRef<'a>; N]
    where
        Self: 'a;
    type OutputRef<'a>
        = [T::OutputRef<'a>; N]
    where
        Self: 'a;

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

    fn get_input(&self) -> &Self::Input {
        unimplemented!("get_input is not supported for collections; access elements directly")
    }

    fn get_input_mut(&mut self) -> &mut Self::Input {
        unimplemented!("get_input_mut is not supported for collections; access elements directly")
    }

    fn get_output(&self) -> &Self::Output {
        unimplemented!("get_output is not supported for collections; access elements directly")
    }

    fn get_output_mut(&mut self) -> &mut Self::Output {
        unimplemented!("get_output_mut is not supported for collections; access elements directly")
    }

    fn get_ports(&mut self) -> (Self::InputRef<'_>, Self::OutputRef<'_>) {
        // SAFETY: An uninitialized `[MaybeUninit<_>; N]` is valid.
        let mut inputs: [core::mem::MaybeUninit<T::InputRef<'_>>; N] =
            unsafe { core::mem::MaybeUninit::uninit().assume_init() };
        let mut outputs: [core::mem::MaybeUninit<T::OutputRef<'_>>; N] =
            unsafe { core::mem::MaybeUninit::uninit().assume_init() };

        for (i, component) in self.iter_mut().enumerate() {
            let (input, output) = component.get_ports();
            inputs[i].write(input);
            outputs[i].write(output);
        }

        // SAFETY: All elements have been initialized in the loop above.
        unsafe {
            (
                inputs.map(|m| m.assume_init()),
                outputs.map(|m| m.assume_init()),
            )
        }
    }

    fn get_out_ports(&self) -> Self::OutputRef<'_> {
        // SAFETY: An uninitialized `[MaybeUninit<_>; N]` is valid.
        let mut outputs: [core::mem::MaybeUninit<T::OutputRef<'_>>; N] =
            unsafe { core::mem::MaybeUninit::uninit().assume_init() };

        for (i, component) in self.iter().enumerate() {
            outputs[i].write(component.get_out_ports());
        }

        // SAFETY: All elements have been initialized in the loop above.
        unsafe { outputs.map(|m| m.assume_init()) }
    }

    fn clear_input(&mut self) {
        self.iter_mut().for_each(|c| c.clear_input());
    }

    fn clear_output(&mut self) {
        self.iter_mut().for_each(|c| c.clear_output());
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
    fn lambda(&mut self, t: f64) {
        self.iter_mut().for_each(|c| c.lambda(t));
    }

    #[inline]
    fn delta(&mut self, t: f64) -> f64 {
        self.iter_mut()
            .fold(f64::INFINITY, |t_next, c| f64::min(t_next, c.delta(t)))
    }
}

//////////////////////////////////////////////// References //////////////////////////////////////////////

macro_rules! impl_ref {
    ( $ty:ty ) => {
        unsafe impl<T: Bag> Bag for $ty {
            fn is_empty(&self) -> bool {
                (**self).is_empty()
            }

            fn clear(&mut self) {
                (**self).clear();
            }
        }

        unsafe impl<T: Component> Component for $ty {
            type Input = T::Input;
            type Output = T::Output;
            type InputRef<'a>
                = T::InputRef<'a>
            where
                Self: 'a;
            type OutputRef<'a>
                = T::OutputRef<'a>
            where
                Self: 'a;

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

            fn get_input(&self) -> &Self::Input {
                (**self).get_input()
            }

            fn get_input_mut(&mut self) -> &mut Self::Input {
                (**self).get_input_mut()
            }

            fn get_output(&self) -> &Self::Output {
                (**self).get_output()
            }

            fn get_output_mut(&mut self) -> &mut Self::Output {
                (**self).get_output_mut()
            }

            fn get_ports(&mut self) -> (Self::InputRef<'_>, Self::OutputRef<'_>) {
                (**self).get_ports()
            }

            fn get_out_ports(&self) -> Self::OutputRef<'_> {
                (**self).get_out_ports()
            }
        }

        unsafe impl<T: AbstractSimulator> AbstractSimulator for $ty {
            fn start(&mut self, t_start: f64) -> f64 {
                (**self).start(t_start)
            }

            fn stop(&mut self, t_stop: f64) {
                (**self).stop(t_stop);
            }

            fn lambda(&mut self, t: f64) {
                (**self).lambda(t);
            }

            fn delta(&mut self, t: f64) -> f64 {
                (**self).delta(t)
            }
        }
    };
}

impl_ref!(&mut T);

#[cfg(any(feature = "std", feature = "alloc"))]
impl_ref!(alloc::boxed::Box<T>);

//////////////////////////////////////////////// Empty Tuple //////////////////////////////////////////////
unsafe impl Bag for () {
    fn is_empty(&self) -> bool {
        true
    }

    fn clear(&mut self) {}
}
