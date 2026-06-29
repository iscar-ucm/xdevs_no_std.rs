use crate::Component;

use super::common::{AtomicModel, Devstone, JobGenerator};

/// Output struct for HO models (ref version)
#[derive(Debug, Default, crate::Bag)]
pub struct HOModelOutput<const W: usize> {
    output_port_1: xdevs::Port<usize, 1>,
    output_port_2: xdevs::Port<usize, W>,
}

/// Leaf coupled model with only one atomic in HO models (ref version)
#[crate::coupled]
pub struct LeafModel<const W: usize> {
    atomic: AtomicModel,
}

impl<const W: usize> crate::Component for LeafModel<W> {
    type Kind = crate::CoupledKind;
    type Input = crate::Port<usize, 1>;
    type Output = HOModelOutput<W>;
}

impl<const W: usize> crate::Coupled for LeafModel<W> {
    fn eic(from: &Self::Input, to: &mut <Self::Components as crate::Component>::Input) {
        let _ = from.couple(&mut to.atomic);
    }
    fn eoc(from: &<Self::Components as crate::Component>::Output, to: &mut Self::Output) {
        let _ = from.atomic.couple(&mut to.output_port_1);
    }
}

impl<const W: usize> Default for LeafModel<W> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const W: usize> LeafModel<W> {
    pub fn new() -> Self {
        Self::build(AtomicModel::default())
    }
}

impl<const W: usize> Devstone for LeafModel<W> {
    crate::impl_devstone_leaf!();
}

/// HO model enum (ref version)
#[crate::to_component]
pub enum HOEnum<'a, const W: usize> {
    Leaf(LeafModel<W>),
    Branch(HOModel<'a, W>),
}

impl<'a, const W: usize> Devstone for HOEnum<'a, W> {
    crate::impl_devstone_enum!();
}

/// HO coupled model (ref version)
#[crate::coupled]
pub struct HOModel<'a, const W: usize> {
    atomics: [AtomicModel; W],
    inner: &'a mut HOEnum<'a, W>,
}

impl<'a, const W: usize> HOModel<'a, W> {
    pub fn new(inner: &'a mut HOEnum<'a, W>) -> Self {
        Self::build(core::array::from_fn(|_| AtomicModel::default()), inner)
    }
}

impl<'a, const W: usize> Devstone for HOModel<'a, W> {
    crate::impl_devstone_coupled!();
}

impl<'a, const W: usize> crate::Component for HOModel<'a, W> {
    type Kind = crate::CoupledKind;
    type Input = crate::Port<usize, 1>;
    type Output = HOModelOutput<W>;
}

impl<'a, const W: usize> crate::Coupled for HOModel<'a, W> {
    fn eic(from: &Self::Input, to: &mut <Self::Components as crate::Component>::Input) {
        let _ = from.couple(&mut to.inner);
        for atom_ports in to.atomics.iter_mut() {
            let _ = from.couple(atom_ports);
        }
    }

    fn eoc(from: &<Self::Components as crate::Component>::Output, to: &mut Self::Output) {
        let _ = from.inner.output_port_1.couple(&mut to.output_port_1);
        for atom_output_ports in from.atomics.iter() {
            let _ = atom_output_ports.couple(&mut to.output_port_2);
        }
    }

    fn ic(
        from: &<Self::Components as crate::Component>::Output,
        to: &mut <Self::Components as crate::Component>::Input,
    ) {
        for i in 0..(W.saturating_sub(1)) {
            let _ = from.atomics[i].couple(&mut to.atomics[i + 1]);
        }
    }
}

/// End model with Generator and HO model coupled together (ref version)
#[crate::coupled]
pub struct TopModel<'a, const W: usize> {
    generator: JobGenerator,
    ho_model: &'a mut HOEnum<'a, W>,
}

impl<'a, const W: usize> Component for TopModel<'a, W> {
    type Kind = crate::CoupledKind;
    type Input = crate::Port<usize, 1>;
    type Output = crate::Port<usize, 1>;
}

impl<'a, const W: usize> Devstone for TopModel<'a, W> {
    crate::impl_devstone_top!(ho_model);
}

impl<'a, const W: usize> crate::Coupled for TopModel<'a, W> {
    fn ic(
        from: &<Self::Components as crate::Component>::Output,
        to: &mut <Self::Components as crate::Component>::Input,
    ) {
        let _ = from.generator.couple(&mut to.ho_model);
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::simulation::AbstractSimulator;

    fn expected_n_atomic(width: usize, depth: usize) -> usize {
        (width - 1) * (depth - 1) + 1
    }

    fn expected_n_events(width: usize, depth: usize) -> usize {
        1 + (depth - 1) * ((width - 1) * width) / 2
    }

    #[test]
    fn simulation_matches_expected_counts() {
        use crate::simulation::Simulable;
        const WIDTH: usize = 10;
        const DEPTH: usize = 10;
        const W: usize = WIDTH - 1;

        crate::generate_ho!(10, 10);

        let generator = JobGenerator::new(5);
        let top_model: TopModel<'_, W> = TopModel::build(generator, &mut model_ho);
        let mut simulator = top_model.to_simulator();
        let config = crate::simulation::Config::new(0.0, 10.0, 1.0, None);
        simulator.simulate_vt(&config);

        assert_eq!(expected_n_atomic(WIDTH, DEPTH), simulator.get_n_atomics());
        assert_eq!(expected_n_events(WIDTH, DEPTH), simulator.get_n_events());
        assert_eq!(simulator.get_n_internals(), simulator.get_n_externals());
    }

    #[test]
    fn leaf_model_contains_single_atomic() {
        // Verify that the LeafModel contains exactly one atomic model independent of the width parameter
        assert_eq!(LeafModel::<5>::default().get_n_atomics(), 1);
    }
}
