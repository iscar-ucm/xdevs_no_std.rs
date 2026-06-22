use crate::{simulation::coordinator::Coordinator, AbstractSimulator, Component};

use super::common::{AtomicModel, JobGenerator};

/// Output struct for HO models (ref version)
#[derive(Debug, Default, crate::Bag)]
pub struct HOModelOutputRef<const W: usize> {
    pub output_port_1: crate::Port<usize, 1>,
    pub output_port_2: crate::Port<usize, W>,
}

/// Leaf coupled model with only one atomic in HO models (ref version)
#[crate::coupled]
pub struct LeafModelRef<const W: usize> {
    atomic: AtomicModel,
}

impl<const W: usize> crate::Component for LeafModelRef<W> {
    type Kind = crate::CoupledKind;
    type Input = crate::Port<usize, 1>;
    type Output = HOModelOutputRef<W>;
}

impl<const W: usize> crate::Coupled for LeafModelRef<W> {
    fn eic(from: &Self::Input, to: &mut <Self::Components as crate::Component>::Input) {
        let _ = from.couple(&mut to.atomic);
    }
    fn eoc(from: &<Self::Components as crate::Component>::Output, to: &mut Self::Output) {
        let _ = from.atomic.couple(&mut to.output_port_1);
    }
}

impl<const W: usize> Default for LeafModelRef<W> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const W: usize> LeafModelRef<W> {
    pub fn new() -> Self {
        Self::build(AtomicModel::default())
    }

    pub fn get_n_internals(&self) -> usize {
        self.components.atomic.get_n_internals()
    }

    pub fn get_n_externals(&self) -> usize {
        self.components.atomic.get_n_externals()
    }

    pub fn get_n_events(&self) -> usize {
        self.components.atomic.get_n_events()
    }

    pub fn get_n_atomics(&self) -> usize {
        self.components.atomic.get_n_atomics()
    }
}

/// HO model enum (ref version)
pub enum HOEnumRef<'a, const W: usize> {
    Leaf(Coordinator<LeafModelRef<W>>),
    Branch(Coordinator<HOModelRef<'a, W>>),
}

impl<'a, const W: usize> HOEnumRef<'a, W> {
    pub fn get_n_internals(&self) -> usize {
        match self {
            HOEnumRef::Leaf(leaf) => leaf.get_n_internals(),
            HOEnumRef::Branch(branch) => branch.get_n_internals(),
        }
    }

    pub fn get_n_externals(&self) -> usize {
        match self {
            HOEnumRef::Leaf(leaf) => leaf.get_n_externals(),
            HOEnumRef::Branch(branch) => branch.get_n_externals(),
        }
    }

    pub fn get_n_events(&self) -> usize {
        match self {
            HOEnumRef::Leaf(leaf) => leaf.get_n_events(),
            HOEnumRef::Branch(branch) => branch.get_n_events(),
        }
    }

    pub fn get_n_atomics(&self) -> usize {
        match self {
            HOEnumRef::Leaf(leaf) => leaf.get_n_atomics(),
            HOEnumRef::Branch(branch) => branch.get_n_atomics(),
        }
    }
}

/// Manual implementation of `Component` for HO enum (ref version)
impl<'a, const W: usize> Component for HOEnumRef<'a, W> {
    type Kind = crate::component::ComponentsKind;
    type Input = crate::Port<usize, 1>;
    type Output = HOModelOutputRef<W>;
}

/// Manual implementation of `AbstractSimulator` for HO enum (ref version)
unsafe impl<'a, const W: usize> AbstractSimulator for HOEnumRef<'a, W> {
    type Input = crate::Port<usize, 1>;
    type Output = HOModelOutputRef<W>;

    fn start(&mut self, t_start: f64) -> f64 {
        match self {
            HOEnumRef::Leaf(leaf) => {
                <Coordinator<LeafModelRef<W>> as AbstractSimulator>::start(leaf, t_start)
            }
            HOEnumRef::Branch(branch) => {
                <Coordinator<HOModelRef<'a, W>> as AbstractSimulator>::start(branch, t_start)
            }
        }
    }

    fn stop(&mut self) {
        match self {
            HOEnumRef::Leaf(leaf) => {
                <Coordinator<LeafModelRef<W>> as AbstractSimulator>::stop(leaf)
            }
            HOEnumRef::Branch(branch) => {
                <Coordinator<HOModelRef<'a, W>> as AbstractSimulator>::stop(branch)
            }
        }
    }

