use criterion::{criterion_group, criterion_main, Criterion};
use xdevs::{
    devstone::common::{Devstone, JobGenerator},
    devstone::ho::TopModel,
    generate_ho, AbstractSimulator, Config, Simulable,
};

fn bench_ho(c: &mut Criterion) {
    const WIDTH: usize = 400;
    const DEPTH: usize = 400;
    const W: usize = WIDTH - 1;
    const N: usize = (WIDTH - 1) * (DEPTH - 1) + 1;
    const E: usize = 1 + (DEPTH - 1) * ((WIDTH - 1) * WIDTH) / 2;
    generate_ho!(400, 400);
    let generator = JobGenerator::new(5);
    let top_model: TopModel<'_, W> = TopModel::build(generator, &mut model_ho);
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

criterion_group!(benches, bench_ho);
criterion_main!(benches);
