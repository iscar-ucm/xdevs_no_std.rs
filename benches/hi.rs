use criterion::{criterion_group, criterion_main, Criterion};
use xdevs::{
    devstone::common::{Devstone, JobGenerator},
    devstone::hi,
    generate_hi,
    prelude::*,
    Config, Instant,
};
#[cfg(feature = "alloc")]
use xdevs::{devstone::hi_box, generate_hi_box};

fn bench_hi(c: &mut Criterion) {
    const WIDTH: usize = 400;
    const DEPTH: usize = 400;
    const W: usize = WIDTH - 1;
    const N: usize = (WIDTH - 1) * (DEPTH - 1) + 1;
    const E: usize = 1 + (DEPTH - 1) * ((WIDTH - 1) * WIDTH) / 2;
    generate_hi!(400, 400, 0, 0);
    let generator = JobGenerator::new(5);
    let top_model: hi::TopModel<'_, W> = hi::TopModel::build(generator, &mut model_hi);
    let mut simulator = top_model.to_simulator();
    let config = Config::new(Instant::from_secs(0), Instant::from_secs(10), 1, None);

    let mut group = c.benchmark_group("hi-group");
    group.bench_function("hi-sim", |b| {
        b.iter(|| {
            simulator.reset();
            simulator.simulate_vt(&config);
            assert_eq!(N, simulator.get_n_atomics());
            assert_eq!(E, simulator.get_n_events());
            assert_eq!(simulator.get_n_internals(), simulator.get_n_externals());
        })
    });
    group.finish();
}

fn bench_hi_cycles(c: &mut Criterion) {
    const WIDTH: usize = 10;
    const DEPTH: usize = 10;
    const W: usize = WIDTH - 1;
    const N: usize = (WIDTH - 1) * (DEPTH - 1) + 1;
    const E: usize = 1 + (DEPTH - 1) * ((WIDTH - 1) * WIDTH) / 2;
    generate_hi!(10, 10, 1000, 1000);
    let generator = JobGenerator::new(5);
    let top_model: hi::TopModel<'_, W> = hi::TopModel::build(generator, &mut model_hi);
    let mut simulator = top_model.to_simulator();
    let config = Config::new(Instant::from_secs(0), Instant::from_secs(10), 1, None);

    let mut group = c.benchmark_group("hi-group");
    group.bench_function("hi-cycles-sim", |b| {
        b.iter(|| {
            simulator.reset();
            simulator.simulate_vt(&config);
            assert_eq!(N, simulator.get_n_atomics());
            assert_eq!(E, simulator.get_n_events());
            assert_eq!(simulator.get_n_internals(), simulator.get_n_externals());
        })
    });
    group.finish();
}

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "alloc")]
fn bench_hi_box(c: &mut Criterion) {
    const WIDTH: usize = 400;
    const DEPTH: usize = 400;
    const W: usize = WIDTH - 1;
    const N: usize = (WIDTH - 1) * (DEPTH - 1) + 1;
    const E: usize = 1 + (DEPTH - 1) * ((WIDTH - 1) * WIDTH) / 2;
    generate_hi_box!(400, 400, 0, 0);
    let generator = JobGenerator::new(5);
    let top_model: hi_box::TopModel<W> = hi_box::TopModel::build(generator, model_hi);
    let mut simulator = top_model.to_simulator();
    let config = Config::new(Instant::from_secs(0), Instant::from_secs(10), 1, None);

    let mut group = c.benchmark_group("hi-group");
    group.bench_function("hi-box-sim", |b| {
        b.iter(|| {
            simulator.reset();
            simulator.simulate_vt(&config);
            assert_eq!(N, simulator.get_n_atomics());
            assert_eq!(E, simulator.get_n_events());
            assert_eq!(simulator.get_n_internals(), simulator.get_n_externals());
        })
    });
    group.finish();
}

#[cfg(feature = "alloc")]
criterion_group!(benches, bench_hi, bench_hi_cycles, bench_hi_box);
#[cfg(not(feature = "alloc"))]
criterion_group!(benches, bench_hi, bench_hi_cycles);
criterion_main!(benches);
