use crate::processor::Processor;

/// Simple atomic model that generates jobs and sends them to the input port of the model
pub struct JobGenerator {
    sigma: f64,
    count: usize,
}

impl xdevs::Component for JobGenerator {
    type Kind = xdevs::AtomicKind;
    type Input = ();
    type Output = xdevs::port::Port<usize, 1>;
}

impl xdevs::Atomic for JobGenerator {
    fn delta_int(&mut self) {
        self.sigma = f64::INFINITY;
    }

    fn lambda(&self, output: &mut Self::Output) {
        let _ = output.add_value(self.count);
    }

    fn ta(&self) -> f64 {
        self.sigma
    }

    fn delta_ext(&mut self, _elapsed: f64, _input: &Self::Input) {}
}

impl JobGenerator {
    pub fn new(val_count: usize) -> Self {
        Self {
            sigma: 0.0,
            count: val_count,
        }
    }
}

/// Simple atomic model
pub struct AtomicModel {
    sigma: f64,
    n_internals: usize,
    n_externals: usize,
    n_events: usize,
}

impl xdevs::Component for AtomicModel {
    type Kind = xdevs::AtomicKind;
    type Input = xdevs::port::Port<usize, 1>;
    type Output = xdevs::port::Port<usize, 1>;
}

impl xdevs::Atomic for AtomicModel {
    fn delta_int(&mut self) {
        self.sigma = f64::INFINITY;
        self.n_internals += 1;
    }

    fn lambda(&self, output: &mut Self::Output) {
        let _ = output.add_value(self.n_events);
    }

    fn ta(&self) -> f64 {
        self.sigma
    }

    fn delta_ext(&mut self, _elapsed: f64, input: &Self::Input) {
        self.sigma = 0.0;
        self.n_externals += 1;
        self.n_events += input.get_values().len();
    }
}

impl Default for AtomicModel {
    fn default() -> Self {
        Self::new()
    }
}

impl AtomicModel {
    pub fn new() -> Self {
        Self {
            sigma: f64::INFINITY,
            n_internals: 0,
            n_externals: 0,
            n_events: 0,
        }
    }

    pub fn get_n_internals(&self) -> usize {
        self.n_internals
    }

    pub fn get_n_externals(&self) -> usize {
        self.n_externals
    }

    pub fn get_n_events(&self) -> usize {
        self.n_events
    }

    pub fn get_n_atomics(&self) -> usize {
        1
    }
}

/// Leaf coupled model with only one atomic in LI models and HI leaf model
#[xdevs::coupled]
pub struct LeafModel {
    atomic: AtomicModel,
}

impl xdevs::Component for LeafModel {
    type Kind = xdevs::CoupledKind;
    type Input = xdevs::Port<usize, 1>;
    type Output = xdevs::Port<usize, 1>;
}

impl LeafModel {
    pub fn new() -> Self {
        Self::build(AtomicModel::new())
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

impl Default for LeafModel {
    fn default() -> Self {
        Self::new()
    }
}

impl xdevs::Coupled for LeafModel {
    fn eic(from: &Self::Input, to: &mut Self::ComponentsInput) {
        let _ = from.couple(&mut to.atomic);
    }
    fn eoc(from: &Self::ComponentsOutput, to: &mut Self::Output) {
        let _ = from.atomic.couple(to);
    }
}
