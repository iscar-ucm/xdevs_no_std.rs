#![no_std]
#![feature(trace_macros)]

trace_macros!(true);

pub use xdevs_no_std_macros::*;

pub mod port;

use crate as xdevs;

#[component(input=[in_ack<usize, 4>, in_ready<bool, 3>], output=[out_job<usize, 1>])]
pub struct Component {
    id: u8,
}