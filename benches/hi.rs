use criterion::{criterion_group, criterion_main, Criterion};
use xdevs::{
    devstone::{common::JobGenerator, hi::TopModel},
    generate_hi, AbstractSimulator, Config, Simulable,
};

fn bench_hi(c: &mut Criterion) {
    const W: usize = 399; // WIDTH - 1
    generate_hi!(400, 400);
    let generator = JobGenerator::new(5);
    let top_model: TopModel<'_, W> = TopModel::build(generator, &mut model_hi);
    let mut simulator = top_model.to_simulator();
    let config = Config::new(0.0, 10.0, 1.0, None);

    let mut group = c.benchmark_group("hi-group");
    group.bench_function("hi-sim", |b| {
        b.iter(|| {
            simulator.simulate_vt(&config);
        })
    });
    group.finish();
}

criterion_group!(benches, bench_hi);
criterion_main!(benches);
