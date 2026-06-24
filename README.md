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

Both `std` and `embassy` enable executor-dependent tools. They are mutually exclusive and interchangeable.

## Work in progress!

There is still a lot of work to do! However, we already proved the effectiveness of this simulator.

## References

1. R. Cárdenas, P. Malagón, P. Arroba and J. L. Risco-Martín, "[xDEVS no_std: A Rust Crate for Real-Time DEVS on Embedded Systems](https://doi.org/10.23919/ANNSIM61499.2024.10732777)," 2024 Annual Modeling and Simulation Conference (ANNSIM), Washington D.C., USA, 2024, pp. 1-13.

## License

Licensed under [GPL-3.0-or-later](LICENSE).
