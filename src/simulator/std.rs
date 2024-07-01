extern crate std;
use std::{cmp, time::{Duration, SystemTime}};

/// Closure for RT simulation on targets with `std`.
/// It sleeps until the next state transition.
pub fn sleep<T: crate::traits::Bag>(
    t_start: f64,
    time_scale: f64,
    max_jitter: Option<std::time::Duration>,
) -> impl FnMut(f64, &mut T) -> f64 {
    let mut last_vt = t_start;
    let mut last_rt = SystemTime::now();

    move |t_next, _| -> f64 {
        let next_rt = last_rt + Duration::from_secs_f64((t_next - last_vt) * time_scale);
        match next_rt.duration_since(SystemTime::now()) {
            Ok(duration) => std::thread::sleep(duration),
            Err(err) => {
                if let Some(max_jitter) = max_jitter {
                    if err.duration() > max_jitter {
                        panic!("Jitter too high");
                    }
                }
            }
        }
        last_vt = t_next;
        last_rt = next_rt;

        t_next
    }
}

/// Closure for waiting for an event in real-time simulation.
/// It calculates the next real-time and virtual-time based on the given time scale and sleeps until the next state transition.
/// If the maximum jitter is provided, it checks if the actual time exceeds the future time and panics if it does.
/// It also calls the input handler function with the duration between the current and next real-time.
///
/// # Arguments
///
/// * `t_start` - The starting virtual time.
/// * `time_scale` - The time scale factor.
/// * `max_jitter` - The maximum allowed jitter duration.
/// * `input_handler` - The function to handle the input event.
///
/// # Returns
///
/// A closure that takes the next virtual time and a mutable reference to the bag and returns the next virtual time.
pub fn wait_event<T: crate::traits::Bag>(
    t_start: f64,
    time_scale: f64,
    max_jitter: Option<std::time::Duration>,
    mut input_handler: impl FnMut(Duration, &mut T),
) -> impl FnMut(f64, &mut T) -> f64 {
    
    let mut last_vt = t_start;
    let mut last_rt = SystemTime::now();
    let start_rt = last_rt;

    move |t_next, binput: &mut T| -> f64 {
        
        if t_next < last_vt {
            panic!("Virtual time higher than t_next");
        }
        let next_rt = last_rt + Duration::from_secs_f64((t_next - last_vt) * time_scale);
        match next_rt.duration_since(SystemTime::now()) {
            Ok(duration) => input_handler(duration, binput),
            Err(err) => {
                if let Some(max_jitter) = max_jitter {
                    // println!("Hay Jitter: {:?}", err.duration());
                    if err.duration() > max_jitter {
                        panic!("Jitter too high");
                    }
                }
            }
        }
        let t = SystemTime::now();
        
        // Update time
        last_rt = cmp::min(next_rt, t);
        
        if t < next_rt {
            let duration_from_str = last_rt.duration_since(start_rt).unwrap();
            last_vt = duration_from_str.as_secs_f64() / time_scale;
        }else {
            last_vt = t_next;
        }
        last_vt 
        
    }
}