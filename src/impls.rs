///! Blanket implementations of the traits in `traits` module.
#[cfg(any(feature = "std", feature = "alloc"))]
extern crate alloc;

use crate::traits::AbstractSimulator;
use crate::traits::Bag;
use crate::traits::Component;

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
