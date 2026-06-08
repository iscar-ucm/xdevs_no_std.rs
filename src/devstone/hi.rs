use super::common::*;
use xdevs::traits::{AbstractSimulator, Component, SimTime};

use alloc::boxed::Box;

/// HI model enum
pub enum HIEnum<const W: usize> {
    Leaf(LeafModel),
    Branch(HIModel<W>),
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

/// Manual implementation of `AbstractSimulator` for HI enum
unsafe impl<const W: usize> AbstractSimulator for HIEnum<W> {
    fn start(&mut self, t_start: f64) -> f64 {
        match self {
            HIEnum::Leaf(leaf) => leaf.start(t_start),
            HIEnum::Branch(branch) => branch.start(t_start),
        }
    }

    fn stop(&mut self, t_stop: f64) {
        match self {
            HIEnum::Leaf(leaf) => leaf.stop(t_stop),
            HIEnum::Branch(branch) => branch.stop(t_stop),
        }
    }

    fn lambda(&mut self, output: &mut Self::Output, t: f64) {
        match self {
            HIEnum::Leaf(leaf) => leaf.lambda(output, t),
            HIEnum::Branch(branch) => branch.lambda(output, t),
        }
    }

    fn delta(&mut self, input: &mut Self::Input, output: &mut Self::Output, t: f64) -> f64 {
        match self {
            HIEnum::Leaf(leaf) => leaf.delta(input, output, t),
            HIEnum::Branch(branch) => branch.delta(input, output, t),
        }
    }
}

/// Manual implementation of `Component` for HI enum
impl<const W: usize> Component for HIEnum<W> {
    type Input = xdevs::Port<usize, 1>;

    type Output = xdevs::Port<usize, 1>;
}

/// Manual implementation of `SimTime` for HI enum
unsafe impl<const W: usize> SimTime for HIEnum<W> {
    fn get_t_last(&self) -> f64 {
        match self {
            HIEnum::Leaf(leaf) => leaf.get_t_last(),
            HIEnum::Branch(branch) => branch.get_t_last(),
        }
    }

    fn set_t_last(&mut self, t_last: f64) {
        match self {
            HIEnum::Leaf(leaf) => leaf.set_t_last(t_last),
            HIEnum::Branch(branch) => branch.set_t_last(t_last),
        }
    }

    fn get_t_next(&self) -> f64 {
        match self {
            HIEnum::Leaf(leaf) => leaf.get_t_next(),
            HIEnum::Branch(branch) => branch.get_t_next(),
        }
    }

    fn set_t_next(&mut self, t_next: f64) {
        match self {
            HIEnum::Leaf(leaf) => leaf.set_t_next(t_next),
            HIEnum::Branch(branch) => branch.set_t_next(t_next),
        }
    }
}

/// Manual implementation of the HI coupled model
pub struct HIModelComponents<const W: usize> {
    atomics: [AtomicModel; W],
    inner: Box<HIEnum<W>>,
}
impl<const W: usize> HIModelComponents<W> {
    #[inline]
    pub fn new(atomics: [AtomicModel; W], inner: Box<HIEnum<W>>) -> Self {
        Self { atomics, inner }
    }
}
#[doc = r" Wrapper struct holding all inner components' inputs."]
#[derive(xdevs::Bag)]
pub struct HIModelComponentsInput<const W: usize> {
    pub atomics: <[AtomicModel; W] as xdevs::traits::Component>::Input,
    pub inner: <Box<HIEnum<W>> as xdevs::traits::Component>::Input,
}
#[doc = r" Wrapper struct holding all inner components' outputs."]
#[derive(xdevs::Bag)]
pub struct HIModelComponentsOutput<const W: usize> {
    pub atomics: <[AtomicModel; W] as xdevs::traits::Component>::Output,
    pub inner: <Box<HIEnum<W>> as xdevs::traits::Component>::Output,
}
pub struct HIModel<const W: usize> {
    pub t_last: f64,
    pub t_next: f64,
    pub components: HIModelComponents<W>,
    pub components_input: HIModelComponentsInput<W>,
    pub components_output: HIModelComponentsOutput<W>,
}
impl<const W: usize> HIModel<W> {
    #[inline]
    pub fn build(atomics: [AtomicModel; W], inner: Box<HIEnum<W>>) -> Self {
        Self {
            t_last: 0.0,
            t_next: f64::INFINITY,
            components: HIModelComponents::new(atomics, inner),
            components_input: <HIModelComponentsInput<W> as xdevs::traits::Bag>::build(),
            components_output: <HIModelComponentsOutput<W> as xdevs::traits::Bag>::build(),
        }
    }
}

impl<const W: usize> xdevs::traits::Component for HIModel<W> {
    type Input = xdevs::Port<usize, 1>;
    type Output = xdevs::Port<usize, 1>;
}

unsafe impl<const W: usize> xdevs::traits::SimTime for HIModel<W> {
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
unsafe impl<const W: usize> xdevs::traits::PartialCoupled for HIModel<W> {
    type ComponentsInput = HIModelComponentsInput<W>;
    type ComponentsOutput = HIModelComponentsOutput<W>;
}
unsafe impl<const W: usize> xdevs::traits::AbstractSimulator for HIModel<W> {
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
        // get minimum t_next from all components after executing their delta
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
        ::xdevs::traits::SimTime::set_t_last(self, t);
        ::xdevs::traits::SimTime::set_t_next(self, t_next);

        // clear input and output events
        <Self::Input as ::xdevs::traits::Bag>::clear(input);
        <Self::Output as ::xdevs::traits::Bag>::clear(output);

        t_next
    }
}

impl<const W: usize> xdevs::Coupled for HIModel<W> {
    fn eic(from: &Self::Input, to: &mut Self::ComponentsInput) {
        for atom_ports in to.atomics.iter_mut() {
            let _ = from.couple(&mut atom_ports.input_port);
        }

        let _ = from.couple(&mut to.inner);
    }

    fn eoc(from: &Self::ComponentsOutput, to: &mut Self::Output) {
        let _ = from.inner.couple(to);
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

impl<const W: usize> HIModel<W> {
    pub fn new(inner: Box<HIEnum<W>>) -> Self {
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

/// End model with Generator and HI model coupled together
#[xdevs::coupled]
pub struct TopModel<const W: usize> {
    #[input]
    input_port: xdevs::port::Port<usize, 1>,
    #[output]
    output_port: xdevs::port::Port<usize, 1>,
    #[components]
    generator: JobGenerator,
    hi_model: HIEnum<W>,
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
        let _ = from.generator.out_job.couple(&mut to.hi_model);
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
        let top_model = simulator.get_model();

        assert_eq!(expected_n_atomic(WIDTH, DEPTH), top_model.get_n_atomics());
        assert_eq!(expected_n_events(WIDTH, DEPTH), top_model.get_n_events());
        assert_eq!(top_model.get_n_internals(), top_model.get_n_externals());
    }
}
