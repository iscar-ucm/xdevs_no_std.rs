#![no_std]
#![feature(trace_macros)]

trace_macros!(true);

pub use xdevs_no_std_macros::*;

pub mod atomic;
pub mod component;
pub mod coupled;
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

    fn delta_ext(state: &mut Self::State, _e: f64, x: &Self::Input) {
        let _msg = x.in_ack.get_values();
        *state += 1;
    }

    fn lambda(_state: &Self::State, output: &mut Self::Output) {
        output.out_job.add_value(1).expect("port full!");
    }

    fn ta(_state: &Self::State) -> f64 {
        1.0
    }
}

coupled!(
    component = {
        name = MyCoupled,
        input = [
            in_ack<usize, 4>,
            in_ready<bool, 3>
        ],
        output=[
            out_job<usize, 1>
        ]
    },
    components = MyAtomic,
    eic = [
        in_ack -> my_atomic.in_ack,
        ],
    constant = true,
);
