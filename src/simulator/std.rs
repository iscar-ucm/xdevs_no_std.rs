extern crate std;
use std::time::{Duration, SystemTime};

/// Closure for RT simulation on targets with `std`.
/// It sleeps until the next state transition.
///
pub fn sleep<T: crate::aux::Port>(
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
