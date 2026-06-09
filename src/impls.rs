//! Blanket implementations of the traits in `traits` module.
#[cfg(any(feature = "std", feature = "alloc"))]
extern crate alloc;

use crate::simulator::Simulator;
use crate::traits::{sealed::Sealed, AbstractSimulator, AsPort, Bag, Component};
use crate::Atomic;

////////////////////////////////////////////////// Atomic //////////////////////////////////////////////
unsafe impl<T: Atomic> AbstractSimulator for T {
    #[inline]
    fn start(simulator: &mut Simulator<Self>, t_start: f64) -> f64 {
        // set t_last to t_start
        simulator.set_t_last(t_start);
        // start state and get t_next from ta
        simulator.start();
        let t_next = t_start + simulator.ta();
        simulator.set_t_next(t_next);

        t_next
    }
    #[inline]
    fn stop(simulator: &mut Simulator<Self>, t_stop: f64) {
        // stop state
        simulator.stop();
        // set t_last to t_stop and t_next to infinity
        simulator.set_t_last(t_stop);
        simulator.set_t_next(f64::INFINITY);
    }
    #[inline]
    fn lambda(simulator: &mut Simulator<Self>, output: &mut Self::Output, t: f64) {
        if t >= simulator.get_t_next() {
            // execute atomic model's lambda if applies
            simulator.lambda(output);
        }
    }
    #[inline]
    fn delta(
        simulator: &mut Simulator<Self>,
        input: &mut Self::Input,
        output: &mut Self::Output,
        t: f64,
    ) -> f64 {
        let mut t_next = simulator.get_t_next();
        if !::xdevs::traits::Bag::is_empty(input) {
            if t >= t_next {
                // confluent transition
                simulator.delta_conf(input);
                // clear output events
                output.clear();
            } else {
                // external transition
                let e = t - simulator.get_t_last();
                simulator.delta_ext(e, input);
            }
            // clear input events
            input.clear();
        } else if t >= t_next {
            // internal transition
            simulator.delta_int();
            // clear output events
            output.clear();
        } else {
            return t_next; // nothing to do
        }
        // get t_next from ta and set new t_last and t_next
        t_next = t + simulator.ta();
        simulator.set_t_last(t);
        simulator.set_t_next(t_next);

        t_next
    }
}
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
    type Input = [T::Input; N];
    type Output = [T::Output; N];
}

impl<T: Atomic, const N: usize> Atomic for [T; N] {
    #[inline]
    fn delta_int(&mut self) {
        for component in self.iter_mut() {
            component.delta_int();
        }
    }

    #[inline]
    fn lambda(&self, output: &mut Self::Output) {
        for (component, output_bag) in self.iter().zip(output.iter_mut()) {
            component.lambda(output_bag);
        }
    }

    #[inline]
    fn ta(&self) -> f64 {
        self.iter()
            .map(|component| component.ta())
            .fold(f64::INFINITY, f64::min)
    }

    #[inline]
    fn delta_ext(&mut self, elapsed: f64, input: &Self::Input) {
        for (component, input_bag) in self.iter_mut().zip(input.iter()) {
            component.delta_ext(elapsed, input_bag);
        }
    }
}

//////////////////////////////////////////////// References //////////////////////////////////////////////

/*
macro_rules! impl_ref {
    ( $ty:ty ) => {
        impl<T: Component> Component for $ty {
            type Input = T::Input;
            type Output = T::Output;
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
*/

//////////////////////////////////////////////// Empty Tuple //////////////////////////////////////////////
unsafe impl Bag for () {
    fn build() -> Self {}

    fn is_empty(&self) -> bool {
        true
    }

    fn clear(&mut self) {}
}
