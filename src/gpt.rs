/// Generator that produces jobs at a fixed period until told to stop.
pub struct Generator {
    sigma: f64,
    period: f64,
    count: usize,
}

impl xdevs::Component for Generator {
    type Kind = xdevs::AtomicKind;
    type Input = xdevs::Port<bool, 1>;
    type Output = xdevs::Port<usize, 1>;
}

impl xdevs::Atomic for Generator {
    fn delta_int(&mut self) {
        self.count += 1;
        self.sigma = self.period;
    }

    fn lambda(&self, output: &mut Self::Output) {
        #[cfg(feature = "std")]
        std::println!("[G] sending job {}", self.count);
        output.add_value(self.count).unwrap();
    }

    fn ta(&self) -> f64 {
        self.sigma
    }

    fn delta_ext(&mut self, elapsed: f64, input: &Self::Input) {
        self.sigma -= elapsed;
        if let Some(&stop) = input.get_values().last() {
            #[cfg(feature = "std")]
            std::println!("[G] received stop: {}", stop);
            if stop {
                self.sigma = f64::INFINITY;
            }
        }
    }
}

impl Generator {
    pub fn new(period: f64) -> Self {
        Self {
            sigma: 0.0,
            period,
            count: 0,
        }
    }
}

/// Processor that receives a job, processes it for a fixed duration, then outputs it.
pub struct Processor {
    sigma: f64,
    time: f64,
    job: Option<usize>,
}

impl xdevs::Component for Processor {
    type Kind = xdevs::AtomicKind;
    type Input = xdevs::Port<usize, 1>;
    type Output = xdevs::Port<usize, 1>;
}

impl xdevs::Atomic for Processor {
    fn delta_int(&mut self) {
        self.sigma = f64::INFINITY;
        if self.job.is_some() {
            #[cfg(feature = "std")]
            std::println!("[P] processed job {}", self.job.unwrap());
            self.job = None;
        }
    }

    fn lambda(&self, output: &mut Self::Output) {
        if let Some(job) = self.job {
            output.add_value(job).unwrap();
        }
    }

    fn ta(&self) -> f64 {
        self.sigma
    }

    fn delta_ext(&mut self, elapsed: f64, input: &Self::Input) {
        self.sigma -= elapsed;
        if let Some(&job) = input.get_values().last() {
            #[cfg(feature = "std")]
            std::print!("[P] received job {}", job);
            if self.job.is_none() {
                #[cfg(feature = "std")]
                std::println!(" (idle)");
                self.job = Some(job);
                self.sigma = self.time;
            } else {
                #[cfg(feature = "std")]
                std::println!(" (busy)");
            }
        }
    }
}

impl Processor {
    pub fn new(time: f64) -> Self {
        Self {
            sigma: 0.0,
            time,
            job: None,
        }
    }
}

/// Input bag for the Transducer.
#[derive(xdevs::Bag)]
pub struct TransducerInput {
    pub in_generator: xdevs::Port<usize, 1>,
    pub in_processor: xdevs::Port<usize, 1>,
}

/// Transducer that observes generated and processed jobs, computes metrics,
/// and sends a stop signal to the Generator.
pub struct Transducer {
    sigma: f64,
    clock: f64,
    n_generated: usize,
    n_processed: usize,
}

impl xdevs::Component for Transducer {
    type Kind = xdevs::AtomicKind;
    type Input = TransducerInput;
    type Output = xdevs::Port<bool, 1>;
}

impl xdevs::Atomic for Transducer {
    fn delta_int(&mut self) {
        self.clock += self.sigma;
        #[cfg(feature = "std")]
        std::println!(
            "[T] acceptance: {:.2}, throughput: {:.2}",
            self.acceptance(),
            self.throughput()
        );
        self.sigma = f64::INFINITY;
    }

    fn lambda(&self, output: &mut Self::Output) {
        output.add_value(true).unwrap();
    }

    fn ta(&self) -> f64 {
        self.sigma
    }

    fn delta_ext(&mut self, elapsed: f64, input: &Self::Input) {
        self.sigma -= elapsed;
        self.clock += elapsed;
        self.n_generated += input.in_generator.get_values().len();
        self.n_processed += input.in_processor.get_values().len();
    }
}

impl Transducer {
    pub fn new(obs_time: f64) -> Self {
        Self {
            sigma: obs_time,
            clock: 0.0,
            n_generated: 0,
            n_processed: 0,
        }
    }

    pub fn acceptance(&self) -> f64 {
        if self.n_processed > 0 {
            self.n_processed as f64 / self.n_generated as f64
        } else {
            0.0
        }
    }

    pub fn throughput(&self) -> f64 {
        if self.n_processed > 0 {
            self.n_processed as f64 / self.clock
        } else {
            0.0
        }
    }
}

#[xdevs::coupled]
pub struct GPT {
    generator: Generator,
    processor: Processor,
    transducer: Transducer,
}

impl xdevs::Component for GPT {
    type Kind = xdevs::CoupledKind;
    type Input = ();
    type Output = ();
}

