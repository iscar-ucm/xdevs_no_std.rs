use super::common::{AtomicModel, JobGenerator};
use alloc::boxed::Box;
use xdevs::traits::{AbstractSimulator, Component, SimTime};

/// Manual implementation of the leaf coupled model with only one atomic in HO models
pub struct LeafModelComponents {
    atomic: AtomicModel,
}
impl LeafModelComponents {
    #[inline]
    pub fn new(atomic: AtomicModel) -> Self {
        Self { atomic }
    }
}
#[doc = r" Wrapper struct holding all inner components' inputs."]
#[derive(xdevs::Bag)]
pub struct LeafModelComponentsInput {
    pub atomic: <AtomicModel as xdevs::traits::Component>::Input,
}
#[doc = r" Wrapper struct holding all inner components' outputs."]
#[derive(xdevs::Bag)]
pub struct LeafModelComponentsOutput {
    pub atomic: <AtomicModel as xdevs::traits::Component>::Output,
}
pub struct LeafModel<const W: usize> {
    pub t_last: f64,
    pub t_next: f64,
    pub components: LeafModelComponents,
    pub components_input: LeafModelComponentsInput,
    pub components_output: LeafModelComponentsOutput,
}
impl<const W: usize> LeafModel<W> {
    #[inline]
    pub fn build(atomic: AtomicModel) -> Self {
        Self {
            t_last: 0.0,
            t_next: f64::INFINITY,
            components: LeafModelComponents::new(atomic),
            components_input: <LeafModelComponentsInput as xdevs::traits::Bag>::build(),
            components_output: <LeafModelComponentsOutput as xdevs::traits::Bag>::build(),
        }
    }
}
impl<const W: usize> xdevs::traits::Component for LeafModel<W> {
    type Input = HOModelInput;
    type Output = HOModelOutput<W>;
}
unsafe impl<const W: usize> xdevs::traits::SimTime for LeafModel<W> {
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
unsafe impl<const W: usize> xdevs::traits::PartialCoupled for LeafModel<W> {
    type ComponentsInput = LeafModelComponentsInput;
    type ComponentsOutput = LeafModelComponentsOutput;
}
unsafe impl<const W: usize> xdevs::traits::AbstractSimulator for LeafModel<W> {
    #[inline]
    fn start(&mut self, t_start: f64) -> f64 {
        xdevs::traits::SimTime::set_t_last(self, t_start);
        let mut t_next = f64::INFINITY;
        t_next = f64::min(
            t_next,
            xdevs::traits::AbstractSimulator::start(&mut self.components.atomic, t_start),
        );
        xdevs::traits::SimTime::set_t_next(self, t_next);
        t_next
    }
    #[inline]
    fn stop(&mut self, t_stop: f64) {
        xdevs::traits::AbstractSimulator::stop(&mut self.components.atomic, t_stop);
        xdevs::traits::SimTime::set_t_last(self, t_stop);
        xdevs::traits::SimTime::set_t_next(self, f64::INFINITY);
    }
    #[inline]
    fn lambda(&mut self, output: &mut Self::Output, t: f64) {
        if t >= xdevs::traits::SimTime::get_t_next(self) {
            xdevs::traits::AbstractSimulator::lambda(
                &mut self.components.atomic,
                &mut self.components_output.atomic,
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

        // get minimum t_next from all components after executing their delta
        let mut t_next = f64::INFINITY;
        t_next = f64::min(
            t_next,
            xdevs::traits::AbstractSimulator::delta(
                &mut self.components.atomic,
                &mut self.components_input.atomic,
                &mut self.components_output.atomic,
                t,
            ),
        );

        // set t_last to t and t_next to minimum t_next
        xdevs::traits::SimTime::set_t_last(self, t);
        xdevs::traits::SimTime::set_t_next(self, t_next);

        // clear input and output events
        <Self::Input as ::xdevs::traits::Bag>::clear(input);
        <Self::Output as ::xdevs::traits::Bag>::clear(output);

        t_next
    }
}

impl<const W: usize> Default for LeafModel<W> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const W: usize> xdevs::Coupled for LeafModel<W> {
    fn eic(from: &Self::Input, to: &mut Self::ComponentsInput) {
        let _ = from.input_port.couple(&mut to.atomic.input_port);
    }
    fn eoc(from: &Self::ComponentsOutput, to: &mut Self::Output) {
        let _ = from.atomic.output_port.couple(&mut to.output_port_1);
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
}

/// HO model enum
pub enum HOEnum<const W: usize> {
    Leaf(LeafModel<W>),
    Branch(HOModel<W>),
}

/// Manual implementation of `AbstractSimulator` for HO enum
unsafe impl<const W: usize> AbstractSimulator for HOEnum<W> {
    fn start(&mut self, t_start: f64) -> f64 {
        match self {
            HOEnum::Leaf(leaf) => leaf.start(t_start),
            HOEnum::Branch(branch) => branch.start(t_start),
        }
    }

    fn stop(&mut self, t_stop: f64) {
        match self {
            HOEnum::Leaf(leaf) => leaf.stop(t_stop),
            HOEnum::Branch(branch) => branch.stop(t_stop),
        }
    }

    fn lambda(&mut self, output: &mut Self::Output, t: f64) {
        match self {
            HOEnum::Leaf(leaf) => leaf.lambda(output, t),
            HOEnum::Branch(branch) => branch.lambda(output, t),
        }
    }

    fn delta(&mut self, input: &mut Self::Input, output: &mut Self::Output, t: f64) -> f64 {
        match self {
            HOEnum::Leaf(leaf) => leaf.delta(input, output, t),
            HOEnum::Branch(branch) => branch.delta(input, output, t),
        }
    }
}

/// Manual implementation of `Component` for HO enum
impl<const W: usize> Component for HOEnum<W> {
    type Input = HOModelInput;

    type Output = HOModelOutput<W>;
}

/// Manual implementation of `SimTime` for HO enum
unsafe impl<const W: usize> SimTime for HOEnum<W> {
    fn get_t_last(&self) -> f64 {
        match self {
            HOEnum::Leaf(leaf) => leaf.get_t_last(),
            HOEnum::Branch(branch) => branch.get_t_last(),
        }
    }

    fn set_t_last(&mut self, t_last: f64) {
        match self {
            HOEnum::Leaf(leaf) => leaf.set_t_last(t_last),
            HOEnum::Branch(branch) => branch.set_t_last(t_last),
        }
    }

    fn get_t_next(&self) -> f64 {
        match self {
            HOEnum::Leaf(leaf) => leaf.get_t_next(),
            HOEnum::Branch(branch) => branch.get_t_next(),
        }
    }

    fn set_t_next(&mut self, t_next: f64) {
        match self {
            HOEnum::Leaf(leaf) => leaf.set_t_next(t_next),
            HOEnum::Branch(branch) => branch.set_t_next(t_next),
        }
    }
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

/// Manual implementation of the HO coupled model
#[derive(Debug, Default, xdevs::Bag)]
pub struct HOModelInput {
    pub input_port: xdevs::port::Port<usize, 1>,
}
impl HOModelInput {
    #[inline]
    pub const fn new() -> Self {
        Self {
            input_port: xdevs::port::Port::new(),
        }
    }
}

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

pub struct HOModelComponents<const W: usize> {
    atomics: [AtomicModel; W],
    inner: Box<HOEnum<W>>,
}
impl<const W: usize> HOModelComponents<W> {
    #[inline]
    pub fn new(atomics: [AtomicModel; W], inner: Box<HOEnum<W>>) -> Self {
        Self { atomics, inner }
    }
}
#[doc = r" Wrapper struct holding all inner components' inputs."]
#[derive(xdevs::Bag)]
pub struct HOModelComponentsInput<const W: usize> {
    pub atomics: <[AtomicModel; W] as xdevs::traits::Component>::Input,
    pub inner: <Box<HOEnum<W>> as xdevs::traits::Component>::Input,
}
#[derive(xdevs::Bag)]
#[doc = r" Wrapper struct holding all inner components' outputs."]
pub struct HOModelComponentsOutput<const W: usize> {
    pub atomics: <[AtomicModel; W] as xdevs::traits::Component>::Output,
    pub inner: <Box<HOEnum<W>> as xdevs::traits::Component>::Output,
}
pub struct HOModel<const W: usize> {
    pub t_last: f64,
    pub t_next: f64,
    pub components: HOModelComponents<W>,
    pub components_input: HOModelComponentsInput<W>,
    pub components_output: HOModelComponentsOutput<W>,
}
impl<const W: usize> HOModel<W> {
    #[inline]
    pub fn build(atomics: [AtomicModel; W], inner: Box<HOEnum<W>>) -> Self {
        Self {
            t_last: 0.0,
            t_next: f64::INFINITY,
            components: HOModelComponents::new(atomics, inner),
            components_input: <HOModelComponentsInput<W> as xdevs::traits::Bag>::build(),
            components_output: <HOModelComponentsOutput<W> as xdevs::traits::Bag>::build(),
        }
    }

    pub fn new(inner: Box<HOEnum<W>>) -> Self {
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
impl<const W: usize> xdevs::traits::Component for HOModel<W> {
    type Input = HOModelInput;
    type Output = HOModelOutput<W>;
}
unsafe impl<const W: usize> xdevs::traits::SimTime for HOModel<W> {
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
unsafe impl<const W: usize> xdevs::traits::PartialCoupled for HOModel<W> {
    type ComponentsInput = HOModelComponentsInput<W>;
    type ComponentsOutput = HOModelComponentsOutput<W>;
}
unsafe impl<const W: usize> xdevs::traits::AbstractSimulator for HOModel<W> {
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
            <Self as ::xdevs::Coupled>::eoc(&self.components_output, output);
        }
    }
    #[inline]
    fn delta(&mut self, input: &mut Self::Input, output: &mut Self::Output, t: f64) -> f64 {
        // propagate EICs and ICs via Coupled trait
        <Self as xdevs::Coupled>::eic(input, &mut self.components_input);
        <Self as xdevs::Coupled>::ic(&self.components_output, &mut self.components_input);

        // get minimum t_next from all components after executing their delta
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
        <Self::Input as ::xdevs::traits::Bag>::clear(input);
        <Self::Output as ::xdevs::traits::Bag>::clear(output);
        t_next
    }
}

impl<const W: usize> xdevs::Coupled for HOModel<W> {
    fn eic(from: &Self::Input, to: &mut Self::ComponentsInput) {
        let _ = from.input_port.couple(&mut to.inner.input_port);
        for atom_ports in to.atomics.iter_mut() {
            let _ = from.input_port.couple(&mut atom_ports.input_port);
        }
    }

    fn eoc(from: &Self::ComponentsOutput, to: &mut Self::Output) {
        let _ = from.inner.output_port_1.couple(&mut to.output_port_1);
        for atom_output_ports in from.atomics.iter() {
            let _ = atom_output_ports.output_port.couple(&mut to.output_port_2);
        }
    }

    fn ic(from: &Self::ComponentsOutput, to: &mut Self::ComponentsInput) {
        if W > 1 {
            for i in 0..(W - 1) {
                let _ = from.atomics[i]
                    .output_port
                    .couple(&mut to.atomics[i + 1].input_port);
            }
        }
    }
}

/// End model with Generator and HO model coupled together
#[xdevs::coupled]
pub struct TopModel<const W: usize> {
    #[input]
    input_port: xdevs::port::Port<usize, 1>,
    #[output]
    output_port: xdevs::port::Port<usize, 1>,
    #[components]
    generator: JobGenerator,
    ho_model: HOEnum<W>,
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
        let _ = from.generator.out_job.couple(&mut to.ho_model.input_port);
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
        let top_model = simulator.get_model();

        assert_eq!(expected_n_atomic(WIDTH, DEPTH), top_model.get_n_atomics());
        assert_eq!(expected_n_events(WIDTH, DEPTH), top_model.get_n_events());
        assert_eq!(top_model.get_n_internals(), top_model.get_n_externals());
    }
}
