#[cfg(not(feature = "std"))]
use crate::Instant;
use crate::{
    Atomic, AtomicKind, Bag, Component, ComponentsInput, ComponentsOutput, Coupled, CoupledKind,
    Duration, Port,
};
#[cfg(feature = "std")]
use cpu_time::ThreadTime;

/// Simple atomic model that generates jobs and sends them to the input port of the model
pub struct JobGenerator {
    sigma: Duration,
    count: usize,
}

impl Component for JobGenerator {
    type Kind = AtomicKind;
    type Input = ();
    type Output = Port<usize, 1>;
}

impl Atomic for JobGenerator {
    fn delta_int(&mut self) {
        self.sigma = Duration::MAX;
    }

    fn lambda(&self, output: &mut Self::Output) {
        let _ = output.add_value(self.count);
    }

    fn ta(&self) -> Duration {
        self.sigma
    }

    fn delta_ext(&mut self, _elapsed: Duration, _input: &Self::Input) {
        self.sigma = Duration::MAX;
    }
}

impl JobGenerator {
    pub fn new(val_count: usize) -> Self {
        Self {
            sigma: Duration::from_secs(0),
            count: val_count,
        }
    }

    pub fn reset(&mut self) {
        self.sigma = Duration::from_secs(0);
    }
}

/// Simple atomic model
pub struct AtomicModel {
    sigma: Duration,
    n_internals: usize,
    n_externals: usize,
    n_events: usize,
    int_delay: Duration,
    ext_delay: Duration,
}

fn burn_cycles(duration: Duration) {
    let (now, during) = match () {
        #[cfg(not(feature = "std"))]
        () => (Instant::now(), duration),
        #[cfg(feature = "std")]
        () => (
            ThreadTime::now(),
            core::time::Duration::from_micros(Duration::as_micros(&duration)),
        ),
    };

    let mut x: usize = 0;
    while now.elapsed() < during {
        core::hint::black_box(x);
        x = x.wrapping_add(1);
    }
}

impl Component for AtomicModel {
    type Kind = AtomicKind;
    type Input = Port<usize, 1>;
    type Output = Port<usize, 1>;
}

impl Atomic for AtomicModel {
    fn delta_int(&mut self) {
        self.sigma = Duration::MAX;
        self.n_internals += 1;
        if self.int_delay > Duration::MIN {
            burn_cycles(self.int_delay);
        }
    }

    fn lambda(&self, output: &mut Self::Output) {
        let _ = output.add_value(self.n_events);
    }

    fn ta(&self) -> Duration {
        self.sigma
    }

    fn delta_ext(&mut self, _elapsed: Duration, input: &Self::Input) {
        self.sigma = Duration::from_secs(0);
        self.n_externals += 1;
        self.n_events += input.get_values().len();
        if self.ext_delay > Duration::MIN {
            burn_cycles(self.ext_delay);
        }
    }
}

impl Default for AtomicModel {
    fn default() -> Self {
        Self::new(0, 0)
    }
}

impl AtomicModel {
    pub fn new(int_delay: u64, ext_delay: u64) -> Self {
        Self {
            sigma: Duration::MAX,
            n_internals: 0,
            n_externals: 0,
            n_events: 0,
            int_delay: Duration::from_micros(int_delay),
            ext_delay: Duration::from_micros(ext_delay),
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
        self.sigma = Duration::MAX;
        self.n_internals = 0;
        self.n_externals = 0;
        self.n_events = 0;
    }
}

/// Leaf coupled model with only one atomic in LI models and HI leaf model
#[crate::coupled]
pub struct LeafModel {
    atomic: AtomicModel,
}

impl Component for LeafModel {
    type Kind = CoupledKind;
    type Input = Port<usize, 1>;
    type Output = Port<usize, 1>;
}

impl LeafModel {
    pub fn new(int_delay: u64, ext_delay: u64) -> Self {
        Self::build(AtomicModel::new(int_delay, ext_delay))
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
        Self::new(0, 0)
    }
}

impl Coupled for LeafModel {
    fn eic(from: &Self::Input, to: &mut ComponentsInput<Self>) {
        let _ = from.couple(&mut to.atomic);
    }
    fn eoc(from: &ComponentsOutput<Self>, to: &mut Self::Output) {
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
    use Atomic;

    #[test]
    fn job_generator_emits_configured_count() {
        let gen = JobGenerator::new(5);

        let mut output = <JobGenerator as Component>::Output::default();
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
        assert_eq!(gen.sigma, Duration::from_secs(0), "sigma starts at 0");
        gen.delta_ext(Duration::from_secs(1), &());
        assert_eq!(
            gen.sigma,
            Duration::MAX,
            "delta_ext should set sigma to MAX"
        );
    }

    #[test]
    fn leaf_model_contains_single_atomic() {
        // Verify that the LeafModel contains exactly one atomic model
        assert_eq!(LeafModel::default().get_n_atomics(), 1);
    }
}