impl xdevs::Coupled for GPT {
    fn ic(
        from: &xdevs::component::coupled::ComponentsOutput<Self>,
        to: &mut xdevs::component::coupled::ComponentsInput<Self>,
    ) {
        from.generator.couple(&mut to.processor).unwrap();
        from.processor
            .couple(&mut to.transducer.in_processor)
            .unwrap();
        from.generator
            .couple(&mut to.transducer.in_generator)
            .unwrap();
        from.transducer.couple(&mut to.generator).unwrap();
    }
}

#[xdevs::coupled]
pub struct EF {
    generator: Generator,
    transducer: Transducer,
}

impl xdevs::Component for EF {
    type Kind = xdevs::CoupledKind;
    type Input = xdevs::Port<usize, 1>;
    type Output = xdevs::Port<usize, 1>;
}

impl xdevs::Coupled for EF {
    fn ic(
        from: &xdevs::component::coupled::ComponentsOutput<Self>,
        to: &mut xdevs::component::coupled::ComponentsInput<Self>,
    ) {
        from.generator
            .couple(&mut to.transducer.in_generator)
            .unwrap();
        from.transducer.couple(&mut to.generator).unwrap();
    }
    fn eic(from: &Self::Input, to: &mut xdevs::component::coupled::ComponentsInput<Self>) {
        from.couple(&mut to.transducer.in_processor).unwrap();
    }
    fn eoc(from: &xdevs::component::coupled::ComponentsOutput<Self>, to: &mut Self::Output) {
        from.generator.couple(to).unwrap();
    }
}

#[xdevs::coupled]
pub struct EFP {
    ef: EF,
    processor: Processor,
}

impl xdevs::Component for EFP {
    type Kind = xdevs::CoupledKind;
    type Input = ();
    type Output = ();
}

