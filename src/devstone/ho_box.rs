use super::common::{AtomicModel, Devstone, JobGenerator};
use alloc::boxed::Box;
use xdevs::Component;

/// Output struct for HO models
#[derive(Debug, Default, xdevs::Bag)]
pub struct HOModelOutput<const W: usize> {
    pub output_port_1: xdevs::Port<usize, 1>,
    pub output_port_2: xdevs::Port<usize, W>,
}

/// Leaf coupled model with only one atomic in HO models
#[xdevs::coupled]
pub struct LeafModel<const W: usize> {
    atomic: AtomicModel,
}

impl<const W: usize> xdevs::Component for LeafModel<W> {
    type Kind = xdevs::CoupledKind;
    type Input = xdevs::Port<usize, 1>;
    type Output = HOModelOutput<W>;
}

impl<const W: usize> xdevs::Coupled for LeafModel<W> {
    fn eic(from: &Self::Input, to: &mut xdevs::ComponentsInput<Self>) {
        let _ = from.couple(&mut to.atomic);
    }
    fn eoc(from: &xdevs::ComponentsOutput<Self>, to: &mut Self::Output) {
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

/// HO model enum
#[xdevs::to_component]
pub enum HOEnum<const W: usize> {
    Leaf(LeafModel<W>),
    Branch(HOModel<W>),
}

impl<const W: usize> Devstone for HOEnum<W> {
    crate::impl_devstone_enum!();
}

/// HO coupled model
#[xdevs::coupled]
pub struct HOModel<const W: usize> {
    atomics: [AtomicModel; W],
    inner: Box<HOEnum<W>>,
}
impl<const W: usize> HOModel<W> {
    pub fn new(inner: Box<HOEnum<W>>) -> Self {
        Self::build(core::array::from_fn(|_| AtomicModel::default()), inner)
    }
}

impl<const W: usize> Devstone for HOModel<W> {
    crate::impl_devstone_coupled!();
}
impl<const W: usize> xdevs::Component for HOModel<W> {
    type Kind = xdevs::CoupledKind;
    type Input = xdevs::Port<usize, 1>;
    type Output = HOModelOutput<W>;
}

impl<const W: usize> xdevs::Coupled for HOModel<W> {
    fn eic(from: &Self::Input, to: &mut xdevs::ComponentsInput<Self>) {
        let _ = from.couple(&mut to.inner);
        for atom_ports in to.atomics.iter_mut() {
            let _ = from.couple(atom_ports);
        }
    }

    fn eoc(from: &xdevs::ComponentsOutput<Self>, to: &mut Self::Output) {
        let _ = from.inner.output_port_1.couple(&mut to.output_port_1);
        for atom_output_ports in from.atomics.iter() {
            let _ = atom_output_ports.couple(&mut to.output_port_2);
        }
    }

    fn ic(from: &xdevs::ComponentsOutput<Self>, to: &mut xdevs::ComponentsInput<Self>) {
        for i in 0..(W.saturating_sub(1)) {
            let _ = from.atomics[i].couple(&mut to.atomics[i + 1]);
        }
    }
}

/// End model with Generator and HO model coupled together
#[xdevs::coupled]
pub struct TopModel<const W: usize> {
    generator: JobGenerator,
    ho_model: HOEnum<W>,
}

impl<const W: usize> Component for TopModel<W> {
    type Kind = xdevs::CoupledKind;
    type Input = xdevs::Port<usize, 1>;
    type Output = xdevs::Port<usize, 1>;
}

impl<const W: usize> Devstone for TopModel<W> {
    crate::impl_devstone_top!(ho_model);
}

impl<const W: usize> xdevs::Coupled for TopModel<W> {
    fn ic(from: &xdevs::ComponentsOutput<Self>, to: &mut xdevs::ComponentsInput<Self>) {
        let _ = from.generator.couple(&mut to.ho_model);
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
        use xdevs::{AbstractSimulator, Simulable};

        const WIDTH: usize = 10;
        const DEPTH: usize = 10;
        const W: usize = WIDTH - 1;

        xdevs::generate_ho_box!(10, 10);

        let generator = JobGenerator::new(5);
        let top_model: TopModel<W> = TopModel::build(generator, model_ho);
        let mut simulator = top_model.to_simulator();
        let config = xdevs::Config::new(0.0, 10.0, 1.0, None);
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
