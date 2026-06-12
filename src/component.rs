use xdevs::traits;

/// Marker type for atomic DEVS models.
pub struct AtomicKind;

impl sealed::Sealed for AtomicKind {}

/// Marker type for coupled DEVS models.
pub struct CoupledKind;

impl sealed::Sealed for CoupledKind {}

/// Interface for DEVS components. All DEVS components must implement this trait.
pub trait Component {
    /// Kind of DEVS model. It can be either [`AtomicKind`] or [`CoupledKind`].
    type Kind: sealed::Sealed;

    /// Input event bag of the model.
    type Input: traits::Bag;

    /// Output event bag of the model.
    type Output: traits::Bag;
}

mod sealed {
    /// Trait used to prevent users from implementing certain traits manually.
    pub trait Sealed {}
}
