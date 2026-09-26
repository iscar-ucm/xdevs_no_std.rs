use core::future::Future;

use crate::Duration;

pub trait Clock {
    type Instant;

    /// Returns the current time in microseconds.
    fn now() -> Self::Instant;

    /// Perform the wait operation until the specified instant is reached or input_handler is completed.
    fn wait_until(
        t0: &Self::Instant,
        t_until: Duration,
        input_handler: impl Future<Output = ()>,
        mult: u64,
    ) -> impl Future<Output = Duration>;
}
