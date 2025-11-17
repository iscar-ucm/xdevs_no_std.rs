///! Blanket implementations of the traits in `traits` module.
#[cfg(any(feature = "std", feature = "alloc"))]
extern crate alloc;

use crate::traits::AbstractSimulator;
use crate::traits::Bag;
use crate::traits::Component;

//////////////////////////////////////////////// Iterables //////////////////////////////////////////////

macro_rules! seq_impl_body {
    () => {
        fn is_empty(&self) -> bool {
            self.iter().all(|bag| bag.is_empty())
        }

        fn clear(&mut self) {
            for bag in self.iter_mut() {
                bag.clear();
            }
        }
    };
}

#[cfg(any(feature = "std", feature = "alloc"))]
unsafe impl<T: Bag> Bag for alloc::vec::Vec<T> {
    seq_impl_body!();
}

unsafe impl<T: Bag, const N: usize> Bag for heapless::Vec<T, N> {
    seq_impl_body!();
}
unsafe impl<T: Bag, const N: usize> Bag for [T; N] {
    seq_impl_body!();
}

//////////////////////////////////////////////// Tuples //////////////////////////////////////////////

unsafe impl Bag for () {
    fn is_empty(&self) -> bool {
        true
    }

    fn clear(&mut self) {}
}

// Macro to implement Bag for tuples
macro_rules! impl_bag_for_tuples {
    ( $( $name:ident ),+ ) => {
        #[allow(non_snake_case)]
        unsafe impl<$( $name ),+> Bag for ( $( $name ),+ )
        where
            $( $name: Bag ),+
        {
            fn is_empty(&self) -> bool {
                let ( $( $name ),+ ) = self;
                true $( && $name.is_empty() )+
            }

            fn clear(&mut self) {
                let ( $( $name ),+ ) = self;
                $( $name.clear(); )+
            }
        }
    };
}

// Generate impls for tuples of size 2 up to, say, 12
impl_bag_for_tuples!(T1, T2);
impl_bag_for_tuples!(T1, T2, T3);
impl_bag_for_tuples!(T1, T2, T3, T4);
impl_bag_for_tuples!(T1, T2, T3, T4, T5);
impl_bag_for_tuples!(T1, T2, T3, T4, T5, T6);
impl_bag_for_tuples!(T1, T2, T3, T4, T5, T6, T7);
impl_bag_for_tuples!(T1, T2, T3, T4, T5, T6, T7, T8);
impl_bag_for_tuples!(T1, T2, T3, T4, T5, T6, T7, T8, T9);
impl_bag_for_tuples!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10);
impl_bag_for_tuples!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11);
impl_bag_for_tuples!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12);
impl_bag_for_tuples!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13);
impl_bag_for_tuples!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14);
impl_bag_for_tuples!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15);
impl_bag_for_tuples!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16);

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
