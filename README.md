[![crates.io](https://img.shields.io/crates/d/xdevs_no_std.svg)](https://crates.io/crates/xdevs_no_std)
[![crates.io](https://img.shields.io/crates/v/xdevs_no_std.svg)](https://crates.io/crates/xdevs_no_std)
[![codecov](https://codecov.io/gh/iscar-ucm/xdevs_no_std.rs/graph/badge.svg)](https://codecov.io/gh/iscar-ucm/xdevs_no_std.rs)

# `xdevs_no_std.rs`

`no_std` version of xDEVS for Rust.

## Run Real-Time DEVS simulations on embedded devices!

This crate is mainly focused on enabling the execution of Real-Time simulations on embedded devices.
This allows a robust Model-Based design approach, from a mathematical model to a real IoT application.

## Key features

- **`no_std` first** — zero heap allocations by default make the simulator compatible with any target.
- **Performant** - test it by yourself using the built-in DEVStone benchmark implementation.
- **Real-Time guarantees** - for critical IoT applications in which timing is key.
- **`async` support** - seamless interaction between your simulation and external tasks.
- **Extra tools for the most popular executors** — choose between `std` (Tokio) and `embassy` backends.

## Feature flags

| Feature | Description |
|---------|-------------|
| `std` | Tokio-based async backend. Enables heap-allocated (`alloc`) variants. |
| `embassy` | Embassy-based async backend for bare-metal targets. |
| `alloc` | Enables `Box` of DEVS models and `Box`-based DEVStone models. |
| `rayon` | Parallel simulation of independent components (arrays/tuples/struct fields). Implies `std`; components must be `Send`. |

Both `std` and `embassy` enable executor-dependent tools. They are mutually exclusive and interchangeable.

## Formal verification

Core simulator invariants are proven with the [Kani](https://model-checking.github.io/kani/) model checker.
General simulator proofs assume `t_last <= t_next` and `t_last <= t <= t_next`, allowing zero time advance.
Quiescent proofs assume `t_last < t_next` and `t_last <= t < t_next`, excluding zero time advance and internal transitions.
Under these quiescent conditions and empty input, we prove that `Simulator::delta` is a strict no-op (no transition function invoked; component state, `t_last`, and `t_next` untouched).
All reusable verification tools live under [`src/verification/`](src/verification/).
All verification code is compiled only under `cfg(kani)`.

Kani verifies *monomorphized* code, so these proofs certify the concrete model types they are instantiated with.
To check **your** models, the `xdevs::verification` module (only compiled under `cargo kani`) provides reusable proof functions you can instantiate with your own types:

```rust,ignore
#[cfg(kani)]
mod proofs {
    use super::MyModel;

    #[kani::proof]
    fn my_model_quiescence() {
        let sigma: f64 = kani::any();
        kani::assume(sigma >= 0.0); // constrain symbolic state to your model's invariants
        let my_model = MyModel::new(sigma);
        xdevs::verification::simulation::simulator::delta_is_noop_when_quiescent(my_model);
    }
}
```

- `simulation::simulator::{delta_is_noop_when_quiescent, lambda_is_noop_when_quiescent}`:
  check quiescent behavior (`t_last < t_next`, `t_last <= t < t_next`, empty input) after `start` at a symbolic finite time.
  `delta` preserves the post-start model state and returns the same next event time; neither call produces output.
  These helpers do not inspect private timing fields or construct arbitrary internal simulator states.
- `component::atomic::atomic_sanity`: `ta()` is non-negative (and not NaN) before and after every transition function, and no transition panics, overflows, or accesses memory out of bounds for the symbolic state/input you provide.
- `bag::{build_is_empty, clear_is_empty, is_empty_matches_iteration}`: newly built and cleared bags report empty, repeated clearing stays empty, and the separate iteration check verifies that emptiness agrees with iteration.
  Pass valid symbolic states to the helpers from your own `#[kani::proof]` harnesses.
- `bag::check_bag_laws::<B>(max_insertions)`: runs these checks after up to the
  specified number of symbolic insertions. Requires `B::Value: kani::Arbitrary`;
  the calling harness must provide a sufficient `#[kani::unwind(...)]` bound.

The allocated-vector harness also covers `std::vec::Vec`, with up to four symbolic
`u8` insertions. Enable `alloc` to include it, as Kani CI does.

To run the proofs, [install Kani](https://model-checking.github.io/kani/install-guide.html) and execute:

```sh
cargo kani --features alloc
```

## Work in progress!

There is still a lot of work to do! However, we already proved the effectiveness of this simulator.

## References

1. R. Cárdenas, P. Malagón, P. Arroba and J. L. Risco-Martín, "[xDEVS no_std: A Rust Crate for Real-Time DEVS on Embedded Systems](https://doi.org/10.23919/ANNSIM61499.2024.10732777)," 2024 Annual Modeling and Simulation Conference (ANNSIM), Washington D.C., USA, 2024, pp. 1-13.

## License

Licensed under [GPL-3.0-or-later](LICENSE).
