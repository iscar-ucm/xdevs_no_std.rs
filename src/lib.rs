#![no_std]
#![feature(trace_macros)]

trace_macros!(true);

pub use xdevs_no_std_macros::*;

pub mod atomic;
pub mod component;
pub mod port;
pub mod simulator;
