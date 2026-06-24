#![no_std]
#[cfg(feature = "alloc")]
extern crate alloc;
extern crate self as xdevs;
#[cfg(feature = "std")]
extern crate std;

pub mod component;
pub mod devstone;
pub mod export;
pub mod port;
#[cfg(any(feature = "embassy", feature = "std"))]
pub mod rt_engine;
pub mod simulation;

pub use component::{
    atomic::Atomic, coupled::Coupled, AtomicKind, Component, ComponentsKind, CoupledKind,
};
pub use embassy_time::{Duration, Instant};
pub use port::Port;
pub use simulation::{AbstractSimulator, Config};
pub use xdevs_no_std_macros::*;
