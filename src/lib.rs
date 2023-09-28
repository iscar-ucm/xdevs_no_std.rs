#![no_std]
#![feature(trace_macros)]

trace_macros!(true);

pub use xdevs_no_std_macros::*;

pub mod atomic;
pub mod component;
pub mod port;
pub mod simulator;

use crate as xdevs;

atomic!(
    component = {
        name = MyAtomic,
        input = [
            in_ack<usize, 4>,
            in_ready<bool, 3>
        ],
        output=[
            out_job<usize, 1>
        ]
    },
    state = usize,
    constant = true,
);

impl xdevs::atomic::Atomic for MyAtomic {
    fn delta_int(state: &mut Self::State) {
        *state += 1;
    }

    fn delta_ext(state: &mut Self::State, e: f64, x: &Self::Input) {
        let msg = x.in_ack.get_values();
        *state += 1;
    }

    fn lambda(state: &Self::State, output: &mut Self::Output) {
        output.out_job.add_value(1);
    }

    fn ta(state: &Self::State) -> f64 {
        1.0
    }
}
