use super::common::*;
use crate::{simulation::coordinator::Coordinator, AbstractSimulator, Component};
use alloc::boxed::Box;

/// HI model enum
pub enum HIEnum<const W: usize> {
    Leaf(Coordinator<LeafModel>),
    Branch(Coordinator<HIModel<W>>),
}

impl<const W: usize> HIEnum<W> {
    pub fn get_n_internals(&self) -> usize {
        match self {
            HIEnum::Leaf(leaf) => leaf.get_n_internals(),
            HIEnum::Branch(branch) => branch.get_n_internals(),
        }
    }

    pub fn get_n_externals(&self) -> usize {
        match self {
            HIEnum::Leaf(leaf) => leaf.get_n_externals(),
            HIEnum::Branch(branch) => branch.get_n_externals(),
        }
    }

    pub fn get_n_events(&self) -> usize {
        match self {
            HIEnum::Leaf(leaf) => leaf.get_n_events(),
            HIEnum::Branch(branch) => branch.get_n_events(),
        }
    }

    pub fn get_n_atomics(&self) -> usize {
        match self {
            HIEnum::Leaf(leaf) => leaf.get_n_atomics(),
            HIEnum::Branch(branch) => branch.get_n_atomics(),
        }
    }
}

/// Manual implementation of `Component` for HI enum
impl<const W: usize> Component for HIEnum<W> {
    type Kind = xdevs::component::ComponentsKind;
    type Input = xdevs::Port<usize, 1>;
    type Output = xdevs::Port<usize, 1>;
}

/// Manual implementation of `AbstractSimulator` for HI enum
unsafe impl<const W: usize> AbstractSimulator for HIEnum<W> {
    type Input = xdevs::Port<usize, 1>;

    type Output = xdevs::Port<usize, 1>;

    fn start(&mut self, t_start: f64) -> f64 {
        match self {
            HIEnum::Leaf(leaf) => {
                <Coordinator<LeafModel> as AbstractSimulator>::start(leaf, t_start)
            }
            HIEnum::Branch(branch) => {
                <Coordinator<HIModel<W>> as AbstractSimulator>::start(branch, t_start)
            }
        }
    }

    fn stop(&mut self) {
        match self {
            HIEnum::Leaf(leaf) => <Coordinator<LeafModel> as AbstractSimulator>::stop(leaf),
            HIEnum::Branch(branch) => <Coordinator<HIModel<W>> as AbstractSimulator>::stop(branch),
        }
    }

    fn lambda(&mut self, output: &mut Self::Output, t: f64) {
        match self {
            HIEnum::Leaf(leaf) => {
                <Coordinator<LeafModel> as AbstractSimulator>::lambda(leaf, output, t)
            }
            HIEnum::Branch(branch) => {
                <Coordinator<HIModel<W>> as AbstractSimulator>::lambda(branch, output, t)
            }
        }
    }

    fn delta(&mut self, input: &mut Self::Input, output: &mut Self::Output, t: f64) -> f64 {
        match self {
            HIEnum::Leaf(leaf) => {
                <Coordinator<LeafModel> as AbstractSimulator>::delta(leaf, input, output, t)
            }
            HIEnum::Branch(branch) => {
                <Coordinator<HIModel<W>> as AbstractSimulator>::delta(branch, input, output, t)
            }
        }
    }
}

/// HI coupled model
#[xdevs::coupled]
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

/// End model with Generator and HI model coupled together
#[xdevs::coupled]
pub struct TopModel<const W: usize> {
    generator: JobGenerator,
    hi_model: HIEnum<W>,
}

impl<const W: usize> Component for TopModel<W> {
    type Kind = xdevs::CoupledKind;
    type Input = xdevs::Port<usize, 1>;
    type Output = xdevs::Port<usize, 1>;
}
impl<const W: usize> TopModel<W> {
    pub fn get_n_internals(&self) -> usize {
        self.components.hi_model.get_n_internals()
    }

    pub fn get_n_externals(&self) -> usize {
        self.components.hi_model.get_n_externals()
    }

    pub fn get_n_events(&self) -> usize {
        self.components.hi_model.get_n_events()
    }

    pub fn get_n_atomics(&self) -> usize {
        self.components.hi_model.get_n_atomics()
    }
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
        use xdevs::simulation::Simulable;
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
