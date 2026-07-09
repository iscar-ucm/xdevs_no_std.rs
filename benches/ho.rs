use criterion::{criterion_group, criterion_main, Criterion};
use xdevs::{
    devstone::common::{Devstone, JobGenerator},
    devstone::ho,
    generate_ho, AbstractSimulator, Config, Simulable,
};
#[cfg(feature = "alloc")]
use xdevs::{devstone::ho_box, generate_ho_box};

fn bench_ho(c: &mut Criterion) {
    const WIDTH: usize = 400;
    const DEPTH: usize = 400;
    const W: usize = WIDTH - 1;
    const N: usize = (WIDTH - 1) * (DEPTH - 1) + 1;
    const E: usize = 1 + (DEPTH - 1) * ((WIDTH - 1) * WIDTH) / 2;
    generate_ho!(400, 400, 0, 0);
    let generator = JobGenerator::new(5);
    let top_model: ho::TopModel<'_, W> = ho::TopModel::build(generator, &mut model_ho);
    let mut simulator = top_model.to_simulator();
    let config = Config::new(0.0, 10.0, 1.0, None);

    let mut group = c.benchmark_group("ho-group");
    group.bench_function("ho-sim", |b| {
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

fn bench_ho_cycles(c: &mut Criterion) {
    const WIDTH: usize = 10;
    const DEPTH: usize = 10;
    const W: usize = WIDTH - 1;
    const N: usize = (WIDTH - 1) * (DEPTH - 1) + 1;
    const E: usize = 1 + (DEPTH - 1) * ((WIDTH - 1) * WIDTH) / 2;
    generate_ho!(10, 10, 1000, 1000);
    let generator = JobGenerator::new(5);
    let top_model: ho::TopModel<'_, W> = ho::TopModel::build(generator, &mut model_ho);
    let mut simulator = top_model.to_simulator();
    let config = Config::new(0.0, 10.0, 1.0, None);

    let mut group = c.benchmark_group("ho-group");
    group.bench_function("ho-cycles-sim", |b| {
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
fn bench_ho_box(c: &mut Criterion) {
    const WIDTH: usize = 400;
    const DEPTH: usize = 400;
    const W: usize = WIDTH - 1;
    const N: usize = (WIDTH - 1) * (DEPTH - 1) + 1;
    const E: usize = 1 + (DEPTH - 1) * ((WIDTH - 1) * WIDTH) / 2;
    generate_ho_box!(400, 400, 0, 0);
    let generator = JobGenerator::new(5);
    let top_model: ho_box::TopModel<W> = ho_box::TopModel::build(generator, model_ho);
    let mut simulator = top_model.to_simulator();
    let config = Config::new(0.0, 10.0, 1.0, None);

    let mut group = c.benchmark_group("ho-group");
    group.bench_function("ho-box-sim", |b| {
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
criterion_group!(benches, bench_ho, bench_ho_cycles, bench_ho_box);
#[cfg(not(feature = "alloc"))]
criterion_group!(benches, bench_ho, bench_ho_cycles);
criterion_main!(benches);
