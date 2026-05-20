extern crate std;

use crate::{
    simulator::Config,
    traits::{AsyncInput, Bag},
    Duration as eDuration, Instant as eInstant,
};

//use std::time::Duration as StdDuration;
use std::{thread, time::SystemTime};

/// Closure for RT simulation on targets with `std`.
/// It sleeps until the next state transition.
pub fn sleep<T: Bag>(config: &Config) -> impl FnMut(eInstant, &mut T) -> eInstant {
    wait_event(config, |waiting_period, _| {
        thread::sleep(std::time::Duration::from_millis(waiting_period.as_millis()))
    })
    //embassy_time::Duration::from_nanos(waiting_period.as_nanos() as u64)
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
///    - `instant: [eInstant]` - Instant for external events.
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
    mut input_handler: impl FnMut(eInstant, &mut T),
) -> impl FnMut(eInstant, &mut T) -> eInstant {
    let (mult, max_jitter) = (config.mult, config.max_jitter);
    let mut last_rt = SystemTime::now();
    let start_rt = last_rt;

    move |t_until, binput: &mut T| -> eInstant {
        // TODO casteo

        // Timer::at(t_until);
        // Timer::now()

        //let next_rt = last_rt + Duration::from_secs((t_until - t_from) * mult);
        //let duration_embassy = t_until - eInstant::now();
        let duration_embassy = t_until.saturating_duration_since(eInstant::now());
        let duration_std = std::time::Duration::from_millis(duration_embassy.as_millis());
        let next_rt = last_rt + duration_std * (mult as u32);

        if let Ok(duration) = next_rt.duration_since(SystemTime::now()) {
            input_handler(
                eInstant::now().saturating_add(eDuration::from_millis(duration.as_millis() as u64)),
                binput,
            );
        }

        let t = SystemTime::now();

        match t.duration_since(next_rt) {
            Ok(duration) => {
                // t >= next_rt, check for the jitter
                if let Some(max_jitter) = max_jitter {
                    if duration_embassy > max_jitter {
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
                let dur_std =
                    std::time::Duration::from_millis(duration.as_millis() as u64) / (mult as u32);
                eInstant::now().saturating_add(eDuration::from_millis(dur_std.as_millis() as u64))
            }
        }
    }
}

/// A simple asynchronous input handler that sleeps until the next state transition of the model.
#[derive(Default)]
pub struct SleepAsync<T: Bag> {
    /// The last recorded real time instant.
    last_rt: Option<eInstant>,
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

    async fn handle(&mut self, _input: &mut Self::Input) {
        //no devuelve nada. en el propio simulador se comprueba que se realice en el tiempo correcto
        //let last_rt = self.last_rt.unwrap_or_else(eInstant::now);
        //let next_rt = last_rt + (t_until - Timer::now()) * (config.mult as u32);
        //Timer::at(t_until).await;
        // Timer::at(next_rt).await;
        // self.last_rt = Some(next_rt);
        //t_until
        //Timer::now() //más preciso que t_until por si ha habido alguna variación
        core::future::pending::<()>().await; //<()> significa que no devuelve nada
    }
}
