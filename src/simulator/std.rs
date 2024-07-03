extern crate std;
use std::time::{Duration, SystemTime};

/// Closure for RT simulation on targets with `std`.
/// It sleeps until the next state transition.
pub fn sleep<T: crate::traits::Bag>(
    t_start: f64,
    time_scale: f64,
    max_jitter: Option<std::time::Duration>,
) -> impl FnMut(f64, &mut T) -> f64 {
    wait_event(t_start, time_scale, max_jitter, |waiting_period, _| {
        std::thread::sleep(waiting_period)
    })
}

///
/// It computes the next wall-clock time corresponding to the next state transition of the model.
///
/// An input handler function waits for external events without exceeding the time for the next internal event.
/// Finally, it checks that the wall-clock drift does not exceed the maximum jitter allowed (if any) and panics if it does.
///
///  # Arguments
///
///  * `t_start` - The virtual time at the beginning of the simulation.
///  * `time_scale` - The time scale factor between virtual and wall-clock time.
///  * `max_jitter` - The maximum allowed jitter duration. If `None`, no jitter check is performed.
///  * `input_handler` - The function to handle incoming external events. This function expects two arguments:
///    - `duration: [Duration]` - Maximum duration of the time interval to wait for external events.
///      The input handler function may return earlier if an input event is received.
///      Note, however, that it must **NOT** return after, as it would result in an incorrect real-time implementation.
///    - `input_ports: &mut T` - Mutable reference to the input ports of the top-most model under simulation.
///    
///  # Returns
///
///  A closure that takes the next virtual time and a mutable reference to the bag and returns the next virtual time.
///
/// # Example
///
/// ```ignore
/// xdevs::simulator::std::wait_event(0., 1., Some(Duration::from_millis(50)), some_input_handler);
/// ```

pub fn wait_event<T: crate::traits::Bag>(
    t_start: f64,
    time_scale: f64,
    max_jitter: Option<Duration>,
    mut input_handler: impl FnMut(Duration, &mut T),
) -> impl FnMut(f64, &mut T) -> f64 {
    let mut last_vt = t_start;
    let mut last_rt = SystemTime::now();
    let start_rt = last_rt;

    move |t_next, binput: &mut T| -> f64 {
        assert!(t_next >= last_vt);

        let next_rt = last_rt + Duration::from_secs_f64((t_next - last_vt) * time_scale);

        if let Ok(duration) = next_rt.duration_since(SystemTime::now()) {
            input_handler(duration, binput);
        }

        let t = SystemTime::now();

        last_vt = match t.duration_since(next_rt) {
            Ok(duration) => {
                // t >= next_rt, check for the jitter
                if let Some(max_jitter) = max_jitter {
                    if duration > max_jitter {
                        panic!("[WE]>> Jitter too high: {:?}", duration);
                    }
                }
                last_rt = next_rt;
                t_next
            }
            Err(_) => {
                // t < next_rt
                last_rt = t;
                let duration = last_rt.duration_since(start_rt).unwrap();
                duration.as_secs_f64() / time_scale
            }
        };

        last_vt
    }
}
