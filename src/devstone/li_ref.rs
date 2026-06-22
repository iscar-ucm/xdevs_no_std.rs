use crate::{simulation::coordinator::Coordinator, AbstractSimulator, Component};

use super::common::*;

/// LI model enum (ref version)
pub enum LIEnumRef<'a, const W: usize> {
    Leaf(Coordinator<LeafModel>),
    Branch(Coordinator<LIModelRef<'a, W>>),
}

impl<'a, const W: usize> LIEnumRef<'a, W> {
    pub fn get_n_internals(&self) -> usize {
        match self {
            LIEnumRef::Leaf(leaf) => leaf.get_n_internals(),
            LIEnumRef::Branch(branch) => branch.get_n_internals(),
        }
    }

    pub fn get_n_externals(&self) -> usize {
        match self {
            LIEnumRef::Leaf(leaf) => leaf.get_n_externals(),
            LIEnumRef::Branch(branch) => branch.get_n_externals(),
        }
    }

    pub fn get_n_events(&self) -> usize {
        match self {
            LIEnumRef::Leaf(leaf) => leaf.get_n_events(),
            LIEnumRef::Branch(branch) => branch.get_n_events(),
        }
    }

    pub fn get_n_atomics(&self) -> usize {
        match self {
            LIEnumRef::Leaf(leaf) => leaf.get_n_atomics(),
            LIEnumRef::Branch(branch) => branch.get_n_atomics(),
        }
    }
}

/// Manual implementation of `Component` for LI enum (ref version)
impl<'a, const W: usize> Component for LIEnumRef<'a, W> {
    type Kind = crate::component::ComponentsKind;
    type Input = crate::Port<usize, 1>;
    type Output = crate::Port<usize, 1>;
}

/// Manual implementation of `AbstractSimulator` for LI enum (ref version)
unsafe impl<'a, const W: usize> AbstractSimulator for LIEnumRef<'a, W> {
    type Input = crate::Port<usize, 1>;
    type Output = crate::Port<usize, 1>;

    fn start(&mut self, t_start: f64) -> f64 {
        match self {
            LIEnumRef::Leaf(leaf) => {
                <Coordinator<LeafModel> as AbstractSimulator>::start(leaf, t_start)
            }
            LIEnumRef::Branch(branch) => {
                <Coordinator<LIModelRef<'a, W>> as AbstractSimulator>::start(branch, t_start)
            }
        }
    }

    fn stop(&mut self) {
        match self {
            LIEnumRef::Leaf(leaf) => <Coordinator<LeafModel> as AbstractSimulator>::stop(leaf),
            LIEnumRef::Branch(branch) => {
                <Coordinator<LIModelRef<'a, W>> as AbstractSimulator>::stop(branch)
            }
        }
    }

    fn lambda(&mut self, output: &mut Self::Output, t: f64) {
        match self {
            LIEnumRef::Leaf(leaf) => {
                <Coordinator<LeafModel> as AbstractSimulator>::lambda(leaf, output, t)
            }
            LIEnumRef::Branch(branch) => {
                <Coordinator<LIModelRef<'a, W>> as AbstractSimulator>::lambda(branch, output, t)
            }
        }
    }

    fn delta(&mut self, input: &mut Self::Input, output: &mut Self::Output, t: f64) -> f64 {
        match self {
            LIEnumRef::Leaf(leaf) => {
                <Coordinator<LeafModel> as AbstractSimulator>::delta(leaf, input, output, t)
            }
            LIEnumRef::Branch(branch) => {
                <Coordinator<LIModelRef<'a, W>> as AbstractSimulator>::delta(
                    branch, input, output, t,
                )
            }
        }
    }
}

/// LI coupled model (ref version)
#[crate::coupled]
pub struct LIModelRef<'a, const W: usize> {
    atomics: [AtomicModel; W],
    inner: &'a mut LIEnumRef<'a, W>,
}

impl<'a, const W: usize> crate::Component for LIModelRef<'a, W> {
    type Kind = crate::CoupledKind;
    type Input = crate::Port<usize, 1>;
    type Output = crate::Port<usize, 1>;
}

impl<'a, const W: usize> crate::Coupled for LIModelRef<'a, W> {
    fn eic(from: &Self::Input, to: &mut <Self::Components as crate::Component>::Input) {
        for atom_ports in to.atomics.iter_mut() {
            let _ = from.couple(atom_ports);
        }

        let _ = from.couple(&mut to.inner);
    }

    fn eoc(from: &<Self::Components as crate::Component>::Output, to: &mut Self::Output) {
        let _ = from.inner.couple(to);
    }
}

impl<'a, const W: usize> LIModelRef<'a, W> {
    pub fn new(inner: &'a mut LIEnumRef<'a, W>) -> Self {
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

/// End model with Generator and LI model coupled together (ref version)
#[crate::coupled]
pub struct TopModelRef<'a, const W: usize> {
    generator: JobGenerator,
    li_model: &'a mut LIEnumRef<'a, W>,
}

impl<'a, const W: usize> Component for TopModelRef<'a, W> {
    type Kind = crate::CoupledKind;
    type Input = ();
    type Output = ();
}

impl<'a, const W: usize> TopModelRef<'a, W> {
    pub fn get_n_internals(&self) -> usize {
        self.components.li_model.get_n_internals()
    }

    pub fn get_n_externals(&self) -> usize {
        self.components.li_model.get_n_externals()
    }

    pub fn get_n_events(&self) -> usize {
        self.components.li_model.get_n_events()
    }

    pub fn get_n_atomics(&self) -> usize {
        self.components.li_model.get_n_atomics()
    }
}

impl<'a, const W: usize> crate::Coupled for TopModelRef<'a, W> {
    fn ic(
        from: &<Self::Components as crate::Component>::Output,
        to: &mut <Self::Components as crate::Component>::Input,
    ) {
        let _ = from.generator.couple(&mut to.li_model);
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn expected_n_atomic(width: usize, depth: usize) -> usize {
        (width - 1) * (depth - 1) + 1
    }

    fn expected_n_events(width: usize, depth: usize) -> usize {
        (width - 1) * (depth - 1) + 1
    }

    #[test]
    fn simulation_matches_expected_counts() {
        use crate::simulation::Simulable;
        const WIDTH: usize = 10;
        const DEPTH: usize = 10;
        const W: usize = WIDTH - 1;

        crate::generate_li_ref!(10, 10);

        let generator = JobGenerator::new(5);
        let top_model: TopModelRef<'_, W> = TopModelRef::build(generator, &mut model_li);
        let mut simulator = top_model.to_simulator();
        let config = crate::simulation::Config::new(0.0, 10.0, 1.0, None);
        simulator.simulate_vt(&config);

        assert_eq!(expected_n_atomic(WIDTH, DEPTH), simulator.get_n_atomics());
        assert_eq!(expected_n_events(WIDTH, DEPTH), simulator.get_n_events());
        assert_eq!(simulator.get_n_internals(), simulator.get_n_externals());
    }
}
