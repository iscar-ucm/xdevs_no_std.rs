pub mod atomic;
pub mod coupled;

use crate::{port::Bag, processor::Processor};
use sealed::Sealed;

/// Marker type for atomic DEVS models.
pub struct AtomicKind;

impl Sealed for AtomicKind {}

/// Marker type for coupled DEVS models.
pub struct CoupledKind;

impl Sealed for CoupledKind {}

/// Interface for DEVS components. All DEVS components must implement this trait.
pub trait Component {
    /// Kind of DEVS model. It can be either [`AtomicKind`] or [`CoupledKind`].
    type Kind: Sealed;

    /// Input event bag of the model.
    type Input: Bag;

    /// Output event bag of the model.
    type Output: Bag;
}

/// Interface for simulating DEVS models. All DEVS models must implement this trait.
///
/// # Safety
///
/// This trait is implemented internally. Do not implement it manually.
pub unsafe trait AbstractSimulator<K>: Component<Kind = K>
where
    Self: Sized,
{
    /// It starts the simulation, setting the initial time to t_start.
    /// It returns the time for the next state transition of the inner DEVS model.
    fn start(processor: &mut Processor<Self>, t_start: f64) -> f64;

    /// It stops the simulation, setting the last time to t_stop.
    fn stop(processor: &mut Processor<Self>);

    /// Executes output functions and propagates messages according to EOCs.
    /// Internally, it checks that the model is imminent before executing.
    fn lambda(processor: &mut Processor<Self>, output: &mut Self::Output, t: f64);

    /// Propagates messages according to ICs and EICs, and executes state transition functions.
    /// Internally, it checks that the model is imminent before executing.
    /// Finally, it returns the time for the next state transition of the inner DEVS model.
    fn delta(
        processor: &mut Processor<Self>,
        input: &mut Self::Input,
        output: &mut Self::Output,
        t: f64,
    ) -> f64;
}

impl<T: Component, const N: usize> Component for [T; N] {
    type Kind = T::Kind;
    type Input = [T::Input; N];
    type Output = [T::Output; N];
}

impl<T: Component> Component for &mut T {
    type Input = T::Input;
    type Output = T::Output;
    type Kind = T::Kind;
}

#[cfg(feature = "alloc")]
impl<T: Component> Component for alloc::boxed::Box<T> {
    type Input = T::Input;
    type Output = T::Output;
    type Kind = T::Kind;
}

mod sealed {
    /// Trait used to prevent users from implementing certain traits manually.
    pub trait Sealed {}
}
