use super::common::{AtomicModel, Devstone, JobGenerator, LeafModel};
use crate::{Bag, Component, ComponentsInput, ComponentsOutput, Coupled, CoupledKind, Port};
use alloc::boxed::Box;

#[crate::to_component]
pub enum LIEnum<const W: usize> {
    Leaf(LeafModel),
    Branch(LIModel<W>),
}

impl<const W: usize> Devstone for LIEnum<W> {
    crate::impl_devstone_enum!();
}

/// LI coupled model
#[crate::coupled]
pub struct LIModel<const W: usize> {
    atomics: [AtomicModel; W],
    inner: Box<LIEnum<W>>,
}

impl<const W: usize> Component for LIModel<W> {
    type Kind = CoupledKind;
    type Input = Port<usize, 1>;
    type Output = Port<usize, 1>;
}

impl<const W: usize> Coupled for LIModel<W> {
    fn eic(from: &Self::Input, to: &mut ComponentsInput<Self>) {
        for atom_ports in to.atomics.iter_mut() {
            let _ = from.couple(atom_ports);
        }

        let _ = from.couple(&mut to.inner);
    }

    fn eoc(from: &ComponentsOutput<Self>, to: &mut Self::Output) {
        let _ = from.inner.couple(to);
    }
}

impl<const W: usize> LIModel<W> {
    pub fn new(int_delay: u64, ext_delay: u64, inner: Box<LIEnum<W>>) -> Self {
        Self::build(
            core::array::from_fn(|_| AtomicModel::new(int_delay, ext_delay)),
            inner,
        )
    }
}

impl<const W: usize> Devstone for LIModel<W> {
    crate::impl_devstone_coupled!();
}

/// End model with Generator and LI model coupled together
#[crate::coupled]
pub struct TopModel<const W: usize> {
    generator: JobGenerator,
    li_model: LIEnum<W>,
}

impl<const W: usize> Component for TopModel<W> {
    type Kind = CoupledKind;
    type Input = ();
    type Output = ();
}

impl<const W: usize> Devstone for TopModel<W> {
    crate::impl_devstone_top!(li_model, generator);
}

impl<const W: usize> Coupled for TopModel<W> {
    fn ic(from: &ComponentsOutput<Self>, to: &mut ComponentsInput<Self>) {
        let _ = from.generator.couple(&mut to.li_model);
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::prelude::*;

    fn expected_n_atomic(width: usize, depth: usize) -> usize {
        (width - 1) * (depth - 1) + 1
    }

    fn expected_n_events(width: usize, depth: usize) -> usize {
        (width - 1) * (depth - 1) + 1
    }

    #[test]
    fn simulation_matches_expected_counts_and_resets() {
        const WIDTH: usize = 10;
        const DEPTH: usize = 10;
        const W: usize = WIDTH - 1;

        crate::generate_li_box!(10, 10, 0, 0);

        let generator = JobGenerator::new(5);
        let top_model: TopModel<W> = TopModel::build(generator, model_li);
        let mut simulator = top_model.to_simulator();
        let config = crate::Config::new(0.0, 10.0, 1.0, None);
        simulator.simulate_vt(&config);

        assert_eq!(expected_n_atomic(WIDTH, DEPTH), simulator.get_n_atomics());
        assert_eq!(expected_n_events(WIDTH, DEPTH), simulator.get_n_events());
        assert_eq!(simulator.get_n_internals(), simulator.get_n_externals());

        simulator.reset();

        assert_eq!(expected_n_atomic(WIDTH, DEPTH), simulator.get_n_atomics());
        assert_eq!(0, simulator.get_n_events());
        assert_eq!(0, simulator.get_n_internals());
        assert_eq!(0, simulator.get_n_externals());
    }
}
