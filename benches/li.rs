use criterion::{criterion_group, criterion_main, Criterion};
use xdevs::{
    devstone::common::{Devstone, JobGenerator},
    devstone::li,
    generate_li,
    prelude::*,
    Config, Instant,
};
#[cfg(feature = "alloc")]
use xdevs::{devstone::li_box, generate_li_box};

fn bench_li(c: &mut Criterion) {
    const WIDTH: usize = 400;
    const DEPTH: usize = 400;
    const W: usize = WIDTH - 1;
    const N: usize = (WIDTH - 1) * (DEPTH - 1) + 1;
    generate_li!(400, 400, 0, 0);
    let generator = JobGenerator::new(5);
    let top_model: li::TopModel<'_, W> = li::TopModel::build(generator, &mut model_li);
    let mut simulator = top_model.to_simulator();
    let config = Config::new(Instant::from_secs(0), Instant::from_secs(10), 1, None);

    let mut group = c.benchmark_group("li-group");
    group.bench_function("li-sim", |b| {
        b.iter(|| {
            simulator.reset();
            simulator.simulate_vt(&config);
            assert_eq!(N, simulator.get_n_atomics());
            assert_eq!(N, simulator.get_n_events());
            assert_eq!(simulator.get_n_internals(), simulator.get_n_externals());
        })
    });
    group.finish();
}

fn bench_li_cycles(c: &mut Criterion) {
    const WIDTH: usize = 10;
    const DEPTH: usize = 10;
    const W: usize = WIDTH - 1;
    const N: usize = (WIDTH - 1) * (DEPTH - 1) + 1;
    generate_li!(10, 10, 1000, 1000);
    let generator = JobGenerator::new(5);
    let top_model: li::TopModel<'_, W> = li::TopModel::build(generator, &mut model_li);
    let mut simulator = top_model.to_simulator();
    let config = Config::new(Instant::from_secs(0), Instant::from_secs(10), 1, None);

    let mut group = c.benchmark_group("li-group");
    group.bench_function("li-cycles-sim", |b| {
        b.iter(|| {
            simulator.reset();
            simulator.simulate_vt(&config);
            assert_eq!(N, simulator.get_n_atomics());
            assert_eq!(N, simulator.get_n_events());
            assert_eq!(simulator.get_n_internals(), simulator.get_n_externals());
        })
    });
    group.finish();
}

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "alloc")]
fn bench_li_box(c: &mut Criterion) {
    const WIDTH: usize = 400;
    const DEPTH: usize = 400;
    const W: usize = WIDTH - 1;
    const N: usize = (WIDTH - 1) * (DEPTH - 1) + 1;
    generate_li_box!(400, 400, 0, 0);
    let generator = JobGenerator::new(5);
    let top_model: li_box::TopModel<W> = li_box::TopModel::build(generator, model_li);
    let mut simulator = top_model.to_simulator();
    let config = Config::new(Instant::from_secs(0), Instant::from_secs(10), 1, None);

    let mut group = c.benchmark_group("li-group");
    group.bench_function("li-box-sim", |b| {
        b.iter(|| {
            simulator.reset();
            simulator.simulate_vt(&config);
            assert_eq!(N, simulator.get_n_atomics());
            assert_eq!(N, simulator.get_n_events());
            assert_eq!(simulator.get_n_internals(), simulator.get_n_externals());
        })
    });
    group.finish();
}

#[cfg(feature = "alloc")]
criterion_group!(benches, bench_li, bench_li_cycles, bench_li_box);
#[cfg(not(feature = "alloc"))]
criterion_group!(benches, bench_li, bench_li_cycles);
criterion_main!(benches);
