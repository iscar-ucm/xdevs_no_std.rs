use xdevs::traits;

/// Interface for DEVS coupled models. All DEVS coupled models must implement this trait.
pub trait Coupled: traits::PartialCoupled {
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
