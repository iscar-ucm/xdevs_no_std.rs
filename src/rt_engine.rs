pub use crate::export::{RecvError, SubscribeError};
use crate::traits::{
    AbstractSimulator, EjectOutput, InjectInput, RtEngineInputChannel, RtEngineOutputChannel,
};
use crate::{Duration, Instant, Simulator};

/// Automated simulation engine for real-time execution of DEVS models.
/// Its interfaces are created through the use of the `rt_engine` macro.
pub struct RtEngine<M: AbstractSimulator>
where
    M::Input: InjectInput,
    M::Output: EjectOutput,
{
    simulator: Simulator<M>,
    input_channel: <M::Input as InjectInput>::InputChannel,
    output_channel: <M::Output as EjectOutput>::OutputChannel,
}

impl<M: AbstractSimulator> RtEngine<M>
where
    M::Input: InjectInput,
    M::Output: EjectOutput,
{
    pub fn new(
        model: M,
        input_channel: <M::Input as InjectInput>::InputChannel,
        output_channel: <M::Output as EjectOutput>::OutputChannel,
    ) -> Self {
        Self {
            simulator: Simulator::new(model),
            input_channel,
            output_channel,
        }
    }

    pub async fn simulate_rt_async(&mut self, config: &crate::Config) {
        let input_handler = RtEngineInputHandler::<M>::new(&mut self.input_channel);
        self.simulator
            .simulate_rt_async(config, input_handler, |output| {
                output.map_output(&mut self.output_channel);
            })
            .await;
    }
}

/// Specialized implementation: Only exists if IC is a &'static Channel.
/// Note how I and N are declared here, not on the struct.
impl<M: AbstractSimulator> RtEngine<M>
where
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

/// Specialized implementation: Only exists if OC is a &'static PubSubChannel.
/// Note how O, CAP, and SUBS are declared here.
impl<M: AbstractSimulator> RtEngine<M>
where
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

struct RtEngineInputHandler<'a, M: AbstractSimulator>
where
    M::Input: InjectInput,
{
    input_channel: &'a mut <M::Input as InjectInput>::InputChannel,
    last_rt: Option<crate::Instant>,
}

impl<'a, M: AbstractSimulator> RtEngineInputHandler<'a, M>
where
    M::Input: InjectInput,
{
    fn new(input_channel: &'a mut <M::Input as InjectInput>::InputChannel) -> Self {
        Self {
            input_channel,
            last_rt: None,
        }
    }
}

impl<'a, M: AbstractSimulator> crate::traits::AsyncInput for RtEngineInputHandler<'a, M>
where
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
            input.map_input(&mut self.input_channel).await;
        };

        if let Err(_) = embassy_time::with_deadline(next_rt.into(), future).await {
            // Deadline reached (timeout), check for jitter
            if let Some(max_jitter) = config.max_jitter {
                let jitter = Instant::now().duration_since(next_rt);
                let max_jitter_ticks = Duration::from_micros(max_jitter.as_micros() as u64);
                if jitter > max_jitter_ticks {
                    panic!("Jitter too high: {:?}", jitter);
                }
            }
            self.last_rt = Some(next_rt);
            return t_until;
        } else {
            let now = Instant::now();
            self.last_rt = Some(now);
            let elapsed_rt = now.duration_since(last_rt).as_micros() as f64 / 1_000_000.0;
            let elapsed_sim = elapsed_rt / config.time_scale;

            return t_from + elapsed_sim;
        }
    }
}
