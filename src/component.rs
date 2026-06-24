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

impl<T: Component> Component for Option<T> {
    type Kind = Option<T::Kind>;
    type Input = T::Input;
    type Output = T::Output;
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

macro_rules! impl_component_for_tuple {
    ($($T:ident),+) => {
        impl<$($T: Component),+> Component for ($($T,)+) {
            type Kind = ($($T::Kind,)+);
            type Input = ($($T::Input,)+);
            type Output = ($($T::Output,)+);
        }
    }
}

impl_component_for_tuple!(T0);
impl_component_for_tuple!(T0, T1);
impl_component_for_tuple!(T0, T1, T2);
impl_component_for_tuple!(T0, T1, T2, T3);
impl_component_for_tuple!(T0, T1, T2, T3, T4);
impl_component_for_tuple!(T0, T1, T2, T3, T4, T5);
impl_component_for_tuple!(T0, T1, T2, T3, T4, T5, T6);
impl_component_for_tuple!(T0, T1, T2, T3, T4, T5, T6, T7);
impl_component_for_tuple!(T0, T1, T2, T3, T4, T5, T6, T7, T8);
impl_component_for_tuple!(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9);
impl_component_for_tuple!(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10);
impl_component_for_tuple!(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11);

pub(crate) mod sealed {
    /// Trait used to prevent users from implementing certain traits manually.
    pub trait Sealed {}

    impl<T: Sealed, const N: usize> Sealed for [T; N] {}
    impl<T: Sealed> Sealed for Option<T> {}

    macro_rules! impl_sealed_for_tuple {
        ($($T:ident),+) => {
            impl<$($T: Sealed),+> Sealed for ($($T,)+) {}
        }
    }

    impl_sealed_for_tuple!(T0);
    impl_sealed_for_tuple!(T0, T1);
    impl_sealed_for_tuple!(T0, T1, T2);
    impl_sealed_for_tuple!(T0, T1, T2, T3);
    impl_sealed_for_tuple!(T0, T1, T2, T3, T4);
    impl_sealed_for_tuple!(T0, T1, T2, T3, T4, T5);
    impl_sealed_for_tuple!(T0, T1, T2, T3, T4, T5, T6);
    impl_sealed_for_tuple!(T0, T1, T2, T3, T4, T5, T6, T7);
    impl_sealed_for_tuple!(T0, T1, T2, T3, T4, T5, T6, T7, T8);
    impl_sealed_for_tuple!(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9);
    impl_sealed_for_tuple!(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10);
    impl_sealed_for_tuple!(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11);
}
