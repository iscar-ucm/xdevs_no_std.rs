/// Simple atomic model that generates jobs and sends them to the input port of the model
pub struct JobGenerator {
    sigma: f64,
    count: usize,
}

impl xdevs::Component for JobGenerator {
    type Kind = xdevs::AtomicKind;
    type Input = ();
    type Output = xdevs::Port<usize, 1>;
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

    fn delta_ext(&mut self, _elapsed: f64, _input: &Self::Input) {
        self.sigma = f64::INFINITY;
    }
}

impl JobGenerator {
    pub fn new(val_count: usize) -> Self {
        Self {
            sigma: 0.0,
            count: val_count,
        }
    }

    pub fn reset(&mut self) {
        self.sigma = 0.0;
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
    type Input = xdevs::Port<usize, 1>;
    type Output = xdevs::Port<usize, 1>;
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
}

pub trait Devstone {
    fn get_n_internals(&self) -> usize;
    fn get_n_externals(&self) -> usize;
    fn get_n_events(&self) -> usize;
    fn get_n_atomics(&self) -> usize;
    fn reset(&mut self);
}

impl Devstone for AtomicModel {
    fn get_n_internals(&self) -> usize {
        self.n_internals
    }

    fn get_n_externals(&self) -> usize {
        self.n_externals
    }

    fn get_n_events(&self) -> usize {
        self.n_events
    }

    fn get_n_atomics(&self) -> usize {
        1
    }

    fn reset(&mut self) {
        self.sigma = f64::INFINITY;
        self.n_internals = 0;
        self.n_externals = 0;
        self.n_events = 0;
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
}

impl Devstone for LeafModel {
    fn get_n_internals(&self) -> usize {
        self.components.atomic.get_n_internals()
    }

    fn get_n_externals(&self) -> usize {
        self.components.atomic.get_n_externals()
    }

    fn get_n_events(&self) -> usize {
        self.components.atomic.get_n_events()
    }

    fn get_n_atomics(&self) -> usize {
        self.components.atomic.get_n_atomics()
    }

    fn reset(&mut self) {
        self.components.atomic.reset();
    }
}

impl Default for LeafModel {
    fn default() -> Self {
        Self::new()
    }
}

impl xdevs::Coupled for LeafModel {
    fn eic(from: &Self::Input, to: &mut xdevs::ComponentsInput<Self>) {
        let _ = from.couple(&mut to.atomic);
    }
    fn eoc(from: &xdevs::ComponentsOutput<Self>, to: &mut Self::Output) {
        let _ = from.atomic.couple(to);
    }
}

#[macro_export]
macro_rules! impl_devstone_leaf {
    () => {
        fn get_n_internals(&self) -> usize {
            self.components.atomic.get_n_internals()
        }
        fn get_n_externals(&self) -> usize {
            self.components.atomic.get_n_externals()
        }
        fn get_n_events(&self) -> usize {
            self.components.atomic.get_n_events()
        }
        fn get_n_atomics(&self) -> usize {
            self.components.atomic.get_n_atomics()
        }
        fn reset(&mut self) {
            self.components.atomic.reset();
        }
    };
}

#[macro_export]
macro_rules! impl_devstone_enum {
    () => {
        fn get_n_internals(&self) -> usize {
            match self {
                Self::Leaf(leaf) => leaf.get_n_internals(),
                Self::Branch(branch) => branch.get_n_internals(),
            }
        }
        fn get_n_externals(&self) -> usize {
            match self {
                Self::Leaf(leaf) => leaf.get_n_externals(),
                Self::Branch(branch) => branch.get_n_externals(),
            }
        }
        fn get_n_events(&self) -> usize {
            match self {
                Self::Leaf(leaf) => leaf.get_n_events(),
                Self::Branch(branch) => branch.get_n_events(),
            }
        }
        fn get_n_atomics(&self) -> usize {
            match self {
                Self::Leaf(leaf) => leaf.get_n_atomics(),
                Self::Branch(branch) => branch.get_n_atomics(),
            }
        }
        fn reset(&mut self) {
            match self {
                Self::Leaf(leaf) => leaf.reset(),
                Self::Branch(branch) => branch.reset(),
            }
        }
    };
}

#[macro_export]
macro_rules! impl_devstone_coupled {
    () => {
        fn get_n_internals(&self) -> usize {
            let mut sum = self.components.inner.get_n_internals();
            for a in self.components.atomics.iter() {
                sum += a.get_n_internals();
            }
            sum
        }
        fn get_n_externals(&self) -> usize {
            let mut sum = self.components.inner.get_n_externals();
            for a in self.components.atomics.iter() {
                sum += a.get_n_externals();
            }
            sum
        }
        fn get_n_events(&self) -> usize {
            let mut sum = self.components.inner.get_n_events();
            for a in self.components.atomics.iter() {
                sum += a.get_n_events();
            }
            sum
        }
        fn get_n_atomics(&self) -> usize {
            let mut sum = self.components.inner.get_n_atomics();
            for _ in self.components.atomics.iter() {
                sum += 1;
            }
            sum
        }
        fn reset(&mut self) {
            self.components.inner.reset();
            for a in self.components.atomics.iter_mut() {
                a.reset();
            }
        }
    };
}

#[macro_export]
macro_rules! impl_devstone_top {
    ($child:ident $(, $extra:ident)*) => {
        fn get_n_internals(&self) -> usize {
            self.components.$child.get_n_internals()
        }
        fn get_n_externals(&self) -> usize {
            self.components.$child.get_n_externals()
        }
        fn get_n_events(&self) -> usize {
            self.components.$child.get_n_events()
        }
        fn get_n_atomics(&self) -> usize {
            self.components.$child.get_n_atomics()
        }
        fn reset(&mut self) {
            self.components.$child.reset();
            $(self.components.$extra.reset();)*
        }
    };
}

#[cfg(test)]
mod test {
    use super::*;
    use xdevs::Atomic;

    #[test]
    fn job_generator_emits_configured_count() {
        let gen = JobGenerator::new(5);

        let mut output = <JobGenerator as xdevs::Component>::Output::default();
        gen.lambda(&mut output);
        assert_eq!(
            output.get_values(),
            &[5],
            "generator should output its count"
        );
    }

    #[test]
    fn generator_sets_sigma_to_infinity_on_external_event() {
        let mut gen = JobGenerator::new(5);
        assert_eq!(gen.sigma, 0.0, "delta_ext should set sigma to infinity");
        gen.delta_ext(1.0, &());
        assert_eq!(
            gen.sigma,
            f64::INFINITY,
            "delta_ext should set sigma to infinity"
        );
    }

    #[test]
    fn leaf_model_contains_single_atomic() {
        // Verify that the LeafModel contains exactly one atomic model
        assert_eq!(LeafModel::default().get_n_atomics(), 1);
    }
}
