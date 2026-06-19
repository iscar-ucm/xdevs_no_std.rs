pub mod atomic;
pub mod coupled;

use crate::port::Bag;
use sealed::Sealed;

/// Marker type for atomic DEVS models.
pub struct AtomicKind;

impl Sealed for AtomicKind {}

/// Marker type for coupled DEVS models.
pub struct CoupledKind;

impl Sealed for CoupledKind {}

/// Marker type for component groups.
///
/// This kind represents collections of components (for example arrays, tuples,
/// or structs) where elements implement [`Component`] but are not coupled
/// models by themselves.
pub struct ComponentsKind;

impl Sealed for ComponentsKind {}

/// Interface for DEVS components. All DEVS components must implement this trait.
pub trait Component {
    /// Kind of DEVS model. It can be [`AtomicKind`], [`CoupledKind`], or [`ComponentsKind`].
    type Kind: Sealed;

    /// Input event bag of the model.
    type Input: Bag;

    /// Output event bag of the model.
    type Output: Bag;
}

impl<T: Component, const N: usize> Component for [T; N] {
    type Kind = [T::Kind; N];
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

pub(crate) mod sealed {
    /// Trait used to prevent users from implementing certain traits manually.
    pub trait Sealed {}

    impl<T: Sealed, const N: usize> Sealed for [T; N] {}
}
