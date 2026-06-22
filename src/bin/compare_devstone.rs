//! Comparison binary for Box (alloc) vs &mut (ref) DEVStone implementations.
//!
//! Compile & Run:
//!   cargo run --bin compare_devstone --features alloc --release
//!   cargo run --bin compare_devstone --release
//!
//! Benchmarks LI, HI, HO at 100×100 (WIDTH=100, DEPTH=100).

use std::time::Instant;

extern crate alloc;

fn main() {
    // LI 100×100
    compare_li_100x100();
    // HI 100×100
    compare_hi_100x100();
    // HO 100×100
    compare_ho_100x100();
}

// ─── LI 100×100 ───────────────────────────────────────────────────────────────

#[cfg(feature = "alloc")]
fn run_boxed_li_100x100() {
    use xdevs::devstone::{common::JobGenerator, li::TopModel};
    use xdevs::simulation::{AbstractSimulator, Simulable};
    const W: usize = 99;

    let t0 = Instant::now();
    xdevs::generate_li!(100, 100);
    let gen = JobGenerator::new(5);
    let top: TopModel<W> = TopModel::build(gen, model_li);
    let build_time = t0.elapsed();

    let t0 = Instant::now();
    let mut sim = top.to_simulator();
    let create_time = t0.elapsed();

    let cfg = xdevs::simulation::Config::new(0.0, 10.0, 1.0, None);
    let t0 = Instant::now();
    sim.simulate_vt(&cfg);
    let sim_time = t0.elapsed();

    println!(
        "  Box:     build={:?} create={:?} sim={:?} | atomics={} events={} int={} ext={}",
        build_time,
        create_time,
        sim_time,
        sim.get_n_atomics(),
        sim.get_n_events(),
        sim.get_n_internals(),
        sim.get_n_externals(),
    );
}

#[cfg(not(feature = "alloc"))]
fn run_boxed_li_100x100() {
    println!("  Box:     SKIPPED (use --features alloc)");
}

fn run_ref_li_100x100() {
    use xdevs::devstone::{common::JobGenerator, li_ref::TopModelRef};
    use xdevs::simulation::{AbstractSimulator, Simulable};
    const W: usize = 99;

    let t0 = Instant::now();
    xdevs::generate_li_ref!(100, 100);
    let gen = JobGenerator::new(5);
    let top: TopModelRef<'_, W> = TopModelRef::build(gen, &mut model_li);
    let build_time = t0.elapsed();

    let t0 = Instant::now();
    let mut sim = top.to_simulator();
    let create_time = t0.elapsed();

    let cfg = xdevs::simulation::Config::new(0.0, 10.0, 1.0, None);
    let t0 = Instant::now();
    sim.simulate_vt(&cfg);
    let sim_time = t0.elapsed();

    println!(
        "  &mut:    build={:?} create={:?} sim={:?} | atomics={} events={} int={} ext={}",
        build_time,
        create_time,
        sim_time,
        sim.get_n_atomics(),
        sim.get_n_events(),
        sim.get_n_internals(),
        sim.get_n_externals(),
    );
}

fn compare_li_100x100() {
    println!("\n╔══════════════════════════════════════╗");
    println!("║  LI Model   WIDTH=100  DEPTH=100     ║");
    println!("╚══════════════════════════════════════╝");
    run_ref_li_100x100();
    run_boxed_li_100x100();
}

// ─── HI 100×100 ───────────────────────────────────────────────────────────────

#[cfg(feature = "alloc")]
fn run_boxed_hi_100x100() {
    use xdevs::devstone::{common::JobGenerator, hi::TopModel};
    use xdevs::simulation::{AbstractSimulator, Simulable};
    const W: usize = 99;

    let t0 = Instant::now();
    xdevs::generate_hi!(100, 100);
    let gen = JobGenerator::new(5);
    let top: TopModel<W> = TopModel::build(gen, model_hi);
    let build_time = t0.elapsed();

    let t0 = Instant::now();
    let mut sim = top.to_simulator();
    let create_time = t0.elapsed();

    let cfg = xdevs::simulation::Config::new(0.0, 10.0, 1.0, None);
    let t0 = Instant::now();
    sim.simulate_vt(&cfg);
    let sim_time = t0.elapsed();

    println!(
        "  Box:     build={:?} create={:?} sim={:?} | atomics={} events={} int={} ext={}",
        build_time,
        create_time,
        sim_time,
        sim.get_n_atomics(),
        sim.get_n_events(),
        sim.get_n_internals(),
        sim.get_n_externals(),
    );
}

