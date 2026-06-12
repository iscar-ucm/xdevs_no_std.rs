#![no_std]
#[cfg(feature = "alloc")]
extern crate alloc;
extern crate self as xdevs;
#[cfg(feature = "std")]
extern crate std;

pub mod component;
#[cfg(feature = "alloc")]
pub mod devstone;
pub mod export;
mod impls;
pub mod port;
pub mod processor;
#[cfg(any(feature = "embassy", feature = "std"))]
pub mod rt_engine;
pub mod simulator;
pub mod traits;

pub use component::{atomic::Atomic, coupled::Coupled, AtomicKind, Component, CoupledKind};
pub use embassy_time::{Duration, Instant};
pub use port::Port;
pub use simulator::{Config, Simulator};
pub use xdevs_no_std_macros::*;
