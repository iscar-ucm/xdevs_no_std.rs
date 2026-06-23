use criterion::{criterion_group, criterion_main, Criterion};
use xdevs::{
    devstone::{common::JobGenerator, ho::TopModel},
    generate_ho, Config,
};

extern crate alloc;

fn bench_ho(c: &mut Criterion) {
    use xdevs::simulation::{AbstractSimulator, Simulable};
    const W: usize = 399; // WIDTH - 1
    generate_ho!(400, 400);
    let generator = JobGenerator::new(5);
    let top_model: TopModel<W> = TopModel::build(generator, model_ho);
    let mut simulator = top_model.to_simulator();
    let config = Config::new(0.0, 10.0, 1.0, None);

    let mut group = c.benchmark_group("ho-group");
    group.bench_function("ho-sim", |b| {
        b.iter(|| {
            simulator.simulate_vt(&config);
        })
    });
    group.finish();
}

criterion_group!(benches, bench_ho);
criterion_main!(benches);
