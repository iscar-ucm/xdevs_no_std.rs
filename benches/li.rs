use criterion::{criterion_group, criterion_main, Criterion};
use xdevs::{
    devstone::{common::JobGenerator, li::TopModel},
    generate_li, AbstractSimulator, Config, Simulable,
};

fn bench_li(c: &mut Criterion) {
    const W: usize = 399; // WIDTH - 1
    generate_li!(400, 400);
    let generator = JobGenerator::new(5);
    let top_model: TopModel<'_, W> = TopModel::build(generator, &mut model_li);
    let mut simulator = top_model.to_simulator();
    let config = Config::new(0.0, 10.0, 1.0, None);

    let mut group = c.benchmark_group("li-group");
    group.bench_function("li-sim", |b| {
        b.iter(|| {
            simulator.simulate_vt(&config);
        })
    });
    group.finish();
}

criterion_group!(benches, bench_li);
criterion_main!(benches);
