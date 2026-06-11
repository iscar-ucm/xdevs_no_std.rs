use crate::processor::Processor;

use super::common::{AtomicModel, JobGenerator};
use alloc::boxed::Box;
use xdevs::{traits::AbstractSimulator, Component};

/// Output struct for HO models
#[derive(Debug, Default, xdevs::Bag)]
pub struct HOModelOutput<const W: usize> {
    pub output_port_1: xdevs::port::Port<usize, 1>,
    pub output_port_2: xdevs::port::Port<usize, W>,
}
impl<const W: usize> HOModelOutput<W> {
    #[inline]
    pub const fn new() -> Self {
        Self {
            output_port_1: xdevs::port::Port::new(),
            output_port_2: xdevs::port::Port::new(),
        }
    }
}

/// Leaf coupled model with only one atomic in HO models
#[xdevs::coupled]
pub struct LeafModel<const W: usize> {
    atomic: AtomicModel,
}

impl<const W: usize> xdevs::Component for LeafModel<W> {
    type Kind = xdevs::CoupledKind;
    type Input = xdevs::port::Port<usize, 1>;
    type Output = HOModelOutput<W>;
}

impl<const W: usize> xdevs::Coupled for LeafModel<W> {
    fn eic(from: &Self::Input, to: &mut Self::ComponentsInput) {
        let _ = from.couple(&mut to.atomic);
    }
    fn eoc(from: &Self::ComponentsOutput, to: &mut Self::Output) {
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
    pub fn new_processor() -> Processor<Self> {
        Processor::new(Self::new())
    }
}

/// HO model enum
pub enum HOEnum<const W: usize> {
    Leaf(Processor<LeafModel<W>>),
    Branch(Processor<HOModel<W>>),
}

/// Manual implementation of `AbstractSimulator` for HO enum
unsafe impl<const W: usize> AbstractSimulator<xdevs::CoupledKind> for HOEnum<W> {
    fn start(processor: &mut Processor<Self>, t_start: f64) -> f64 {
        match &mut **processor {
            HOEnum::Leaf(leaf) => {
                <LeafModel<W> as AbstractSimulator<xdevs::CoupledKind>>::start(leaf, t_start)
            }
            HOEnum::Branch(branch) => {
                <HOModel<W> as AbstractSimulator<xdevs::CoupledKind>>::start(branch, t_start)
            }
        }
    }

    fn stop(processor: &mut Processor<Self>) {
        match &mut **processor {
            HOEnum::Leaf(leaf) => {
                <LeafModel<W> as AbstractSimulator<xdevs::CoupledKind>>::stop(leaf)
            }
            HOEnum::Branch(branch) => {
                <HOModel<W> as AbstractSimulator<xdevs::CoupledKind>>::stop(branch)
            }
        }
    }

    fn lambda(processor: &mut Processor<Self>, output: &mut Self::Output, t: f64) {
        match &mut **processor {
            HOEnum::Leaf(leaf) => {
                <LeafModel<W> as AbstractSimulator<xdevs::CoupledKind>>::lambda(leaf, output, t)
            }
            HOEnum::Branch(branch) => {
                <HOModel<W> as AbstractSimulator<xdevs::CoupledKind>>::lambda(branch, output, t)
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
            HOEnum::Leaf(leaf) => <LeafModel<W> as AbstractSimulator<xdevs::CoupledKind>>::delta(
                leaf, input, output, t,
            ),
            HOEnum::Branch(branch) => <HOModel<W> as AbstractSimulator<xdevs::CoupledKind>>::delta(
                branch, input, output, t,
            ),
        }
    }
}

/// Manual implementation of `Component` for HO enum
impl<const W: usize> Component for HOEnum<W> {
    type Kind = xdevs::CoupledKind;
    type Input = xdevs::port::Port<usize, 1>;
    type Output = HOModelOutput<W>;
}

impl<const W: usize> HOEnum<W> {
    pub fn get_n_internals(&self) -> usize {
        match self {
            HOEnum::Leaf(leaf) => leaf.get_n_internals(),
            HOEnum::Branch(branch) => branch.get_n_internals(),
        }
    }

    pub fn get_n_externals(&self) -> usize {
        match self {
            HOEnum::Leaf(leaf) => leaf.get_n_externals(),
            HOEnum::Branch(branch) => branch.get_n_externals(),
        }
    }

    pub fn get_n_events(&self) -> usize {
        match self {
            HOEnum::Leaf(leaf) => leaf.get_n_events(),
            HOEnum::Branch(branch) => branch.get_n_events(),
        }
    }

