use crate::{
    component::{Component, CoupledKind},
    simulation::ErasedSimulable,
};

/// Partial interface for DEVS coupled models. All DEVS coupled models must implement this trait.
pub trait PartialCoupled: Component<Kind = CoupledKind> {
    /// Type of the inner components of this coupled model.
    type Components: ErasedSimulable;

    fn get_components(&self) -> &Processors<Self>;

    fn get_components_mut(&mut self) -> &mut Processors<Self>;
}

/// Type alias for the inner components of a coupled model.
pub type Components<T> = <T as PartialCoupled>::Components;

/// Type alias for the simulator of the inner components of a coupled model.
pub type Processors<T> = <<T as PartialCoupled>::Components as ErasedSimulable>::Simulator;

/// Type alias for the input of the inner components of a coupled model.
pub type ComponentsInput<T> = <<T as PartialCoupled>::Components as Component>::Input;

/// Type alias for the output of the inner components of a coupled model.
pub type ComponentsOutput<T> = <<T as PartialCoupled>::Components as Component>::Output;

/// Interface for DEVS coupled models. All DEVS coupled models must implement this trait.
pub trait Coupled: PartialCoupled {
    /// External Input Coupling. Propagates input events from the coupled model to its inner components.
    #[allow(unused_variables)]
    #[inline(always)]
    fn eic(from: &Self::Input, to: &mut ComponentsInput<Self>) {}

    /// Internal Coupling. Propagates output events from inner components to input events of other inner components.
    #[allow(unused_variables)]
    #[inline(always)]
    fn ic(from: &ComponentsOutput<Self>, to: &mut ComponentsInput<Self>) {}

    /// External Output Coupling. Propagates output events from inner components to the coupled model's output.
    #[allow(unused_variables)]
    #[inline(always)]
    fn eoc(from: &ComponentsOutput<Self>, to: &mut Self::Output) {}
}

impl<T: PartialCoupled> PartialCoupled for &mut T {
    type Components = T::Components;

    fn get_components(&self) -> &Processors<Self> {
        T::get_components(&**self)
    }

    fn get_components_mut(&mut self) -> &mut Processors<Self> {
        T::get_components_mut(&mut **self)
    }
}

impl<T: Coupled> Coupled for &mut T {
    #[inline(always)]
    fn eic(from: &Self::Input, to: &mut ComponentsInput<Self>) {
        T::eic(from, to);
    }
    #[inline(always)]
    fn ic(from: &ComponentsOutput<Self>, to: &mut ComponentsInput<Self>) {
        T::ic(from, to);
    }
    #[inline(always)]
    fn eoc(from: &ComponentsOutput<Self>, to: &mut Self::Output) {
        T::eoc(from, to);
    }
}

#[cfg(feature = "alloc")]
impl<T: PartialCoupled> PartialCoupled for alloc::boxed::Box<T> {
    type Components = T::Components;

    fn get_components(&self) -> &Processors<Self> {
        T::get_components(&**self)
    }

    fn get_components_mut(&mut self) -> &mut Processors<Self> {
        T::get_components_mut(&mut **self)
    }
}

#[cfg(feature = "alloc")]
impl<T: Coupled> Coupled for alloc::boxed::Box<T> {
    #[inline(always)]
    fn eic(from: &Self::Input, to: &mut ComponentsInput<Self>) {
        T::eic(from, to);
    }
    #[inline(always)]
    fn ic(from: &ComponentsOutput<Self>, to: &mut ComponentsInput<Self>) {
        T::ic(from, to);
    }
    #[inline(always)]
    fn eoc(from: &ComponentsOutput<Self>, to: &mut Self::Output) {
        T::eoc(from, to);
    }
}
