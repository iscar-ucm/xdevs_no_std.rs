use super::common::{AtomicModel, Devstone, JobGenerator, LeafModel};
use crate::{Bag, Component, ComponentsInput, ComponentsOutput, Coupled, CoupledKind, Port};
/// LI model enum (ref version)
#[crate::to_component]
pub enum LIEnum<'a, const W: usize> {
    Leaf(LeafModel),
    Branch(LIModel<'a, W>),
}

impl<'a, const W: usize> Devstone for LIEnum<'a, W> {
    crate::impl_devstone_enum!();
}

/// LI coupled model (ref version)
#[crate::coupled]
pub struct LIModel<'a, const W: usize> {
    atomics: [AtomicModel; W],
    inner: &'a mut LIEnum<'a, W>,
}

impl<'a, const W: usize> Component for LIModel<'a, W> {
    type Kind = CoupledKind;
    type Input = Port<usize, 1>;
    type Output = Port<usize, 1>;
}

impl<'a, const W: usize> Coupled for LIModel<'a, W> {
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

impl<'a, const W: usize> LIModel<'a, W> {
    pub fn new(int_delay: u64, ext_delay: u64, inner: &'a mut LIEnum<'a, W>) -> Self {
        Self::build(
            core::array::from_fn(|_| AtomicModel::new(int_delay, ext_delay)),
            inner,
        )
    }
}

impl<'a, const W: usize> Devstone for LIModel<'a, W> {
    crate::impl_devstone_coupled!();
}

/// End model with Generator and LI model coupled together (ref version)
#[crate::coupled]
pub struct TopModel<'a, const W: usize> {
    generator: JobGenerator,
    li_model: &'a mut LIEnum<'a, W>,
}

impl<'a, const W: usize> Component for TopModel<'a, W> {
    type Kind = CoupledKind;
    type Input = ();
    type Output = ();
}

impl<'a, const W: usize> Devstone for TopModel<'a, W> {
    crate::impl_devstone_top!(li_model, generator);
}

impl<'a, const W: usize> Coupled for TopModel<'a, W> {
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

        crate::generate_li!(10, 10, 1, 1);

        let generator = JobGenerator::new(5);
        let top_model: TopModel<'_, W> = TopModel::build(generator, &mut model_li);
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
