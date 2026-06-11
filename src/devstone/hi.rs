use crate::processor::Processor;

use super::common::*;
use xdevs::traits::{AbstractSimulator, Component};

use alloc::boxed::Box;

/// HI model enum
pub enum HIEnum<const W: usize> {
    Leaf(Processor<LeafModel>),
    Branch(Processor<HIModel<W>>),
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
    type Kind = xdevs::CoupledKind;
    type Input = xdevs::Port<usize, 1>;
    type Output = xdevs::Port<usize, 1>;
}

/// Manual implementation of `AbstractSimulator` for HI enum
unsafe impl<const W: usize> AbstractSimulator<xdevs::CoupledKind> for HIEnum<W> {
    fn start(processor: &mut Processor<Self>, t_start: f64) -> f64 {
        match &mut **processor {
            HIEnum::Leaf(leaf) => {
                <LeafModel as AbstractSimulator<xdevs::CoupledKind>>::start(leaf, t_start)
            }
            HIEnum::Branch(branch) => {
                <HIModel<W> as AbstractSimulator<xdevs::CoupledKind>>::start(branch, t_start)
            }
        }
    }

    fn stop(processor: &mut Processor<Self>) {
        match &mut **processor {
            HIEnum::Leaf(leaf) => <LeafModel as AbstractSimulator<xdevs::CoupledKind>>::stop(leaf),
            HIEnum::Branch(branch) => {
                <HIModel<W> as AbstractSimulator<xdevs::CoupledKind>>::stop(branch)
            }
        }
    }

    fn lambda(processor: &mut Processor<Self>, output: &mut Self::Output, t: f64) {
        match &mut **processor {
            HIEnum::Leaf(leaf) => {
                <LeafModel as AbstractSimulator<xdevs::CoupledKind>>::lambda(leaf, output, t)
            }
            HIEnum::Branch(branch) => {
                <HIModel<W> as AbstractSimulator<xdevs::CoupledKind>>::lambda(branch, output, t)
            }
        }
    }

    fn delta(
        processor: &mut Processor<Self>,
        input: &mut Self::Input,
        output: &mut Self::Output,
        t: f64,
    ) -> f64 {
        match &mut **processor {
            HIEnum::Leaf(leaf) => {
                <LeafModel as AbstractSimulator<xdevs::CoupledKind>>::delta(leaf, input, output, t)
            }
            HIEnum::Branch(branch) => <HIModel<W> as AbstractSimulator<xdevs::CoupledKind>>::delta(
                branch, input, output, t,
            ),
        }
    }
}

/// Manual implementation of `AbstractSimulator` for the Boxed HI enum
unsafe impl<const W: usize> AbstractSimulator<xdevs::CoupledKind> for Box<HIEnum<W>> {
    #[inline]
    fn start(processor: &mut Processor<Self>, t_start: f64) -> f64 {
        match &mut ***processor {
            HIEnum::Leaf(leaf) => {
                <LeafModel as AbstractSimulator<xdevs::CoupledKind>>::start(leaf, t_start)
            }
            HIEnum::Branch(branch) => {
                <HIModel<W> as AbstractSimulator<xdevs::CoupledKind>>::start(branch, t_start)
            }
        }
    }

    #[inline]
    fn stop(processor: &mut Processor<Self>) {
        match &mut ***processor {
            HIEnum::Leaf(leaf) => <LeafModel as AbstractSimulator<xdevs::CoupledKind>>::stop(leaf),
            HIEnum::Branch(branch) => {
                <HIModel<W> as AbstractSimulator<xdevs::CoupledKind>>::stop(branch)
            }
        }
    }

    #[inline]
    fn lambda(processor: &mut Processor<Self>, output: &mut Self::Output, t: f64) {
        match &mut ***processor {
            HIEnum::Leaf(leaf) => {
                <LeafModel as AbstractSimulator<xdevs::CoupledKind>>::lambda(leaf, output, t)
            }
            HIEnum::Branch(branch) => {
                <HIModel<W> as AbstractSimulator<xdevs::CoupledKind>>::lambda(branch, output, t)
            }
        }
    }

    #[inline]
    fn delta(
        processor: &mut Processor<Self>,
        input: &mut Self::Input,
        output: &mut Self::Output,
        t: f64,
    ) -> f64 {
        match &mut ***processor {
            HIEnum::Leaf(leaf) => {
                <LeafModel as AbstractSimulator<xdevs::CoupledKind>>::delta(leaf, input, output, t)
            }
            HIEnum::Branch(branch) => <HIModel<W> as AbstractSimulator<xdevs::CoupledKind>>::delta(
                branch, input, output, t,
            ),
        }
    }
}

/// HI coupled model
#[xdevs::coupled]
pub struct HIModel<const W: usize> {
    atomics: [AtomicModel; W],
    inner: Box<HIEnum<W>>,
}

impl<const W: usize> xdevs::traits::Component for HIModel<W> {
    type Kind = xdevs::CoupledKind;
    type Input = xdevs::Port<usize, 1>;
    type Output = xdevs::Port<usize, 1>;
}

impl<const W: usize> xdevs::Coupled for HIModel<W> {
    fn eic(from: &Self::Input, to: &mut Self::ComponentsInput) {
        for atom_ports in to.atomics.iter_mut() {
            let _ = from.couple(atom_ports);
        }

        let _ = from.couple(&mut to.inner);
    }

    fn eoc(from: &Self::ComponentsOutput, to: &mut Self::Output) {
        let _ = from.inner.couple(to);
    }

    fn ic(from: &Self::ComponentsOutput, to: &mut Self::ComponentsInput) {
        if W > 1 {
            for i in 0..(W - 1) {
                let _ = from.atomics[i].couple(&mut to.atomics[i + 1]);
            }
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

    pub fn new_processor(inner: Box<HIEnum<W>>) -> Processor<Self> {
        Processor::new(Self::new(inner))
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
    fn ic(from: &Self::ComponentsOutput, to: &mut Self::ComponentsInput) {
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
    fn test_hi() {
        const WIDTH: usize = 100;
        const DEPTH: usize = 100;
        const W: usize = WIDTH - 1;

        xdevs::generate_hi!(100, 100);

        let generator = JobGenerator::new(5);

        //Creación del modelo final (modelo HI + atómico generador que mete datos en el puerto del HI)
        let top_model: TopModel<W> = TopModel::build(generator, model_hi);
        let mut simulator = xdevs::simulator::Simulator::new(top_model);
        let config = xdevs::simulator::Config::new(0.0, 10.0, 1.0, None);
        simulator.simulate_vt(&config);

        assert_eq!(expected_n_atomic(WIDTH, DEPTH), simulator.get_n_atomics());
        assert_eq!(expected_n_events(WIDTH, DEPTH), simulator.get_n_events());
        assert_eq!(simulator.get_n_internals(), simulator.get_n_externals());
    }
}
