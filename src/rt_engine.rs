use core::future::Future;

pub use crate::export::{RecvError, SubscribeError};
use crate::{
    port::Bag,
    simulation::{AbstractSimulator, AsyncInput, Simulable},
    Component, Duration, Instant,
};
use sealed::Sealed;

/// Automated simulation engine for real-time execution of DEVS models.
/// Its interfaces are created through the use of the `rt_engine` macro.
pub struct RtEngine<K, M>
where
    M: Component<Kind = K> + Simulable<K>,
    M::Input: InjectInput,
    M::Output: EjectOutput,
{
    simulator: <M as Simulable<K>>::Simulator,
    input_channel: <M::Input as InjectInput>::InputChannel,
    output_channel: <M::Output as EjectOutput>::OutputChannel,
}

impl<K, M> RtEngine<K, M>
where
    M: Component<Kind = K> + Simulable<K>,
    M::Input: InjectInput,
    M::Output: EjectOutput,
{
    pub fn new(
        model: M,
        input_channel: <M::Input as InjectInput>::InputChannel,
        output_channel: <M::Output as EjectOutput>::OutputChannel,
    ) -> Self {
        Self {
            simulator: model.to_simulator(),
            input_channel,
            output_channel,
        }
    }

    pub async fn simulate_rt_async(&mut self, config: &crate::Config) {
        let input_handler = RtEngineInputHandler::<K, M>::new(&mut self.input_channel);
        self.simulator
            .simulate_rt_async(config, input_handler, |output| {
                output.map_output(&self.output_channel);
            })
            .await;
    }
}

/// Specialized implementation: Only exists if IC is RtEngineInputChannel.
impl<K, M> RtEngine<K, M>
where
    M: Component<Kind = K> + Simulable<K>,
    M::Input: InjectInput,
    M::Output: EjectOutput,
    <M::Input as InjectInput>::InputChannel: RtEngineInputChannel,
{
    pub fn sender(
        &self,
    ) -> <<M::Input as InjectInput>::InputChannel as RtEngineInputChannel>::Sender {
        self.input_channel.sender()
    }
}

/// Specialized implementation: Only exists if OC is RtEngineOutputChannel.
impl<K, M> RtEngine<K, M>
where
    M: Component<Kind = K> + Simulable<K>,
    M::Input: InjectInput,
    M::Output: EjectOutput,
    <M::Output as EjectOutput>::OutputChannel: RtEngineOutputChannel,
{
    pub fn receiver(
        &self,
    ) -> Result<
        <<M::Output as EjectOutput>::OutputChannel as RtEngineOutputChannel>::Receiver,
        crate::rt_engine::SubscribeError,
    > {
        self.output_channel.receiver()
    }
}

struct RtEngineInputHandler<'a, K, M>
where
    M: Component<Kind = K> + Simulable<K>,
    M::Input: InjectInput,
{
    input_channel: &'a mut <M::Input as InjectInput>::InputChannel,
    last_rt: Option<crate::Instant>,
}

impl<'a, K, M> RtEngineInputHandler<'a, K, M>
where
    M: Component<Kind = K> + Simulable<K>,
    M::Input: InjectInput,
{
    fn new(input_channel: &'a mut <M::Input as InjectInput>::InputChannel) -> Self {
        Self {
            input_channel,
            last_rt: None,
        }
    }
}

impl<'a, K, M> AsyncInput for RtEngineInputHandler<'a, K, M>
where
    M: Component<Kind = K> + Simulable<K>,
    M::Input: InjectInput,
{
    type Input = M::Input;

    async fn handle(
        &mut self,
        config: &crate::Config,
        t_from: f64,
        t_until: f64,
        input: &mut Self::Input,
    ) -> f64 {
        let last_rt = self.last_rt.unwrap_or_else(Instant::now);
        let time_duration = (t_until - t_from) * config.time_scale;
        let time_duration = (time_duration * 1_000_000_000.0) as u64;
        let next_rt = last_rt + Duration::from_nanos(time_duration);

        let future = async {
            input.map_input(self.input_channel).await;
        };

        if embassy_time::with_deadline(next_rt, future).await.is_err() {
            // Deadline reached (timeout), check for jitter
            if let Some(max_jitter) = config.max_jitter {
                let jitter = Instant::now().duration_since(next_rt);
                let max_jitter_ticks = Duration::from_micros(max_jitter.as_micros() as u64);
                if jitter > max_jitter_ticks {
                    panic!("Jitter too high: {:?}", jitter);
                }
            }
            self.last_rt = Some(next_rt);
            t_until
        } else {
            let now = Instant::now();
            self.last_rt = Some(now);
            let elapsed_rt = now.duration_since(last_rt).as_micros() as f64 / 1_000_000.0;
            let elapsed_sim = elapsed_rt / config.time_scale;

            t_from + elapsed_sim
        }
    }
}

/// Input port interface for DEVS models that can be simulated in real-time using the `RtEngine`.
///
/// # Safety
///
/// This trait must be implemented via macros. Do not implement it manually.
pub unsafe trait InjectInput: Bag {
    /// Input channel for the rt_engine macro.
    type InputChannel;

    /// Maps the input enum to the corresponding input port
    fn map_input(&mut self, in_channel: &mut Self::InputChannel)
        -> impl Future<Output = ()> + Send;
}

/// Output port interface for DEVS models that can be simulated in real-time using the `RtEngine`.
///
/// # Safety
///
/// This trait must be implemented via macros. Do not implement it manually.
pub unsafe trait EjectOutput: Bag {
    /// Output channel for the rt_engine macro.
    type OutputChannel;

    /// Maps the output enum to the corresponding output port
    fn map_output(&self, out_channel: &Self::OutputChannel);
}

/// Input channel for the rt_engine macro.
pub trait RtEngineInputChannel: Sealed {
    /// Enum representing the input ports of the model. Each variant corresponds to an input port.
    type Input;
    /// Type of the sender used to send input events to the model.
    type Sender;

    /// Returns a sender to the channel. The sender can be used to send input events to the model.
    fn sender(&self) -> Self::Sender;

    fn recv(&mut self) -> impl Future<Output = Self::Input> + Send;
}

/// Output channel for the rt_engine macro.
pub trait RtEngineOutputChannel: Sealed {
    /// Enum representing the output ports of the model. Each variant corresponds to an output port.
    type Output;

    /// Type of the receiver used to receive output events from the model.
    type Receiver;

    /// Returns a subscriber to the channel. The subscriber can be used to receive output events from the model.
    fn receiver(&self) -> Result<Self::Receiver, crate::rt_engine::SubscribeError>;

    /// Publishes output events from the model to the channel, mapping the model's output ports to the channel's output events.
    fn publish(&self, output: Self::Output);
}

unsafe impl InjectInput for () {
    type InputChannel = ();

    #[inline(always)]
    fn map_input(
        &mut self,
        _in_channel: &mut Self::InputChannel,
    ) -> impl Future<Output = ()> + Send {
        core::future::pending()
    }
}

unsafe impl EjectOutput for () {
    type OutputChannel = ();

    #[inline(always)]
    fn map_output(&self, _out_channel: &Self::OutputChannel) {}
}

pub(crate) mod sealed {
    /// Trait used to prevent users from implementing certain traits manually.
    pub trait Sealed {}
}
