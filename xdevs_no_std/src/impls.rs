//! Blanket implementations of the traits in `traits` module.
#[cfg(any(feature = "std", feature = "alloc"))]
extern crate alloc;

use crate::traits::AbstractSimulator;
use crate::traits::Bag;
use crate::traits::Component;
use crate::Instant;

//////////////////////////////////////////////// Iterables //////////////////////////////////////////////

macro_rules! seq_bag_impl_body {
    () => {
        fn is_empty(&self) -> bool {
            self.iter().all(|bag| bag.is_empty())
        }

        fn clear(&mut self) {
            self.iter_mut().for_each(|bag| bag.clear());
        }
    };
}

macro_rules! seq_simulator_impl_body {
    () => {
        #[inline]
        fn start(&mut self, t_start: Instant) -> Instant {
            self.iter_mut().fold(Instant::MAX, |t_next, c| {
                Instant::min(t_next, c.start(t_start))
            })
        }

        #[inline]
        fn stop(&mut self, t_stop: Instant) {
            self.iter_mut().for_each(|c| c.stop(t_stop));
        }

        #[inline]
        fn lambda(&mut self, t: Instant) {
            self.iter_mut().for_each(|c| c.lambda(t));
        }

        #[inline]
        fn delta(&mut self, t: Instant) -> Instant {
            self.iter_mut()
                .fold(Instant::MAX, |t_next, c| Instant::min(t_next, c.delta(t)))
        }
    };
}

#[cfg(any(feature = "std", feature = "alloc"))]
unsafe impl<T: Bag> Bag for alloc::vec::Vec<T> {
    seq_bag_impl_body!();
}

#[cfg(any(feature = "std", feature = "alloc"))]
unsafe impl<T: Component> Component for alloc::vec::Vec<T> {
    type Input = ();
    type Output = ();
    type InputRef<'a>
        = alloc::vec::Vec<T::InputRef<'a>>
    where
        Self: 'a;
    type OutputRef<'a>
        = alloc::vec::Vec<T::OutputRef<'a>>
    where
        Self: 'a;

    fn get_t_last(&self) -> Instant {
        self.iter()
            .map(|c| c.get_t_last())
            .fold(Instant::MAX, Instant::min)
    }

    fn set_t_last(&mut self, t_last: Instant) {
        self.iter_mut().for_each(|c| c.set_t_last(t_last));
    }

    fn get_t_next(&self) -> Instant {
        self.iter()
            .map(|c| c.get_t_next())
            .fold(Instant::MAX, Instant::min)
    }

    fn set_t_next(&mut self, t_next: Instant) {
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
        let (inputs, outputs): (alloc::vec::Vec<_>, alloc::vec::Vec<_>) =
            self.iter_mut().map(|c| c.get_ports()).unzip();
        (inputs, outputs)
    }

    fn get_out_ports(&self) -> Self::OutputRef<'_> {
        self.iter().map(|c| c.get_out_ports()).collect()
    }

    fn clear_input(&mut self) {
        self.iter_mut().for_each(|c| c.clear_input());
    }

    fn clear_output(&mut self) {
        self.iter_mut().for_each(|c| c.clear_output());
    }
}

#[cfg(any(feature = "std", feature = "alloc"))]
unsafe impl<T: AbstractSimulator> AbstractSimulator for alloc::vec::Vec<T> {
    seq_simulator_impl_body!();
}

unsafe impl<T: Bag, const N: usize> Bag for heapless::Vec<T, N> {
    seq_bag_impl_body!();
}

unsafe impl<T: Component, const N: usize> Component for heapless::Vec<T, N> {
    type Input = ();
    type Output = ();
    type InputRef<'a>
        = heapless::Vec<T::InputRef<'a>, N>
    where
        Self: 'a;
    type OutputRef<'a>
        = heapless::Vec<T::OutputRef<'a>, N>
    where
        Self: 'a;

    fn get_t_last(&self) -> Instant {
        self.iter()
            .map(|c| c.get_t_last())
            .fold(Instant::MAX, Instant::min)
    }

    fn set_t_last(&mut self, t_last: Instant) {
        self.iter_mut().for_each(|c| c.set_t_last(t_last));
    }

    fn get_t_next(&self) -> Instant {
        self.iter()
            .map(|c| c.get_t_next())
            .fold(Instant::MAX, Instant::min)
    }

    fn set_t_next(&mut self, t_next: Instant) {
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
        let mut inputs = heapless::Vec::new();
        let mut outputs = heapless::Vec::new();
        for component in self.iter_mut() {
            let (input, output) = component.get_ports();
            // SAFETY: Vectors are not full
            unsafe {
                inputs.push_unchecked(input);
                outputs.push_unchecked(output);
            }
        }
        (inputs, outputs)
    }

    fn get_out_ports(&self) -> Self::OutputRef<'_> {
        let mut outputs = heapless::Vec::new();
        for component in self.iter() {
            // SAFETY: Vector is not full
            unsafe { outputs.push_unchecked(component.get_out_ports()) };
        }
        outputs
    }

    fn clear_input(&mut self) {
        self.iter_mut().for_each(|c| c.clear_input());
    }

    fn clear_output(&mut self) {
        self.iter_mut().for_each(|c| c.clear_output());
    }
}

unsafe impl<T: AbstractSimulator, const N: usize> AbstractSimulator for heapless::Vec<T, N> {
    seq_simulator_impl_body!();
}

unsafe impl<T: Bag, const N: usize> Bag for [T; N] {
    seq_bag_impl_body!();
}

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

    fn get_t_last(&self) -> Instant {
        self.iter()
            .map(|c| c.get_t_last())
            .fold(Instant::MAX, Instant::min)
    }

    fn set_t_last(&mut self, t_last: Instant) {
        self.iter_mut().for_each(|c| c.set_t_last(t_last));
    }

    fn get_t_next(&self) -> Instant {
        self.iter()
            .map(|c| c.get_t_next())
            .fold(Instant::MAX, Instant::min)
    }

    fn set_t_next(&mut self, t_next: Instant) {
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
    seq_simulator_impl_body!();
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

// Generate impls for tuples of size 2 up to 16
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
            type InputRef<'a>
                = T::InputRef<'a>
            where
                Self: 'a;
            type OutputRef<'a>
                = T::OutputRef<'a>
            where
                Self: 'a;

            fn get_t_last(&self) -> Instant {
                (**self).get_t_last()
            }

            fn set_t_last(&mut self, t_last: Instant) {
                (**self).set_t_last(t_last)
            }

            fn get_t_next(&self) -> Instant {
                (**self).get_t_next()
            }

            fn set_t_next(&mut self, t_next: Instant) {
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
            fn start(&mut self, t_start: Instant) -> Instant {
                (**self).start(t_start)
            }

            fn stop(&mut self, t_stop: Instant) {
                (**self).stop(t_stop);
            }

            fn lambda(&mut self, t: Instant) {
                (**self).lambda(t);
            }

            fn delta(&mut self, t: Instant) -> Instant {
                (**self).delta(t)
            }
        }
    };
}

impl_ref!(&mut T);

#[cfg(any(feature = "std", feature = "alloc"))]
impl_ref!(alloc::boxed::Box<T>);
