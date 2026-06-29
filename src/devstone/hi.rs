use super::common::{AtomicModel, Devstone, JobGenerator, LeafModel};
use crate::Component;

/// HI model enum (ref version)
#[crate::to_component]
pub enum HIEnum<'a, const W: usize> {
    Leaf(LeafModel),
    Branch(HIModel<'a, W>),
}

impl<'a, const W: usize> Devstone for HIEnum<'a, W> {
    crate::impl_devstone_enum!();
}

/// HI coupled model (ref version)
#[crate::coupled]
pub struct HIModel<'a, const W: usize> {
    atomics: [AtomicModel; W],
    inner: &'a mut HIEnum<'a, W>,
}

impl<'a, const W: usize> crate::Component for HIModel<'a, W> {
    type Kind = crate::CoupledKind;
    type Input = crate::Port<usize, 1>;
    type Output = crate::Port<usize, 1>;
}

impl<'a, const W: usize> crate::Coupled for HIModel<'a, W> {
    fn eic(from: &Self::Input, to: &mut <Self::Components as crate::Component>::Input) {
        for atom_ports in to.atomics.iter_mut() {
            let _ = from.couple(atom_ports);
        }

        let _ = from.couple(&mut to.inner);
    }

    fn eoc(from: &<Self::Components as crate::Component>::Output, to: &mut Self::Output) {
        let _ = from.inner.couple(to);
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

impl<'a, const W: usize> HIModel<'a, W> {
    pub fn new(inner: &'a mut HIEnum<'a, W>) -> Self {
        Self::build(core::array::from_fn(|_| AtomicModel::default()), inner)
    }
}

impl<'a, const W: usize> Devstone for HIModel<'a, W> {
    crate::impl_devstone_coupled!();
}

/// End model with Generator and HI model coupled together (ref version)
#[crate::coupled]
pub struct TopModel<'a, const W: usize> {
    generator: JobGenerator,
    hi_model: &'a mut HIEnum<'a, W>,
}

impl<'a, const W: usize> Component for TopModel<'a, W> {
    type Kind = crate::CoupledKind;
    type Input = crate::Port<usize, 1>;
    type Output = crate::Port<usize, 1>;
}

impl<'a, const W: usize> Devstone for TopModel<'a, W> {
    crate::impl_devstone_top!(hi_model);
}

impl<'a, const W: usize> crate::Coupled for TopModel<'a, W> {
    fn ic(
        from: &<Self::Components as crate::Component>::Output,
        to: &mut <Self::Components as crate::Component>::Input,
    ) {
        let _ = from.generator.couple(&mut to.hi_model);
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

        crate::generate_hi!(10, 10);

        let generator = JobGenerator::new(5);
        let top_model: TopModel<'_, W> = TopModel::build(generator, &mut model_hi);
        let mut simulator = top_model.to_simulator();
        let config = crate::simulation::Config::new(0.0, 10.0, 1.0, None);
        simulator.simulate_vt(&config);

        assert_eq!(expected_n_atomic(WIDTH, DEPTH), simulator.get_n_atomics());
        assert_eq!(expected_n_events(WIDTH, DEPTH), simulator.get_n_events());
        assert_eq!(simulator.get_n_internals(), simulator.get_n_externals());
    }
}
