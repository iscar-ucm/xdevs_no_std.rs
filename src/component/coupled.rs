use crate::{
    component::AbstractSimulator, port::Bag, processor::AsProcessor, processor::Processor,
    Component, CoupledKind,
};

/// Interface for DEVS coupled models. All DEVS coupled models must implement this trait.
pub trait Coupled: PartialCoupled {
    /// External Input Coupling. Propagates input events from the coupled model to its inner components.
    #[allow(unused_variables)]
    #[inline]
    fn eic(from: &Self::Input, to: &mut Self::ComponentsInput) {}

    /// Internal Coupling. Propagates output events from inner components to input events of other inner components.
    #[allow(unused_variables)]
    #[inline]
    fn ic(from: &Self::ComponentsOutput, to: &mut Self::ComponentsInput) {}

    /// External Output Coupling. Propagates output events from inner components to the coupled model's output.
    #[allow(unused_variables)]
    #[inline]
    fn eoc(from: &Self::ComponentsOutput, to: &mut Self::Output) {}
}

/// Partial interface for DEVS coupled models.
/// It is used as a helper trait to implement the [`Coupled`] trait.
///
/// # Safety
///
/// This trait must be implemented via the [`coupled`](macro@crate::coupled) macro. Do not implement it manually.
pub unsafe trait PartialCoupled: Component<Kind = CoupledKind>
where
    Self::Components: AsProcessor<Input = Self::ComponentsInput, Output = Self::ComponentsOutput>,
{
    type Components: AsProcessor;
    type ComponentsInput: Bag;
    type ComponentsOutput: Bag;

    fn components(&mut self) -> &mut Self::Components;
    fn inputs(&mut self) -> &mut Self::ComponentsInput;
    fn outputs(&mut self) -> &mut Self::ComponentsOutput;
    fn split(
        &mut self,
    ) -> (
        &mut Self::Components,
        &mut Self::ComponentsInput,
        &mut Self::ComponentsOutput,
    );
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

unsafe impl<T: PartialCoupled> PartialCoupled for &mut T {
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

impl<T: Coupled> Coupled for &mut T {
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

#[cfg(feature = "alloc")]
unsafe impl<T: PartialCoupled> PartialCoupled for alloc::boxed::Box<T> {
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

#[cfg(feature = "alloc")]
impl<T: Coupled> Coupled for alloc::boxed::Box<T> {
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
