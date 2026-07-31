#![no_std]
#[cfg(feature = "alloc")]
extern crate alloc;
extern crate self as xdevs;
#[cfg(feature = "std")]
extern crate std;

pub mod component;
pub mod devstone;
pub mod export;
pub mod gpt;
pub mod port;
#[cfg(any(feature = "embassy", feature = "std"))]
pub mod rt_engine;
pub mod simulation;

pub use component::{
    atomic::Atomic,
    coupled::{ComponentsInput, ComponentsOutput, Coupled},
    AtomicKind, Component, ComponentsKind, CoupledKind,
};
pub use embassy_time::{Duration, Instant};
pub use port::{Bag, Port};
pub use simulation::Config;
pub use xdevs_no_std_macros::*;

/// Prelude with the traits needed to call the high-level simulation methods
/// (`.to_simulator()`, `.simulate_vt()`, `.simulate_rt()`, `.simulate_rt_async()`)
/// directly on components and simulators.
///
/// Intended to be imported with `use xdevs::prelude::*;`.
pub mod prelude {
    pub use crate::port::Bag;
    pub use crate::simulation::{AbstractSimulator, Simulable};
}
