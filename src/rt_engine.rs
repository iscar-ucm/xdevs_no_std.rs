use core::future::Future;

pub use crate::export::{RecvError, SubscribeError};
use crate::{
    bag::Bag,
    simulation::{AbstractSimulator, AsyncInput, Clock, Simulable},
    Component,
};
use sealed::Sealed;

/// Automated simulation engine for real-time execution of DEVS models.
/// Its interfaces are created through the use of the `rt_engine` macro.
pub struct RtEngine<K, M, IC, OC>
where
    M: Component<Kind = K> + Simulable<K>,
{
    simulator: <M as Simulable<K>>::Simulator,
    input_channel: IC,
    output_channel: OC,
}

impl<K, M, IC, OC> RtEngine<K, M, IC, OC>
where
    M: Component<Kind = K> + Simulable<K>,
    IC: RtEngineInputChannel<Input = <M::Input as Bag>::Value>,
    OC: RtEngineOutputChannel<Output = <M::Output as Bag>::Value>,
{
    pub fn new(model: M, input_channel: IC, output_channel: OC) -> Self {
        Self {
            simulator: model.to_simulator(),
            input_channel,
            output_channel,
        }
    }

    pub async fn simulate_rt(
        &mut self,
        duration: crate::Duration,
        config: crate::RtConfig<impl Clock>,
    ) {
        let input_handler = RtEngineInputHandler::<M, IC>::new(&mut self.input_channel);
        self.simulator
            .simulate_rt(duration, config, input_handler, |output| {
                crate::bag::propagate(output, |event| self.output_channel.publish(event));
            })
            .await;
    }

    pub fn sender(&self) -> IC::Sender {
        self.input_channel.sender()
    }

    pub fn receiver(&self) -> Result<OC::Receiver, crate::rt_engine::SubscribeError> {
        self.output_channel.receiver()
    }
}

struct RtEngineInputHandler<'a, M, IC> {
    input_channel: &'a mut IC,
    marker: core::marker::PhantomData<fn() -> M>,
}

impl<'a, M, IC> RtEngineInputHandler<'a, M, IC> {
    fn new(input_channel: &'a mut IC) -> Self {
        Self {
            input_channel,
            marker: core::marker::PhantomData,
        }
    }
}

impl<'a, M, IC> AsyncInput for RtEngineInputHandler<'a, M, IC>
where
    M: Component,
    IC: RtEngineInputChannel<Input = <M::Input as Bag>::Value>,
{
    type Input = M::Input;

    async fn handle(&mut self, input: &mut Self::Input) {
        let event = self.input_channel.recv().await;
        // TODO determine how to handle the case where the input bag is full. For now, we just ignore the error.
        let _ = input.add_value(event);
    }
}

/// Input channel for the rt_engine macro.
pub trait RtEngineInputChannel: Sealed {
    /// Value received from the channel and injected into the model.
    type Input;
    /// Type of the sender used to send input events to the model.
    type Sender;

    /// Returns a sender to the channel. The sender can be used to send input events to the model.
    fn sender(&self) -> Self::Sender;

    fn recv(&mut self) -> impl Future<Output = Self::Input> + Send;
}

/// Output channel for the rt_engine macro.
pub trait RtEngineOutputChannel: Sealed {
    /// Value published by the model through the channel.
    type Output;

    /// Type of the receiver used to receive output events from the model.
    type Receiver;

    /// Returns a subscriber to the channel. The subscriber can be used to receive output events from the model.
    fn receiver(&self) -> Result<Self::Receiver, crate::rt_engine::SubscribeError>;

    /// Publishes output events from the model to the channel, mapping the model's output ports to the channel's output events.
    fn publish(&self, output: Self::Output);
}

impl RtEngineInputChannel for () {
    type Input = ();
    type Sender = ();

    fn sender(&self) {}

    fn recv(&mut self) -> impl Future<Output = ()> + Send {
        core::future::pending()
    }
}

impl RtEngineOutputChannel for () {
    type Output = ();
    type Receiver = ();