#[cfg(not(feature = "alloc"))]
fn run_boxed_hi_100x100() {
    println!("  Box:     SKIPPED (use --features alloc)");
}

fn run_ref_hi_100x100() {
    use xdevs::devstone::{common::JobGenerator, hi_ref::TopModelRef};
    use xdevs::simulation::{AbstractSimulator, Simulable};
    const W: usize = 99;

    let t0 = Instant::now();
    xdevs::generate_hi_ref!(100, 100);
    let gen = JobGenerator::new(5);
    let top: TopModelRef<'_, W> = TopModelRef::build(gen, &mut model_hi);
    let build_time = t0.elapsed();

    let t0 = Instant::now();
    let mut sim = top.to_simulator();
    let create_time = t0.elapsed();

    let cfg = xdevs::simulation::Config::new(0.0, 10.0, 1.0, None);
    let t0 = Instant::now();
    sim.simulate_vt(&cfg);
    let sim_time = t0.elapsed();

    println!(
        "  &mut:    build={:?} create={:?} sim={:?} | atomics={} events={} int={} ext={}",
        build_time,
        create_time,
        sim_time,
        sim.get_n_atomics(),
        sim.get_n_events(),
        sim.get_n_internals(),
        sim.get_n_externals(),
    );
}

fn compare_hi_100x100() {
    println!("\n╔══════════════════════════════════════╗");
    println!("║  HI Model   WIDTH=100  DEPTH=100     ║");
    println!("╚══════════════════════════════════════╝");
    run_ref_hi_100x100();
    run_boxed_hi_100x100();
}

// ─── HO 100×100 ───────────────────────────────────────────────────────────────

#[cfg(feature = "alloc")]
fn run_boxed_ho_100x100() {
    use xdevs::devstone::{common::JobGenerator, ho::TopModel};
    use xdevs::simulation::{AbstractSimulator, Simulable};
    const W: usize = 99;

    let t0 = Instant::now();
    xdevs::generate_ho!(100, 100);
    let gen = JobGenerator::new(5);
    let top: TopModel<W> = TopModel::build(gen, model_ho);
    let build_time = t0.elapsed();

    let t0 = Instant::now();
    let mut sim = top.to_simulator();
    let create_time = t0.elapsed();

    let cfg = xdevs::simulation::Config::new(0.0, 10.0, 1.0, None);
    let t0 = Instant::now();
    sim.simulate_vt(&cfg);
    let sim_time = t0.elapsed();

    println!(
        "  Box:     build={:?} create={:?} sim={:?} | atomics={} events={} int={} ext={}",
        build_time,
        create_time,
        sim_time,
        sim.get_n_atomics(),
        sim.get_n_events(),
        sim.get_n_internals(),
        sim.get_n_externals(),
    );
}

#[cfg(not(feature = "alloc"))]
fn run_boxed_ho_100x100() {
    println!("  Box:     SKIPPED (use --features alloc)");
}

fn run_ref_ho_100x100() {
    use xdevs::devstone::{common::JobGenerator, ho_ref::TopModelRef};
    use xdevs::simulation::{AbstractSimulator, Simulable};
    const W: usize = 99;

    let t0 = Instant::now();
    xdevs::generate_ho_ref!(100, 100);
    let gen = JobGenerator::new(5);
    let top: TopModelRef<'_, W> = TopModelRef::build(gen, &mut model_ho);
    let build_time = t0.elapsed();

    let t0 = Instant::now();
    let mut sim = top.to_simulator();
    let create_time = t0.elapsed();

    let cfg = xdevs::simulation::Config::new(0.0, 10.0, 1.0, None);
    let t0 = Instant::now();
    sim.simulate_vt(&cfg);
    let sim_time = t0.elapsed();

    println!(
        "  &mut:    build={:?} create={:?} sim={:?} | atomics={} events={} int={} ext={}",
        build_time,
        create_time,
        sim_time,
        sim.get_n_atomics(),
        sim.get_n_events(),
        sim.get_n_internals(),
        sim.get_n_externals(),
    );
}

fn compare_ho_100x100() {
    println!("\n╔══════════════════════════════════════╗");
    println!("║  HO Model   WIDTH=100  DEPTH=100     ║");
    println!("╚══════════════════════════════════════╝");
    run_ref_ho_100x100();
    run_boxed_ho_100x100();
}
