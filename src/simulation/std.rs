use crate::{
    port::Bag,
    simulation::{AsyncInput, Config},
    Duration as eDuration, Instant as eInstant,
};
use std::{thread, time::SystemTime};

/// Closure for RT simulation on targets with `std`.
/// It sleeps until the next state transition.
pub fn sleep<T: Bag>(config: &Config) -> impl FnMut(eInstant, &mut T) -> eInstant {
    wait_event(config, |waiting_period, _| {
        thread::sleep(std::time::Duration::from_millis(waiting_period.as_millis()))
    })
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
///  A closure that takes the deadline and a mutable reference to the bag and returns the next virtual time.
///
/// # Example
///
/// ```ignore
/// xdevs::simulator::std::wait_event(config, |duration, input| { /* ... */ });
/// ```
pub fn wait_event<T: Bag>(
    config: &Config,
    mut input_handler: impl FnMut(eInstant, &mut T),
) -> impl FnMut(eInstant, &mut T) -> eInstant {
    let (mult, max_jitter) = (config.mult, config.max_jitter);
    let mut last_rt = SystemTime::now();

    move |t_until, binput: &mut T| -> eInstant {
        let duration_embassy = t_until.saturating_duration_since(eInstant::now());
        let duration_std = std::time::Duration::from_millis(duration_embassy.as_millis());
        let wait_time = duration_std * (mult as u32);
        let next_rt = last_rt + wait_time;

        if let Ok(duration) = next_rt.duration_since(SystemTime::now()) {
            input_handler(
                eInstant::now().saturating_add(eDuration::from_millis(duration.as_millis() as u64)),
                binput,
            );
        }

        let t = SystemTime::now();

        match t.duration_since(next_rt) {
            Ok(overrun) => {
                if let Some(max_jitter) = max_jitter {
                    let jitter = eDuration::from_millis(overrun.as_millis() as u64);
                    if jitter > max_jitter {
                        panic!("[WE]>> Jitter too high: {:?}", overrun);
                    }
                }
                last_rt = next_rt;
                t_until
            }
            Err(_) => {
                last_rt = t;
                eInstant::now()
            }
        }
    }
}

/// A simple asynchronous input handler that sleeps until the next state transition of the model.
#[derive(Default)]
pub struct SleepAsync<T: Bag> {
    /// Phantom data to associate with the input bag type.
    input: core::marker::PhantomData<T>,
}

impl<T: Bag> SleepAsync<T> {
    /// Creates a new `SleepAsync` instance.
    pub fn new() -> Self {
        Self {
            input: core::marker::PhantomData,
        }
    }
}

impl<T: Bag> AsyncInput for SleepAsync<T> {
    type Input = T;

    async fn handle(&mut self, _input: &mut Self::Input) {
        core::future::pending::<()>().await
    }
}
