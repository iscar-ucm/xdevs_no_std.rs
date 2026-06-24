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

macro_rules! impl_component_for_tuple {
    ($($idx:tt => $T:ident),+) => {
        impl<$($T: Component),+> Component for ($($T,)+) {
            type Kind = ($($T::Kind,)+);
            type Input = ($($T::Input,)+);
            type Output = ($($T::Output,)+);
        }
    }
}

impl_component_for_tuple!(0 => T0);
impl_component_for_tuple!(0 => T0, 1 => T1);
impl_component_for_tuple!(0 => T0, 1 => T1, 2 => T2);
impl_component_for_tuple!(0 => T0, 1 => T1, 2 => T2, 3 => T3);
impl_component_for_tuple!(0 => T0, 1 => T1, 2 => T2, 3 => T3, 4 => T4);
impl_component_for_tuple!(0 => T0, 1 => T1, 2 => T2, 3 => T3, 4 => T4, 5 => T5);
impl_component_for_tuple!(0 => T0, 1 => T1, 2 => T2, 3 => T3, 4 => T4, 5 => T5, 6 => T6);
impl_component_for_tuple!(0 => T0, 1 => T1, 2 => T2, 3 => T3, 4 => T4, 5 => T5, 6 => T6, 7 => T7);
impl_component_for_tuple!(0 => T0, 1 => T1, 2 => T2, 3 => T3, 4 => T4, 5 => T5, 6 => T6, 7 => T7, 8 => T8);
impl_component_for_tuple!(0 => T0, 1 => T1, 2 => T2, 3 => T3, 4 => T4, 5 => T5, 6 => T6, 7 => T7, 8 => T8, 9 => T9);
impl_component_for_tuple!(0 => T0, 1 => T1, 2 => T2, 3 => T3, 4 => T4, 5 => T5, 6 => T6, 7 => T7, 8 => T8, 9 => T9, 10 => T10);
impl_component_for_tuple!(0 => T0, 1 => T1, 2 => T2, 3 => T3, 4 => T4, 5 => T5, 6 => T6, 7 => T7, 8 => T8, 9 => T9, 10 => T10, 11 => T11);

pub(crate) mod sealed {
    /// Trait used to prevent users from implementing certain traits manually.
    pub trait Sealed {}

    impl<T: Sealed, const N: usize> Sealed for [T; N] {}

    macro_rules! impl_sealed_for_tuple {
        ($($idx:tt => $T:ident),+) => {
            impl<$($T: Sealed),+> Sealed for ($($T,)+) {}
        }
    }

    impl_sealed_for_tuple!(0 => T0);
    impl_sealed_for_tuple!(0 => T0, 1 => T1);
    impl_sealed_for_tuple!(0 => T0, 1 => T1, 2 => T2);
    impl_sealed_for_tuple!(0 => T0, 1 => T1, 2 => T2, 3 => T3);
    impl_sealed_for_tuple!(0 => T0, 1 => T1, 2 => T2, 3 => T3, 4 => T4);
    impl_sealed_for_tuple!(0 => T0, 1 => T1, 2 => T2, 3 => T3, 4 => T4, 5 => T5);
    impl_sealed_for_tuple!(0 => T0, 1 => T1, 2 => T2, 3 => T3, 4 => T4, 5 => T5, 6 => T6);
    impl_sealed_for_tuple!(0 => T0, 1 => T1, 2 => T2, 3 => T3, 4 => T4, 5 => T5, 6 => T6, 7 => T7);
    impl_sealed_for_tuple!(0 => T0, 1 => T1, 2 => T2, 3 => T3, 4 => T4, 5 => T5, 6 => T6, 7 => T7, 8 => T8);
    impl_sealed_for_tuple!(0 => T0, 1 => T1, 2 => T2, 3 => T3, 4 => T4, 5 => T5, 6 => T6, 7 => T7, 8 => T8, 9 => T9);
    impl_sealed_for_tuple!(0 => T0, 1 => T1, 2 => T2, 3 => T3, 4 => T4, 5 => T5, 6 => T6, 7 => T7, 8 => T8, 9 => T9, 10 => T10);
    impl_sealed_for_tuple!(0 => T0, 1 => T1, 2 => T2, 3 => T3, 4 => T4, 5 => T5, 6 => T6, 7 => T7, 8 => T8, 9 => T9, 10 => T10, 11 => T11);
}