    pub fn get_n_atomics(&self) -> usize {
        match self {
            HOEnum::Leaf(leaf) => leaf.get_n_atomics(),
            HOEnum::Branch(branch) => branch.get_n_atomics(),
        }
    }
}

/// Manual implementation of `AbstractSimulator` for the Boxed HO enum
unsafe impl<const W: usize> AbstractSimulator<xdevs::CoupledKind> for Box<HOEnum<W>> {
    #[inline]
    fn start(processor: &mut Processor<Self>, t_start: f64) -> f64 {
        match &mut ***processor {
            HOEnum::Leaf(leaf) => {
                <LeafModel<W> as AbstractSimulator<xdevs::CoupledKind>>::start(leaf, t_start)
            }
            HOEnum::Branch(branch) => {
                <HOModel<W> as AbstractSimulator<xdevs::CoupledKind>>::start(branch, t_start)
            }
        }
    }

    #[inline]
    fn stop(processor: &mut Processor<Self>) {
        match &mut ***processor {
            HOEnum::Leaf(leaf) => {
                <LeafModel<W> as AbstractSimulator<xdevs::CoupledKind>>::stop(leaf)
            }
            HOEnum::Branch(branch) => {
                <HOModel<W> as AbstractSimulator<xdevs::CoupledKind>>::stop(branch)
            }
        }
    }

    #[inline]
    fn lambda(processor: &mut Processor<Self>, output: &mut Self::Output, t: f64) {
        match &mut ***processor {
            HOEnum::Leaf(leaf) => {
                <LeafModel<W> as AbstractSimulator<xdevs::CoupledKind>>::lambda(leaf, output, t)
            }
            HOEnum::Branch(branch) => {
                <HOModel<W> as AbstractSimulator<xdevs::CoupledKind>>::lambda(branch, output, t)
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
            HOEnum::Leaf(leaf) => <LeafModel<W> as AbstractSimulator<xdevs::CoupledKind>>::delta(
                leaf, input, output, t,
            ),
            HOEnum::Branch(branch) => <HOModel<W> as AbstractSimulator<xdevs::CoupledKind>>::delta(
                branch, input, output, t,
            ),
        }
    }
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
    pub fn new_processor(inner: Box<HOEnum<W>>) -> Processor<Self> {
        Processor::new(Self::new(inner))
    }
}
impl<const W: usize> xdevs::Component for HOModel<W> {
    type Kind = xdevs::CoupledKind;
    type Input = xdevs::port::Port<usize, 1>;
    type Output = HOModelOutput<W>;
}

impl<const W: usize> xdevs::Coupled for HOModel<W> {
    fn eic(from: &Self::Input, to: &mut Self::ComponentsInput) {
        let _ = from.couple(&mut to.inner);
        for atom_ports in to.atomics.iter_mut() {
            let _ = from.couple(atom_ports);
        }
    }

    fn eoc(from: &Self::ComponentsOutput, to: &mut Self::Output) {
        let _ = from.inner.output_port_1.couple(&mut to.output_port_1);
        for atom_output_ports in from.atomics.iter() {
            let _ = atom_output_ports.couple(&mut to.output_port_2);
        }
    }

    fn ic(from: &Self::ComponentsOutput, to: &mut Self::ComponentsInput) {
        if W > 1 {
            for i in 0..(W - 1) {
                let _ = from.atomics[i].couple(&mut to.atomics[i + 1]);
            }
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
    type Input = xdevs::port::Port<usize, 1>;
    type Output = xdevs::port::Port<usize, 1>;
}

impl<const W: usize> TopModel<W> {
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

impl<const W: usize> xdevs::Coupled for TopModel<W> {
    fn ic(from: &Self::ComponentsOutput, to: &mut Self::ComponentsInput) {
        let _ = from.generator.couple(&mut to.ho_model);
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn expected_n_atomic(width: usize, depth: usize) -> usize {
        (width - 1) * (depth - 1) + 1
    }

    //CAMBIAR ESTA ECUACIÓN
    fn expected_n_events(width: usize, depth: usize) -> usize {
        1 + (depth - 1) * ((width - 1) * width) / 2
    }

    #[test]
    fn test_ho() {
        const WIDTH: usize = 100;
        const DEPTH: usize = 100;
        const W: usize = WIDTH - 1;

        xdevs::generate_ho!(100, 100);

        let generator = JobGenerator::new(5);
        let top_model: TopModel<W> = TopModel::build(generator, model_ho);
        let mut simulator = xdevs::simulator::Simulator::new(top_model);
        let config = xdevs::simulator::Config::new(0.0, 10.0, 1.0, None);
        simulator.simulate_vt(&config);

        assert_eq!(expected_n_atomic(WIDTH, DEPTH), simulator.get_n_atomics());
        assert_eq!(expected_n_events(WIDTH, DEPTH), simulator.get_n_events());
        assert_eq!(simulator.get_n_internals(), simulator.get_n_externals());
    }
}
