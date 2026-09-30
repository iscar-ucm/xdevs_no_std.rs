use core::future::Future;

use crate::Duration;

pub trait Clock {
    /// Captures the current instant as the start time of the simulation.
    fn start(&mut self);

    /// Waits until `t_until` of wall-clock time has elapsed since
    /// [`start`](Self::start), or until `input_handler` completes, whichever
    /// happens first. Returns the wall-clock time elapsed since `start`.
    ///
    /// # Note
    ///
    /// Both `t_until` and the returned duration are measured from the instant
    /// captured by [`start`](Self::start). The returned duration must not be
    /// greater than `t_until`. Otherwise, the simulation might panic due to
    /// excessive jitter.
    fn wait_until(
        &self,
        t_until: Duration,
        input_handler: impl Future<Output = ()>,
    ) -> impl Future<Output = Duration>;
}
