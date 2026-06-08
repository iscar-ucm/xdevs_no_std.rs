use super::common::*;
use alloc::boxed::Box;
use xdevs::traits::{AbstractSimulator, Component, SimTime};

pub enum LIEnum<const W: usize> {
    Leaf(LeafModel),
    Branch(LIModel<W>),
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

/// Manual implementation of `AbstractSimulator` for LI enum
unsafe impl<const W: usize> AbstractSimulator for LIEnum<W> {
    fn start(&mut self, t_start: f64) -> f64 {
        match self {
            LIEnum::Leaf(leaf) => leaf.start(t_start),
            LIEnum::Branch(branch) => branch.start(t_start),
        }
    }

    fn stop(&mut self, t_stop: f64) {
        match self {
            LIEnum::Leaf(leaf) => leaf.stop(t_stop),
            LIEnum::Branch(branch) => branch.stop(t_stop),
        }
    }

    fn lambda(&mut self, output: &mut Self::Output, t: f64) {
        match self {
            LIEnum::Leaf(leaf) => leaf.lambda(output, t),
            LIEnum::Branch(branch) => branch.lambda(output, t),
        }
    }

    fn delta(&mut self, input: &mut Self::Input, output: &mut Self::Output, t: f64) -> f64 {
        match self {
            LIEnum::Leaf(leaf) => leaf.delta(input, output, t),
            LIEnum::Branch(branch) => branch.delta(input, output, t),
        }
    }
}

/// Manual implementation of `Component` for LI enum
impl<const W: usize> Component for LIEnum<W> {
    type Input = xdevs::Port<usize, 1>;
    type Output = xdevs::Port<usize, 1>;
}

/// Manual implementation of `SimTime` for LI enum
unsafe impl<const W: usize> SimTime for LIEnum<W> {
    fn get_t_last(&self) -> f64 {
        match self {
            LIEnum::Leaf(leaf) => leaf.get_t_last(),
            LIEnum::Branch(branch) => branch.get_t_last(),
        }
    }

    fn set_t_last(&mut self, _t_last: f64) {
        match self {
            LIEnum::Leaf(leaf) => leaf.set_t_last(_t_last),
            LIEnum::Branch(branch) => branch.set_t_last(_t_last),
        }
    }

    fn get_t_next(&self) -> f64 {
        match self {
            LIEnum::Leaf(leaf) => leaf.get_t_next(),
            LIEnum::Branch(branch) => branch.get_t_next(),
        }
    }