    fn lambda(&mut self, output: &mut Self::Output, t: f64) {
        match self {
            HOEnumRef::Leaf(leaf) => {
                <Coordinator<LeafModelRef<W>> as AbstractSimulator>::lambda(leaf, output, t)
            }
            HOEnumRef::Branch(branch) => {
                <Coordinator<HOModelRef<'a, W>> as AbstractSimulator>::lambda(branch, output, t)
            }
        }
    }

    fn delta(&mut self, input: &mut Self::Input, output: &mut Self::Output, t: f64) -> f64 {
        match self {
            HOEnumRef::Leaf(leaf) => {
                <Coordinator<LeafModelRef<W>> as AbstractSimulator>::delta(leaf, input, output, t)
            }
            HOEnumRef::Branch(branch) => {
                <Coordinator<HOModelRef<'a, W>> as AbstractSimulator>::delta(
                    branch, input, output, t,
                )
            }
        }
    }
}

/// HO coupled model (ref version)
#[crate::coupled]
pub struct HOModelRef<'a, const W: usize> {
    atomics: [AtomicModel; W],
    inner: &'a mut HOEnumRef<'a, W>,
}

impl<'a, const W: usize> HOModelRef<'a, W> {
    pub fn new(inner: &'a mut HOEnumRef<'a, W>) -> Self {
        Self::build(core::array::from_fn(|_| AtomicModel::default()), inner)
    }

    pub fn get_n_internals(&self) -> usize {
        let mut sum_int = self.components.inner.get_n_internals();
        for atomic in self.components.atomics.iter() {
            sum_int += atomic.get_n_internals();
        }
        sum_int
    }

    pub fn get_n_externals(&self) -> usize {
        let mut sum_ext = self.components.inner.get_n_externals();
        for atomic in self.components.atomics.iter() {
            sum_ext += atomic.get_n_externals();
        }
        sum_ext
    }

    pub fn get_n_events(&self) -> usize {
        let mut sum_ev = self.components.inner.get_n_events();
        for atomic in self.components.atomics.iter() {
            sum_ev += atomic.get_n_events();
        }
        sum_ev
    }

    pub fn get_n_atomics(&self) -> usize {
        let mut sum_atomic = self.components.inner.get_n_atomics();
        for _atomic in self.components.atomics.iter() {
            sum_atomic += 1;
        }
        sum_atomic
    }
}

impl<'a, const W: usize> crate::Component for HOModelRef<'a, W> {
    type Kind = crate::CoupledKind;
    type Input = crate::Port<usize, 1>;
    type Output = HOModelOutputRef<W>;
}

impl<'a, const W: usize> crate::Coupled for HOModelRef<'a, W> {
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
pub struct TopModelRef<'a, const W: usize> {
    generator: JobGenerator,
    ho_model: &'a mut HOEnumRef<'a, W>,
}

impl<'a, const W: usize> Component for TopModelRef<'a, W> {
    type Kind = crate::CoupledKind;
    type Input = crate::Port<usize, 1>;
    type Output = crate::Port<usize, 1>;
}

impl<'a, const W: usize> TopModelRef<'a, W> {
    pub fn get_n_internals(&self) -> usize {
        self.components.ho_model.get_n_internals()
    }

    pub fn get_n_externals(&self) -> usize {
        self.components.ho_model.get_n_externals()
    }

    pub fn get_n_events(&self) -> usize {
        self.components.ho_model.get_n_events()
    }

    pub fn get_n_atomics(&self) -> usize {
        self.components.ho_model.get_n_atomics()
    }
}

impl<'a, const W: usize> crate::Coupled for TopModelRef<'a, W> {
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

        crate::generate_ho_ref!(10, 10);

        let generator = JobGenerator::new(5);
        let top_model: TopModelRef<'_, W> = TopModelRef::build(generator, &mut model_ho);
        let mut simulator = top_model.to_simulator();
        let config = crate::simulation::Config::new(0.0, 10.0, 1.0, None);
        simulator.simulate_vt(&config);

        assert_eq!(expected_n_atomic(WIDTH, DEPTH), simulator.get_n_atomics());
        assert_eq!(expected_n_events(WIDTH, DEPTH), simulator.get_n_events());
        assert_eq!(simulator.get_n_internals(), simulator.get_n_externals());
    }

    #[test]
    fn leaf_model_contains_single_atomic() {
        // Verify that the LeafModelRef contains exactly one atomic model independent of the width parameter
        assert_eq!(LeafModelRef::<5>::default().get_n_atomics(), 1);
    }
}
