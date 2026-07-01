use criterion::{criterion_group, criterion_main, Criterion};
use xdevs::{
    devstone::{common::JobGenerator, ho_box::TopModel},
    generate_ho_box, Config,
};

extern crate alloc;

fn bench_ho_box(c: &mut Criterion) {
    use xdevs::{AbstractSimulator, Simulable};
    const W: usize = 399; // WIDTH - 1
    generate_ho_box!(400, 400);
    let generator = JobGenerator::new(5);
    let top_model: TopModel<W> = TopModel::build(generator, model_ho);
    let mut simulator = top_model.to_simulator();
    let config = Config::new(0.0, 10.0, 1.0, None);

    let mut group = c.benchmark_group("ho-group");
    group.bench_function("ho-box-sim", |b| {
        b.iter(|| {
            simulator.simulate_vt(&config);
        })
    });
    group.finish();
}

criterion_group!(benches, bench_ho_box);
criterion_main!(benches);