    fn set_t_next(&mut self, _t_next: f64) {
        match self {
            LIEnum::Leaf(leaf) => leaf.set_t_next(_t_next),
            LIEnum::Branch(branch) => branch.set_t_next(_t_next),
        }
    }
}

/// Manual implementation of `PartialCoupled` for LI coupled model
pub struct LIModelComponents<const W: usize> {
    atomics: [AtomicModel; W],
    inner: Box<LIEnum<W>>,
}
impl<const W: usize> LIModelComponents<W> {
    #[inline]
    pub fn new(atomics: [AtomicModel; W], inner: Box<LIEnum<W>>) -> Self {
        Self { atomics, inner }
    }
}
#[doc = r" Wrapper struct holding all inner components' inputs."]
#[derive(xdevs::Bag)]
pub struct LIModelComponentsInput<const W: usize> {
    pub atomics: <[AtomicModel; W] as xdevs::traits::Component>::Input,
    pub inner: <Box<LIEnum<W>> as xdevs::traits::Component>::Input,
}
#[doc = r" Wrapper struct holding all inner components' outputs."]
#[derive(xdevs::Bag)]
pub struct LIModelComponentsOutput<const W: usize> {
    pub atomics: <[AtomicModel; W] as xdevs::traits::Component>::Output,
    pub inner: <Box<LIEnum<W>> as xdevs::traits::Component>::Output,
}
pub struct LIModel<const W: usize> {
    pub t_last: f64,
    pub t_next: f64,
    pub components: LIModelComponents<W>,
    pub components_input: LIModelComponentsInput<W>,
    pub components_output: LIModelComponentsOutput<W>,
}
impl<const W: usize> LIModel<W> {
    #[inline]
    pub fn build(atomics: [AtomicModel; W], inner: Box<LIEnum<W>>) -> Self {
        Self {
            t_last: 0.0,
            t_next: f64::INFINITY,
            components: LIModelComponents::new(atomics, inner),
            components_input: <LIModelComponentsInput<W> as xdevs::traits::Bag>::build(),
            components_output: <LIModelComponentsOutput<W> as xdevs::traits::Bag>::build(),
        }
    }
}

impl<const W: usize> xdevs::traits::Component for LIModel<W> {
    type Input = xdevs::Port<usize, 1>;
    type Output = xdevs::Port<usize, 1>;
}

unsafe impl<const W: usize> xdevs::traits::SimTime for LIModel<W> {
    #[inline]
    fn get_t_last(&self) -> f64 {
        self.t_last
    }
    #[inline]
    fn set_t_last(&mut self, t_last: f64) {
        self.t_last = t_last;
    }
    #[inline]
    fn get_t_next(&self) -> f64 {
        self.t_next
    }
    #[inline]
    fn set_t_next(&mut self, t_next: f64) {
        self.t_next = t_next;
    }
}
unsafe impl<const W: usize> xdevs::traits::PartialCoupled for LIModel<W> {
    type ComponentsInput = LIModelComponentsInput<W>;
    type ComponentsOutput = LIModelComponentsOutput<W>;
}
unsafe impl<const W: usize> xdevs::traits::AbstractSimulator for LIModel<W> {
    #[inline]
    fn start(&mut self, t_start: f64) -> f64 {
        xdevs::traits::SimTime::set_t_last(self, t_start);
        let mut t_next = f64::INFINITY;
        t_next = f64::min(
            t_next,
            xdevs::traits::AbstractSimulator::start(&mut self.components.atomics, t_start),
        );
        t_next = f64::min(
            t_next,
            xdevs::traits::AbstractSimulator::start(&mut self.components.inner, t_start),
        );
        xdevs::traits::SimTime::set_t_next(self, t_next);
        t_next
    }
    #[inline]
    fn stop(&mut self, t_stop: f64) {
        xdevs::traits::AbstractSimulator::stop(&mut self.components.atomics, t_stop);
        xdevs::traits::AbstractSimulator::stop(&mut self.components.inner, t_stop);
        xdevs::traits::SimTime::set_t_last(self, t_stop);
        xdevs::traits::SimTime::set_t_next(self, f64::INFINITY);
    }
    #[inline]
    fn lambda(&mut self, output: &mut Self::Output, t: f64) {
        if t >= xdevs::traits::SimTime::get_t_next(self) {
            xdevs::traits::AbstractSimulator::lambda(
                &mut self.components.atomics,
                &mut self.components_output.atomics,
                t,
            );
            xdevs::traits::AbstractSimulator::lambda(
                &mut self.components.inner,
                &mut self.components_output.inner,
                t,
            );
            <Self as xdevs::Coupled>::eoc(&self.components_output, output);
        }
    }
    #[inline]
    fn delta(&mut self, input: &mut Self::Input, output: &mut Self::Output, t: f64) -> f64 {
        // propagate EICs and ICs via Coupled trait
        <Self as xdevs::Coupled>::eic(input, &mut self.components_input);
        <Self as xdevs::Coupled>::ic(&self.components_output, &mut self.components_input);

        let mut t_next = f64::INFINITY;
        t_next = f64::min(
            t_next,
            xdevs::traits::AbstractSimulator::delta(
                &mut self.components.atomics,
                &mut self.components_input.atomics,
                &mut self.components_output.atomics,
                t,
            ),
        );
        t_next = f64::min(
            t_next,
            xdevs::traits::AbstractSimulator::delta(
                &mut self.components.inner,
                &mut self.components_input.inner,
                &mut self.components_output.inner,
                t,
            ),
        );

        // set t_last to t and t_next to minimum t_next
        xdevs::traits::SimTime::set_t_last(self, t);
        xdevs::traits::SimTime::set_t_next(self, t_next);

        // clear input and output events
        <Self::Input as xdevs::traits::Bag>::clear(input);
        <Self::Output as xdevs::traits::Bag>::clear(output);

        t_next
    }
}

impl<const W: usize> xdevs::Coupled for LIModel<W> {
    fn eic(from: &Self::Input, to: &mut Self::ComponentsInput) {
        for atom_ports in to.atomics.iter_mut() {
            let _ = from.couple(&mut atom_ports.input_port);
        }

        let _ = from.couple(&mut to.inner);
    }

    fn eoc(from: &Self::ComponentsOutput, to: &mut Self::Output) {
        let _ = from.inner.couple(to);
    }
}

impl<const W: usize> LIModel<W> {
    pub fn new(inner: Box<LIEnum<W>>) -> Self {
        Self::build(core::array::from_fn(|_| AtomicModel::new()), inner)
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
    #[components]
    generator: JobGenerator,
    li_model: LIEnum<W>,
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
    fn ic(from: &Self::ComponentsOutput, to: &mut Self::ComponentsInput) {
        let _ = from.generator.out_job.couple(&mut to.li_model);
    }
}