impl xdevs::Coupled for EFP {
    fn ic(
        from: &xdevs::component::coupled::ComponentsOutput<Self>,
        to: &mut xdevs::component::coupled::ComponentsInput<Self>,
    ) {
        from.ef.couple(&mut to.processor).unwrap();
        from.processor.couple(&mut to.ef).unwrap();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::port::Bag;
    use crate::simulation::{AbstractSimulator, Config, Simulable};
    use crate::{Atomic, Component};

    #[test]
    fn generator_emits_sequential_jobs() {
        let mut gen = Generator::new(1.0);
        assert_eq!(gen.ta(), 0.0, "generator should fire immediately");

        let mut out = <Generator as Component>::Output::build();
        gen.lambda(&mut out);
        assert_eq!(out.get_values(), &[0], "first job should be 0");
        gen.delta_int();
        assert_eq!(gen.ta(), 1.0, "ta should be the period after delta_int");
        out.clear();

        gen.lambda(&mut out);
        assert_eq!(out.get_values(), &[1], "second job should be 1");
        gen.delta_int();
        assert_eq!(gen.ta(), 1.0, "ta should be the period after delta_int");
        out.clear();

        gen.lambda(&mut out);
        assert_eq!(out.get_values(), &[2], "third job should be 2");
        gen.delta_int();
        assert_eq!(gen.ta(), 1.0, "ta should be the period after delta_int");
        out.clear();
    }

    #[test]
    fn generator_stops_on_stop_signal() {
        let mut gen = Generator::new(1.0);
        assert_eq!(gen.ta(), 0.0);

        let mut input = <Generator as Component>::Input::build();
        input.add_value(true).unwrap();
        gen.delta_conf(&input);

        assert_eq!(
            gen.ta(),
            f64::INFINITY,
            "generator should stop after receiving stop=true"
        );
    }

    #[test]
    fn generator_does_not_stop_without_stop_signal() {
        let mut gen = Generator::new(1.0);
        assert_eq!(gen.ta(), 0.0);

        let mut input = <Generator as Component>::Input::build();
        let mut ta = gen.ta();
        let elapsed = 0.5;

        gen.delta_ext(elapsed, &input);
        assert_eq!(
            gen.ta(),
            ta - elapsed,
            "generator should not stop without stop signal"
        );

        input.add_value(false).unwrap();
        ta = gen.ta();
        gen.delta_ext(elapsed, &input);

        assert_eq!(
            gen.ta(),
            ta - elapsed,
            "generator should not stop after receiving stop=false"
        );
    }

    #[test]
    fn processor_receives_and_processes_job() {
        let mut proc = Processor::new(2.5);
        assert_eq!(proc.ta(), 0.0, "processor should fire immediately (idle)");

        let mut out = <Processor as Component>::Output::build();
        proc.lambda(&mut out);
        assert!(out.is_empty(), "no job initially");
        proc.delta_int();
        assert_eq!(
            proc.ta(),
            f64::INFINITY,
            "processor should be awaiting a job after delta_int"
        );

        let mut input = <Processor as Component>::Input::build();
        input.add_value(42).unwrap();
        proc.delta_ext(0.0, &input);
        assert_eq!(proc.ta(), 2.5, "processor should be busy for 2.5 seconds");

        proc.lambda(&mut out);
        assert_eq!(out.get_values(), &[42]);
        proc.delta_int();
        assert_eq!(
            proc.ta(),
            f64::INFINITY,
            "processor should be idle after processing"
        );
        out.clear();
    }

    #[test]
    fn processor_ignores_jobs_while_busy() {
        let mut proc = Processor::new(2.5);

        let mut input = <Processor as Component>::Input::build();
        input.add_value(10).unwrap();
        proc.delta_ext(0.0, &input);

        let mut input = <Processor as Component>::Input::build();
        input.add_value(20).unwrap();
        proc.delta_ext(1.0, &input);

        let mut out = <Processor as Component>::Output::build();
        proc.lambda(&mut out);
        assert_eq!(
            out.get_values(),
            &[10],
            "should retain original job when busy"
        );
        proc.delta_int();

        let mut input = <Processor as Component>::Input::build();
        input.add_value(30).unwrap();
        proc.delta_ext(0.0, &input);

        let mut out = <Processor as Component>::Output::build();
        proc.lambda(&mut out);
        assert_eq!(out.get_values(), &[30], "should accept new job after idle");
    }

    #[test]
    fn transducer_counts_and_computes_metrics() {
        let mut trans = Transducer::new(10.0);
        assert_eq!(trans.ta(), 10.0);
        assert_eq!(trans.acceptance(), 0.0);
        assert_eq!(trans.throughput(), 0.0);

        let mut input = TransducerInput::build();
        input.in_generator.add_value(0).unwrap();
        trans.delta_ext(1.0, &input);

        let mut input = TransducerInput::build();
        input.in_generator.add_value(1).unwrap();
        trans.delta_ext(1.0, &input);

        let mut input = TransducerInput::build();
        input.in_generator.add_value(2).unwrap();
        input.in_processor.add_value(0).unwrap();
        trans.delta_ext(3.0, &input);

        trans.delta_int();

        assert_eq!(trans.ta(), f64::INFINITY, "transducer stops after obs_time");
        assert!(
            trans.acceptance() == 1.0 / 3.0,
            "acceptance = n_processed / n_generated = 1/3"
        );
        assert!(
            trans.throughput() == 1.0 / 10.0,
            "throughput = n_processed / clock = 1/10"
        );
    }

    #[test]
    fn transducer_sends_stop_signal() {
        let trans = Transducer::new(10.0);
        let mut output = <Transducer as Component>::Output::build();
        trans.lambda(&mut output);
        assert_eq!(output.get_values(), &[true], "should send stop signal");
    }

    #[test]
    fn gpt_simulation_runs() {
        let period = 1.0;
        let processing_time = 2.5;
        let obs_time = 10.0;

        // Generator fires at t = 0, period, 2*period, ..., obs_time
        let n_generated = (obs_time / period) as usize + 1;
        // Processor handles job 0 immediately, then every k-th job where
        // k = ceil(processing_time / period) — the next job that arrives
        // after or exactly when the processor becomes idle.
        let k = f64::ceil(processing_time / period) as usize;
        let n_processed = 1 + (n_generated - 1) / k;
        let last_completion = ((n_processed - 1) * k) as f64 * period + processing_time;
        let clock = last_completion.max(obs_time);

        let expected_acceptance = n_processed as f64 / n_generated as f64;
        let expected_throughput = n_processed as f64 / clock;

        let gen = Generator::new(period);
        let proc = Processor::new(processing_time);
        let trans = Transducer::new(obs_time);
        let model = GPT::build(gen, proc, trans);
        let mut sim = model.to_simulator();
        let config = Config::new(0.0, 20.0, 1.0, None);
        sim.simulate_vt(&config);

        let trans = &*sim.components.transducer;
        let acceptance = trans.acceptance();
        let throughput = trans.throughput();
        assert!(
            acceptance == expected_acceptance,
            "acceptance: expected {expected_acceptance}, got {acceptance}"
        );
        assert!(
            throughput == expected_throughput,
            "throughput: expected {expected_throughput}, got {throughput}"
        );
    }

    #[test]
    fn efp_simulation_runs() {
        let period = 1.0;
        let processing_time = 2.5;
        let obs_time = 10.0;

        let n_generated = (obs_time / period) as usize + 1;
        let k = f64::ceil(processing_time / period) as usize;
        let n_processed = 1 + (n_generated - 1) / k;
        let last_completion = ((n_processed - 1) * k) as f64 * period + processing_time;
        let clock = last_completion.max(obs_time);

        let expected_acceptance = n_processed as f64 / n_generated as f64;
        let expected_throughput = n_processed as f64 / clock;

        let gen = Generator::new(period);
        let proc = Processor::new(processing_time);
        let ef = EF::build(gen, Transducer::new(obs_time));
        let efp = EFP::build(ef, proc);
        let mut sim = efp.to_simulator();
        let config = Config::new(0.0, 20.0, 1.0, None);
        sim.simulate_vt(&config);

        let trans = &*sim.components.ef.components.transducer;
        let acceptance = trans.acceptance();
        let throughput = trans.throughput();
        assert!(
            acceptance == expected_acceptance,
            "acceptance: expected {expected_acceptance}, got {acceptance}",
        );
        assert!(
            throughput == expected_throughput,
            "throughput: expected {expected_throughput}, got {throughput}",
        );
    }
}
