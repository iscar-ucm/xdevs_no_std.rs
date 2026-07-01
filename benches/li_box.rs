use criterion::{criterion_group, criterion_main, Criterion};
use xdevs::{
    devstone::common::{Devstone, JobGenerator},
    devstone::li_box::TopModel,
    generate_li_box, AbstractSimulator, Config, Simulable,
};

extern crate alloc;

fn bench_li_box(c: &mut Criterion) {
    const WIDTH: usize = 400;
    const DEPTH: usize = 400;
    const W: usize = WIDTH - 1;
    const N: usize = (WIDTH - 1) * (DEPTH - 1) + 1;
    generate_li_box!(400, 400);
    let generator = JobGenerator::new(5);
    let top_model: TopModel<W> = TopModel::build(generator, model_li);
    let mut simulator = top_model.to_simulator();
    let config = Config::new(0.0, 10.0, 1.0, None);

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

criterion_group!(benches, bench_li_box);
criterion_main!(benches);
