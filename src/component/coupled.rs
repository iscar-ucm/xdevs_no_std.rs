use crate::{
    component::{Component, CoupledKind},
    simulation::SimpleSimulable,
};

/// Partial interface for DEVS coupled models. All DEVS coupled models must implement this trait.
pub trait PartialCoupled: Component<Kind = CoupledKind> {
    /// Type of the inner components of this coupled model.
    type Components: SimpleSimulable;

    fn get_components(&self) -> &Processors<Self>;

    fn get_components_mut(&mut self) -> &mut Processors<Self>;
}

/// Type alias for the inner components of a coupled model.
pub type Components<T> = <T as PartialCoupled>::Components;

/// Type alias for the simulator of the inner components of a coupled model.
pub type Processors<T> = <<T as PartialCoupled>::Components as SimpleSimulable>::Simulator;

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

#[cfg(test)]
mod tests {
    use xdevs_no_std_macros::coupled;

    use super::{ComponentsInput, ComponentsOutput, Coupled, PartialCoupled};
    use crate::{
        component::CoupledKind,
        gpt::Processor,
        port::{Bag, Port},
        Component,
    };

    #[coupled]
    struct ForwardChain {
        components: [Processor; 2],
    }

    impl Component for ForwardChain {
        type Kind = CoupledKind;
        type Input = Port<usize, 1>;
        type Output = Port<usize, 1>;
    }

    impl Coupled for ForwardChain {
        fn eic(from: &Self::Input, to: &mut ComponentsInput<Self>) {
            let _ = from.couple(&mut to.components[0]);
        }
        fn ic(from: &ComponentsOutput<Self>, to: &mut ComponentsInput<Self>) {
            let _ = from.components[0].couple(&mut to.components[1]);
        }
        fn eoc(from: &ComponentsOutput<Self>, to: &mut Self::Output) {
            let _ = from.components[1].couple(to);
        }
    }

    #[test]
    fn forwarding_pipeline() {
        let mut comp_in = ForwardChainComponentsInput::build();
        let mut comp_out = ForwardChainComponentsOutput::build();
        let mut input = Port::<usize, 1>::new();
        let mut output = Port::<usize, 1>::new();
        input.add_value(99).unwrap();

        ForwardChain::eic(&input, &mut comp_in);
        assert_eq!(
            comp_in.components[0].get_values(),
            &[99],
            "value flows through input → 0"
        );

        // Simulate children lambda: populate comp_out with the forwarded value
        comp_out.components[0].add_value(99).unwrap();
        ForwardChain::ic(&comp_out, &mut comp_in);
        assert_eq!(
            comp_in.components[1].get_values(),
            &[99],
            "value flows through 0 → 1"
        );

        // Simulate children lambda again: populate comp_out[1]
        comp_out.components[1].add_value(99).unwrap();
        ForwardChain::eoc(&comp_out, &mut output);
        assert_eq!(output.get_values(), &[99], "value flows through 1 → output");
    }

    #[test]
    fn ref_mut_delegates_all_coupled() {
        // Coupled methods: verify values flow through &mut T blanket
        let mut comp_in = ForwardChainComponentsInput::build();
        let mut comp_out = ForwardChainComponentsOutput::build();
        let mut input = Port::<usize, 1>::new();
        let mut output = Port::<usize, 1>::new();
        input.add_value(99).unwrap();

        <&mut ForwardChain as Coupled>::eic(&input, &mut comp_in);
        assert_eq!(
            comp_in.components[0].get_values(),
            &[99],
            "eic delegates through &mut T"
        );

        comp_out.components[0].add_value(99).unwrap();
        <&mut ForwardChain as Coupled>::ic(&comp_out, &mut comp_in);
        assert_eq!(
            comp_in.components[1].get_values(),
            &[99],
            "ic delegates through &mut T"
        );

        comp_out.components[1].add_value(99).unwrap();
        <&mut ForwardChain as Coupled>::eoc(&comp_out, &mut output);
        assert_eq!(output.get_values(), &[99], "eoc delegates through &mut T");

        // PartialCoupled: get_components through &mut T blanket
        let mut model = ForwardChain::build([Processor::new(1.), Processor::new(1.)]);
        let mut r: &mut ForwardChain = &mut model;
        let addr_real = &r.components as *const _ as usize;

        let comps = <&mut ForwardChain as PartialCoupled>::get_components(&r);
        assert_eq!(
            comps as *const _ as usize, addr_real,
            "get_components through &mut T"
        );

        let comps_mut = <&mut ForwardChain as PartialCoupled>::get_components_mut(&mut r);
        assert_eq!(
            comps_mut as *const _ as usize, addr_real,
            "get_components_mut through &mut T"
        );
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn box_delegates_all_coupled() {
        use alloc::boxed::Box;

        // Coupled methods: verify values flow through Box<T> blanket
        let mut comp_in = ForwardChainComponentsInput::build();
        let mut comp_out = ForwardChainComponentsOutput::build();
        let mut input = Port::<usize, 1>::new();
        let mut output = Port::<usize, 1>::new();
        input.add_value(99).unwrap();

        <Box<ForwardChain> as Coupled>::eic(&input, &mut comp_in);
        assert_eq!(
            comp_in.components[0].get_values(),
            &[99],
            "eic delegates through Box<T>"
        );

        comp_out.components[0].add_value(99).unwrap();
        <Box<ForwardChain> as Coupled>::ic(&comp_out, &mut comp_in);
        assert_eq!(
            comp_in.components[1].get_values(),
            &[99],
            "ic delegates through Box<T>"
        );

        comp_out.components[1].add_value(99).unwrap();
        <Box<ForwardChain> as Coupled>::eoc(&comp_out, &mut output);
        assert_eq!(output.get_values(), &[99], "eoc delegates through Box<T>");

        // PartialCoupled: get_components through Box<T> blanket
        let mut model = Box::new(ForwardChain::build([
            Processor::new(1.),
            Processor::new(1.),
        ]));

        let comps = <Box<ForwardChain> as PartialCoupled>::get_components(&model);
        assert_eq!(
            comps as *const _ as usize, &model.components as *const _ as usize,
            "get_components through Box<T>"
        );

        let comps_mut = <Box<ForwardChain> as PartialCoupled>::get_components_mut(&mut model);
        assert_eq!(
            comps_mut as *const _ as usize, &model.components as *const _ as usize,
            "get_components_mut through Box<T>"
        );
    }
}
