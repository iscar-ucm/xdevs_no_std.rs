/// Simple atomic model that generates jobs and sends them to the input port of the model
#[xdevs::atomic]
pub struct JobGenerator {
    #[output]
    pub out_job: xdevs::port::Port<usize, 1>,
    #[state]
    sigma: f64,
    count: usize,
}

impl xdevs::Atomic for JobGenerator {
    fn delta_int(state: &mut Self::State) {
        state.sigma = f64::INFINITY;
    }

    fn lambda(state: &Self::State, output: &mut Self::Output) {
        let _ = output.out_job.add_value(state.count);
    }

    fn ta(state: &Self::State) -> f64 {
        state.sigma
    }

    fn delta_ext(_state: &mut Self::State, _elapsed: f64, _input: &Self::Input) {}
}

impl JobGenerator {
    pub fn new(val_count: usize) -> Self {
        Self::build(0.0, val_count)
    }
}

/// Simple atomic model
#[xdevs::atomic]
pub struct AtomicModel {
    #[input]
    pub input_port: xdevs::port::Port<usize, 1>,
    #[output]
    pub output_port: xdevs::port::Port<usize, 1>,
    #[state]
    sigma: f64,
    n_internals: usize,
    n_externals: usize,
    n_events: usize,
}

impl xdevs::Atomic for AtomicModel {
    fn delta_int(state: &mut Self::State) {
        state.sigma = f64::INFINITY;
        state.n_internals += 1;
    }

    fn lambda(state: &Self::State, output: &mut Self::Output) {
        let _ = output.output_port.add_value(state.n_events);
    }

    fn ta(state: &Self::State) -> f64 {
        state.sigma
    }

    fn delta_ext(state: &mut Self::State, _elapsed: f64, input: &Self::Input) {
        state.sigma = 0.0;
        state.n_externals += 1;
        state.n_events += input.input_port.get_values().len();
    }
}

impl AtomicModel {
    pub fn new() -> Self {
        Self::build(f64::INFINITY, 0, 0, 0)
    }

    pub fn get_n_internals(&self) -> usize {
        self.state.n_internals
    }

    pub fn get_n_externals(&self) -> usize {
        self.state.n_externals
    }

    pub fn get_n_events(&self) -> usize {
        self.state.n_events
    }

    pub fn get_n_atomics(&self) -> usize {
        1
    }
}

impl Default for AtomicModel {
    fn default() -> Self {
        Self::new()
    }
}

/// Manual implementation of the leaf coupled model with only one atomic in LI models and HI leaf model
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
pub struct LeafModel {
    pub t_last: f64,
    pub t_next: f64,
    pub components: LeafModelComponents,
    pub components_input: LeafModelComponentsInput,
    pub components_output: LeafModelComponentsOutput,
}
impl LeafModel {
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
impl xdevs::traits::Component for LeafModel {
    type Input = xdevs::Port<usize, 1>;
    type Output = xdevs::Port<usize, 1>;
}
unsafe impl xdevs::traits::SimTime for LeafModel {
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

unsafe impl xdevs::traits::PartialCoupled for LeafModel {
    type ComponentsInput = LeafModelComponentsInput;
    type ComponentsOutput = LeafModelComponentsOutput;
}
unsafe impl xdevs::traits::AbstractSimulator for LeafModel {
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
        ::xdevs::traits::SimTime::set_t_last(self, t);
        ::xdevs::traits::SimTime::set_t_next(self, t_next);

        // clear input and output events
        <Self::Input as ::xdevs::traits::Bag>::clear(input);
        <Self::Output as ::xdevs::traits::Bag>::clear(output);

        t_next
    }
}

impl LeafModel {
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

impl Default for LeafModel {
    fn default() -> Self {
        Self::new()
    }
}

impl xdevs::Coupled for LeafModel {
    fn eic(from: &Self::Input, to: &mut Self::ComponentsInput) {
        let _ = from.couple(&mut to.atomic.input_port);
    }
    fn eoc(from: &Self::ComponentsOutput, to: &mut Self::Output) {
        let _ = from.atomic.output_port.couple(to);
    }
}