    fn receiver(&self) -> Result<Self::Receiver, crate::rt_engine::SubscribeError> {
        Ok(())
    }

    fn publish(&self, _output: Self::Output) {}
}

impl Sealed for () {}

pub(crate) mod sealed {
    /// Trait used to prevent users from implementing certain traits manually.
    pub trait Sealed {}
}

#[cfg(test)]
mod tests {
    use super::RecvError;
    use crate::{export::Clock, prelude::*, Duration};

    pub struct Echo {
        sigma: Duration,
        pending: Option<usize>,
    }

    impl Echo {
        fn new() -> Self {
            Self {
                sigma: Duration::MAX,
                pending: None,
            }
        }
    }

    #[crate::rt_engine(in_channel_size = 1, out_channel_size = 1)]
    impl Component for Echo {
        type Kind = AtomicKind;
        type Input = Port<usize, 1>;
        type Output = Port<usize, 1>;
    }

    impl Atomic for Echo {
        fn delta_int(&mut self) {
            self.sigma = Duration::MAX;
        }

        fn lambda(&self, output: &mut Self::Output) {
            if let Some(value) = self.pending {
                output.add_value(value).unwrap();
            }
        }

        fn ta(&self) -> Duration {
            self.sigma
        }

        fn delta_ext(&mut self, _elapsed: Duration, input: &Self::Input) {
            if let Some(&value) = input.last() {
                self.pending = Some(value);
                self.sigma = Duration::ZERO;
            }
        }
    }

    #[tokio::test]
    async fn rt_engine_delivers_input_and_publishes_output() {
        let mut engine = Echo::new().into_rt_engine();
        let sender = engine.sender();
        let mut receiver = engine.receiver().unwrap();

        sender.send(7).await.unwrap();

        tokio::time::timeout(
            core::time::Duration::from_secs(2),
            engine.simulate_rt(
                Duration::from_millis(20),
                RtConfig::new(Clock::new(), 1, None),
            ),
        )
        .await
        .expect("simulation timed out");

        let output = tokio::time::timeout(core::time::Duration::from_secs(1), receiver.recv())
            .await
            .expect("no output published");

        assert_eq!(output.unwrap(), 7);
    }

    pub struct Burst {
        next: usize,
        sigma: Duration,
    }

    impl Burst {
        fn new() -> Self {
            Self {
                next: 1,
                sigma: Duration::ZERO,
            }
        }
    }

    #[crate::rt_engine(in_channel_size = 1, out_channel_size = 1)]
    impl Component for Burst {
        type Kind = AtomicKind;
        type Input = ();
        type Output = Port<usize, 1>;
    }

    impl Atomic for Burst {
        fn delta_int(&mut self) {
            if self.next < 3 {
                self.next += 1;
                self.sigma = Duration::ZERO;
            } else {
                self.sigma = Duration::MAX;
            }
        }

        fn lambda(&self, output: &mut Self::Output) {
            output.add_value(self.next).unwrap();
        }

        fn ta(&self) -> Duration {
            self.sigma
        }

        fn delta_ext(&mut self, _elapsed: Duration, _input: &Self::Input) {}
    }

    #[tokio::test]
    async fn rt_engine_reports_lagged_outputs_when_receiver_not_polled() {
        let mut engine = Burst::new().into_rt_engine();
        let mut receiver = engine.receiver().unwrap();

        tokio::time::timeout(
            core::time::Duration::from_secs(2),
            engine.simulate_rt(
                Duration::from_millis(20),
                RtConfig::new(Clock::new(), 1, None),
            ),
        )
        .await
        .expect("simulation timed out");

        let first = tokio::time::timeout(core::time::Duration::from_secs(1), receiver.recv())
            .await
            .expect("no lag reported");

        match first {
            Err(RecvError::Lagged(missed)) => assert_eq!(missed, 2),
            _ => panic!("expected a lagged receiver"),
        }

        let second = tokio::time::timeout(core::time::Duration::from_secs(1), receiver.recv())
            .await
            .expect("no retained output");

        assert_eq!(second.unwrap(), 3);
    }
}
