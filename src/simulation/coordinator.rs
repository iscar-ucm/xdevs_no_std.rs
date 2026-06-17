use crate::{
    component::{
        coupled::{ComponentsInput, ComponentsOutput, Coupled},
        CoupledKind,
    },
    port::Bag,
    simulation::{AbstractSimulator, Simulable},
};
use core::ops::{Deref, DerefMut};

/// Coordinator that encapsulates coupled-model simulation state.
pub struct Coordinator<T: Coupled> {
    pub(crate) component: T,
    pub(crate) components_inputs: ComponentsInput<T>,
    pub(crate) components_outputs: ComponentsOutput<T>,
    pub(crate) t_next: f64,
}

impl<T: Coupled> Coordinator<T> {
    /// Creates a new coordinator for the given coupled model.
    #[inline(always)]
    pub fn new(component: T) -> Self {
        Self {
            component,
            components_inputs: ComponentsInput::<T>::build(),
            components_outputs: ComponentsOutput::<T>::build(),
            t_next: f64::INFINITY,
        }
    }
}

impl<T: Coupled> Deref for Coordinator<T> {
    type Target = T;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.component
    }
}

impl<T: Coupled> DerefMut for Coordinator<T> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.component
    }
}

// Coupled models can be simulated using a `Coupled` struct
impl<T: Coupled> Simulable<CoupledKind> for T {
    type Simulator = Coordinator<T>;

    fn to_simulator(self) -> Self::Simulator {
        Coordinator::new(self)
    }
}

unsafe impl<T: Coupled> AbstractSimulator for Coordinator<T> {
    type Input = T::Input;
    type Output = T::Output;

    #[inline(always)]
    fn start(&mut self, t_start: f64) -> f64 {
        let t_next = self.component.get_components_mut().start(t_start);
        self.t_next = t_next;
        t_next
    }

    #[inline(always)]
    fn stop(&mut self) {
        self.component.get_components_mut().stop();
    }

    #[inline(always)]
    fn lambda(&mut self, output: &mut Self::Output, t: f64) {
        if t >= self.t_next {
            self.component
                .get_components_mut()
                .lambda(&mut self.components_outputs, t);
            T::eoc(&self.components_outputs, output);
        }
    }

    #[inline(always)]
    fn delta(&mut self, input: &mut Self::Input, output: &mut Self::Output, t: f64) -> f64 {
        let t_next = self.t_next;
        if t < t_next && input.is_empty() {
            return t_next;
        }

        T::eic(input, &mut self.components_inputs);
        T::ic(&self.components_outputs, &mut self.components_inputs);
        let t_next = self.component.get_components_mut().delta(
            &mut self.components_inputs,
            &mut self.components_outputs,
            t,
        );

        self.t_next = t_next;

        input.clear();
        output.clear();

        t_next
    }
}
