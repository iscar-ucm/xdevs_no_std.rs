#![no_std]
#[cfg(feature = "alloc")]
extern crate alloc;
extern crate self as xdevs;
#[cfg(feature = "std")]
extern crate std;

pub mod atomic;
pub mod component;
pub mod coupled;
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

pub use atomic::Atomic;
pub use component::{AtomicKind, Component, CoupledKind};
pub use coupled::Coupled;
pub use embassy_time::{Duration, Instant};
pub use port::Port;
pub use simulator::{Config, Simulator};
pub use xdevs_no_std_macros::*;
