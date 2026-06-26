use super::common::{AtomicModel, Devstone, JobGenerator, LeafModel};
use crate::Component;
use alloc::boxed::Box;

/// HI model enum
#[xdevs::to_component]
pub enum HIEnum<const W: usize> {
    Leaf(LeafModel),
    Branch(HIModel<W>),
}

impl<const W: usize> Devstone for HIEnum<W> {
    crate::impl_devstone_enum!();
}

/// HI coupled model
#[xdevs::to_component]
pub struct HIModel<const W: usize> {
    atomics: [AtomicModel; W],
    inner: Box<HIEnum<W>>,
}

impl<const W: usize> xdevs::Component for HIModel<W> {
    type Kind = xdevs::CoupledKind;
    type Input = xdevs::Port<usize, 1>;
    type Output = xdevs::Port<usize, 1>;
}

impl<const W: usize> xdevs::Coupled for HIModel<W> {
    fn eic(from: &Self::Input, to: &mut <Self::Components as xdevs::Component>::Input) {
        for atom_ports in to.atomics.iter_mut() {
            let _ = from.couple(atom_ports);
        }

        let _ = from.couple(&mut to.inner);
    }

    fn eoc(from: &<Self::Components as xdevs::Component>::Output, to: &mut Self::Output) {
        let _ = from.inner.couple(to);
    }

    fn ic(
        from: &<Self::Components as xdevs::Component>::Output,
        to: &mut <Self::Components as xdevs::Component>::Input,
    ) {
        for i in 0..(W.saturating_sub(1)) {
            let _ = from.atomics[i].couple(&mut to.atomics[i + 1]);
        }
    }
}

impl<const W: usize> HIModel<W> {
    pub fn new(inner: Box<HIEnum<W>>) -> Self {
        Self::build(core::array::from_fn(|_| AtomicModel::default()), inner)
    }
}

impl<const W: usize> Devstone for HIModel<W> {
    crate::impl_devstone_coupled!();
}

/// End model with Generator and HI model coupled together
#[xdevs::to_component]
pub struct TopModel<const W: usize> {
    generator: JobGenerator,
    hi_model: HIEnum<W>,
}

impl<const W: usize> Component for TopModel<W> {
    type Kind = xdevs::CoupledKind;
    type Input = xdevs::Port<usize, 1>;
    type Output = xdevs::Port<usize, 1>;
}
impl<const W: usize> Devstone for TopModel<W> {
    crate::impl_devstone_top!(hi_model);
}

impl<const W: usize> xdevs::Coupled for TopModel<W> {
    fn ic(
        from: &<Self::Components as xdevs::Component>::Output,
        to: &mut <Self::Components as xdevs::Component>::Input,
    ) {
        let _ = from.generator.couple(&mut to.hi_model);
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn expected_n_atomic(width: usize, depth: usize) -> usize {
        (width - 1) * (depth - 1) + 1
    }

    fn expected_n_events(width: usize, depth: usize) -> usize {
        1 + (depth - 1) * ((width - 1) * width) / 2
    }

    #[test]
    fn simulation_matches_expected_counts() {
        use xdevs::simulation::{AbstractSimulator, Simulable};

        const WIDTH: usize = 10;
        const DEPTH: usize = 10;
        const W: usize = WIDTH - 1;

        xdevs::generate_hi_box!(10, 10);

        let generator = JobGenerator::new(5);
        let top_model: TopModel<W> = TopModel::build(generator, model_hi);
        let mut simulator = top_model.to_simulator();
        let config = xdevs::simulation::Config::new(0.0, 10.0, 1.0, None);
        simulator.simulate_vt(&config);

        assert_eq!(expected_n_atomic(WIDTH, DEPTH), simulator.get_n_atomics());
        assert_eq!(expected_n_events(WIDTH, DEPTH), simulator.get_n_events());
        assert_eq!(simulator.get_n_internals(), simulator.get_n_externals());
    }
}
