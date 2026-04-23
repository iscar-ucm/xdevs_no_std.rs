use crate::{
    simulator::Config,
    traits::{AsyncInput, Bag},
};
use core::time::Duration;
use embassy_time::{Instant, Timer};

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
        let duration = Duration::from_secs_f64((t_until - t_from) * config.time_scale);
        let next_rt = last_rt + duration.try_into().unwrap();
        Timer::at(next_rt).await;
        self.last_rt = Some(next_rt);
        t_until
    }
}
