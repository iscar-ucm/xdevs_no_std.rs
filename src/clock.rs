use core::future::Future;

use crate::Duration;

pub trait Clock {
    /// Build a new clock instance.
    fn build(mult: u64) -> Self;

    /// Perform the wait operation until the specified instant is reached or input_handler is completed.
    fn wait_until(
        &self,
        t_until: Duration,
        input_handler: impl Future<Output = ()>,
    ) -> impl Future<Output = Duration>;
}
