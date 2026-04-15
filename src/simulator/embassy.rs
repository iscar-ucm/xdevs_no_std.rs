use crate::{
    simulator::Config,
    traits::{AsyncInput, Bag},
};
use crate::{Duration, Instant};
use embassy_time::{with_deadline, Timer};

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

    async fn handle(&mut self, _input: &mut Self::Input) {
        // let last_rt = self.last_rt.unwrap_or_else(Instant::now);
        // let duration = (t_until - Instant::now()) * (config.mult as u32);
        // let next_rt = last_rt + duration.try_into().unwrap();
        //Timer::at(next_rt).await;
        //self.last_rt = Some(next_rt);
        //Timer::at(t_until).await;
        //implementar función handle de asyncInput que duerma hasta el siguiente evento de transición del modelo
        core::future::pending::<()>().await; //<()> significa que no devuelve nada
    }
}
