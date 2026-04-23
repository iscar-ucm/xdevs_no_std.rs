extern crate std;

use crate::{
    simulator::Config,
    traits::{AsyncInput, Bag},
};
use std::{
    thread,
    time::{Duration, Instant, SystemTime},
};

/// Closure for RT simulation on targets with `std`.
/// It sleeps until the next state transition.
pub fn sleep<T: Bag>(config: &Config) -> impl FnMut(f64, f64, &mut T) -> f64 {
    wait_event(config, |waiting_period, _| thread::sleep(waiting_period))
}

/// It computes the next wall-clock time corresponding to the next state transition of the model.
///
/// An input handler function waits for external events without exceeding the time for the next internal event.
/// Finally, it checks that the wall-clock drift does not exceed the maximum jitter allowed (if any) and panics if it does.
///
///  # Arguments
///
///  * `config` - The desired simulator configuration.
///  * `input_handler` - The function to handle incoming external events. This function expects two arguments:
///    - `duration: [Duration]` - Maximum duration of the time interval to wait for external events.
///      The input handler function may return earlier if an input event is received.
///      Note, however, that it must **NOT** return after, as it would result in an incorrect real-time implementation.
///    - `input_ports: &mut T` - Mutable reference to the input ports of the top-most model under simulation.
///    
///  # Returns
///
///  A closure that takes the current and next virtual time and a mutable reference to the bag and returns the next virtual time.
///
/// # Example
///
/// ```ignore
/// xdevs::simulator::std::wait_event(0., 1., Some(Duration::from_millis(50)), some_input_handler);
/// ```
pub fn wait_event<T: Bag>(
    config: &Config,
    mut input_handler: impl FnMut(Duration, &mut T),
) -> impl FnMut(f64, f64, &mut T) -> f64 {
    let (time_scale, max_jitter) = (config.time_scale, config.max_jitter);
    let mut last_rt = SystemTime::now();
    let start_rt = last_rt;

    move |t_from, t_until, binput: &mut T| -> f64 {
        let next_rt = last_rt + Duration::from_secs_f64((t_until - t_from) * time_scale);

        if let Ok(duration) = next_rt.duration_since(SystemTime::now()) {
            input_handler(duration, binput);
        }

        let t = SystemTime::now();

        match t.duration_since(next_rt) {
            Ok(duration) => {
                // t >= next_rt, check for the jitter
                if let Some(max_jitter) = max_jitter {
                    if duration > max_jitter {
                        panic!("[WE]>> Jitter too high: {:?}", duration);
                    }
                }
                last_rt = next_rt;
                t_until
            }
            Err(_) => {
                // t < next_rt
                last_rt = t;
                let duration = last_rt.duration_since(start_rt).unwrap();
                duration.as_secs_f64() / time_scale
            }
        }
    }
}

/// A simple asynchronous input handler that sleeps until the next state transition of the model.
#[derive(Default)]
pub struct SleepAsync<T: Bag> {
    /// The last recorded real time instant.
    last_rt: Option<Instant>,
    /// Phantom data to associate with the input bag type.
    input: core::marker::PhantomData<T>,
}

impl<T: Bag> SleepAsync<T> {
    /// Creates a new `SleepAsync` instance.
    pub fn new() -> Self {
        Self {
            last_rt: None,
            input: core::marker::PhantomData,
        }
    }
}

impl<T: Bag> AsyncInput for SleepAsync<T> {
    type Input = T;

    async fn handle(
        &mut self,
        config: &Config,
        t_from: f64,
        t_until: f64,
        _input: &mut Self::Input,
    ) -> f64 {
        let last_rt = self.last_rt.unwrap_or_else(Instant::now);
        let next_rt = last_rt + Duration::from_secs_f64((t_until - t_from) * config.time_scale);
        tokio::time::sleep_until(next_rt.into()).await;
        self.last_rt = Some(next_rt);
        t_until
    }
}
