use crate::{simulation::coordinator::Coordinator, AbstractSimulator};

use super::common::*;
use alloc::boxed::Box;
use xdevs::Component;

pub enum LIEnum<const W: usize> {
    Leaf(Coordinator<LeafModel>),
    Branch(Coordinator<LIModel<W>>),
}

impl<const W: usize> LIEnum<W> {
    pub fn get_n_internals(&self) -> usize {
        match self {
            LIEnum::Leaf(leaf) => leaf.get_n_internals(),
            LIEnum::Branch(branch) => branch.get_n_internals(),
        }
    }

    pub fn get_n_externals(&self) -> usize {
        match self {
            LIEnum::Leaf(leaf) => leaf.get_n_externals(),
            LIEnum::Branch(branch) => branch.get_n_externals(),
        }
    }

    pub fn get_n_events(&self) -> usize {
        match self {
            LIEnum::Leaf(leaf) => leaf.get_n_events(),
            LIEnum::Branch(branch) => branch.get_n_events(),
        }
    }

    pub fn get_n_atomics(&self) -> usize {
        match self {
            LIEnum::Leaf(leaf) => leaf.get_n_atomics(),
            LIEnum::Branch(branch) => branch.get_n_atomics(),
        }
    }
}

/// Manual implementation of `Component` for LI enum
impl<const W: usize> Component for LIEnum<W> {
    type Kind = xdevs::ComponentsKind;
    type Input = xdevs::Port<usize, 1>;
    type Output = xdevs::Port<usize, 1>;
}

/// Manual implementation of `AbstractSimulator` for LI enum
unsafe impl<const W: usize> AbstractSimulator for LIEnum<W> {
    type Input = <Coordinator<LIModel<W>> as AbstractSimulator>::Input;
    type Output = <Coordinator<LIModel<W>> as AbstractSimulator>::Output;

    fn start(&mut self, t_start: f64) -> f64 {
        match self {
            LIEnum::Leaf(leaf) => {
                <Coordinator<LeafModel> as AbstractSimulator>::start(leaf, t_start)
            }
            LIEnum::Branch(branch) => {
                <Coordinator<LIModel<W>> as AbstractSimulator>::start(branch, t_start)
            }
        }
    }

    fn stop(&mut self) {
        match self {
            LIEnum::Leaf(leaf) => <Coordinator<LeafModel> as AbstractSimulator>::stop(leaf),
            LIEnum::Branch(branch) => <Coordinator<LIModel<W>> as AbstractSimulator>::stop(branch),
        }
    }

    fn lambda(&mut self, output: &mut Self::Output, t: f64) {
        match self {
            LIEnum::Leaf(leaf) => {
                <Coordinator<LeafModel> as AbstractSimulator>::lambda(leaf, output, t)
            }
            LIEnum::Branch(branch) => {
                <Coordinator<LIModel<W>> as AbstractSimulator>::lambda(branch, output, t)
            }
        }
    }

    fn delta(&mut self, input: &mut Self::Input, output: &mut Self::Output, t: f64) -> f64 {
        match self {
            LIEnum::Leaf(leaf) => {
                <Coordinator<LeafModel> as AbstractSimulator>::delta(leaf, input, output, t)
            }
            LIEnum::Branch(branch) => {
                <Coordinator<LIModel<W>> as AbstractSimulator>::delta(branch, input, output, t)
            }
        }
    }
}

/// LI coupled model
#[xdevs::coupled]
pub struct LIModel<const W: usize> {
    atomics: [AtomicModel; W],
    inner: Box<LIEnum<W>>,
}

impl<const W: usize> Component for LIModel<W> {
    type Kind = xdevs::CoupledKind;
    type Input = xdevs::Port<usize, 1>;
    type Output = xdevs::Port<usize, 1>;
}

impl<const W: usize> xdevs::Coupled for LIModel<W> {
    fn eic(from: &Self::Input, to: &mut <Self::Components as xdevs::Component>::Input) {
        for atom_ports in to.atomics.iter_mut() {
            let _ = from.couple(atom_ports);
        }

        let _ = from.couple(&mut to.inner);
    }

    fn eoc(from: &<Self::Components as xdevs::Component>::Output, to: &mut Self::Output) {
        let _ = from.inner.couple(to);
    }
}

impl<const W: usize> LIModel<W> {
    pub fn new(inner: Box<LIEnum<W>>) -> Self {
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

/// End model with Generator and LI model coupled together
#[xdevs::coupled]
pub struct TopModel<const W: usize> {
    generator: JobGenerator,
    li_model: LIEnum<W>,
}

impl<const W: usize> Component for TopModel<W> {
    type Kind = xdevs::CoupledKind;
    type Input = ();
    type Output = ();
}

impl<const W: usize> TopModel<W> {
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

impl<const W: usize> xdevs::Coupled for TopModel<W> {
    fn ic(
        from: &<Self::Components as xdevs::Component>::Output,
        to: &mut <Self::Components as xdevs::Component>::Input,
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
    fn test_li() {
        use xdevs::simulation::Simulable;
        const WIDTH: usize = 10;
        const DEPTH: usize = 10;
        const W: usize = WIDTH - 1;

        xdevs::generate_li!(10, 10);
        let generator = JobGenerator::new(5);
        let top_model: TopModel<W> = TopModel::build(generator, model_li);
        let mut simulator = top_model.to_simulator();
        let config = xdevs::simulation::Config::new(0.0, 10.0, 1.0, None);
        simulator.simulate_vt(&config);

        assert_eq!(expected_n_atomic(WIDTH, DEPTH), simulator.get_n_atomics());
        assert_eq!(expected_n_events(WIDTH, DEPTH), simulator.get_n_events());
        assert_eq!(simulator.get_n_internals(), simulator.get_n_externals());
    }
}
