#![no_std]
#[cfg(feature = "alloc")]
extern crate alloc;
extern crate self as xdevs;
#[cfg(feature = "std")]
extern crate std;

pub mod bag;
pub mod component;
pub mod devstone;
pub mod export;
pub mod gpt;
#[cfg(any(feature = "embassy", feature = "std"))]
pub mod rt_engine;
pub mod simulation;
/// Reusable Kani proof functions for user-defined models (only under `cargo kani`).
#[cfg(kani)]
pub mod verification;

pub use bag::{adapt_and_couple, couple, Bag};
pub use component::{
    atomic::Atomic,
    coupled::{ComponentsInput, ComponentsOutput, Coupled},
    AtomicKind, Component, ComponentsKind, CoupledKind,
};
pub use embassy_time::{Duration, Instant};
pub use simulation::Config;
pub use xdevs_no_std_macros::*;

/// Port is an alias for a heapless::Vec.
pub type Port<T, const N: usize> = heapless::Vec<T, N>;

/// Map bag type alias depending on the available features.
///
/// - If only the `alloc` feature is enabled, `MapBag` refers to `HashbrownMapBag`.
/// - If the `std` feature is enabled, `MapBag` refers to `StdMapBag`.
#[cfg(all(feature = "alloc", not(feature = "std")))]
pub use bag::map_bag::HashbrownMapBag as MapBag;

/// Map bag type alias depending on the available features.
///
/// - If only the `alloc` feature is enabled, `MapBag` refers to `HashbrownMapBag`.
/// - If the `std` feature is enabled, `MapBag` refers to `StdMapBag`.
#[cfg(feature = "std")]
pub use bag::map_bag::StdMapBag as MapBag;

/// Prelude with the traits needed to call the high-level simulation methods
/// (`.to_simulator()`, `.simulate_vt()`, `.simulate_rt()`, `.simulate_rt_async()`)
/// directly on components and simulators.
///
/// Intended to be imported with `use xdevs::prelude::*;`.
pub mod prelude {
    pub use crate::{
        bag::{adapt_and_couple, couple, Bag},
        component::{
            atomic::Atomic,
            coupled::{ComponentsInput, ComponentsOutput, Coupled},
            AtomicKind, Component, ComponentsKind, CoupledKind,
        },
        simulation::{AbstractSimulator, Config, Simulable},
        Port,
    };

    #[cfg(feature = "alloc")]
    pub use crate::bag::map_bag::HashbrownMapBag;
    #[cfg(feature = "std")]
    pub use crate::bag::map_bag::StdMapBag;
}
