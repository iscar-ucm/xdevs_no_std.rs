use crate::traits::{
    AbstractSimulator, RtEngineInputChannel, RtEngineOutputChannel, RtEngineWrapper,
};
use crate::{Duration, Instant};

/// Aliases for the RtEngineWrapper types, to avoid having to write the full path everywhere.
pub type InputEnum<M> = <<M as RtEngineWrapper>::InputChannel as RtEngineInputChannel>::InputEnum;
pub type OutputEnum<M> =
    <<M as RtEngineWrapper>::OutputChannel as RtEngineOutputChannel>::OutputEnum;
pub type Sender<M> = <<M as RtEngineWrapper>::InputChannel as RtEngineInputChannel>::Sender;
pub type Subscriber<M> =
    <<M as RtEngineWrapper>::OutputChannel as RtEngineOutputChannel>::Subscriber;

/// Automated simulation engine for real-time execution of DEVS models.
/// Its interfaces are created through the use of the `rt_engine` macro.
pub struct RtEngine<M: RtEngineWrapper> {
    simulator: crate::Simulator<M>,
    input_channel: M::InputChannel,
    output_channel: M::OutputChannel,
}

impl<M: RtEngineWrapper> RtEngine<M> {
    pub fn new(model: M, input_channel: M::InputChannel, output_channel: M::OutputChannel) -> Self {
        Self {
            simulator: crate::Simulator::new(model),
            input_channel,
            output_channel,
        }
    }

    pub async fn simulate_rt_async(&mut self, config: &crate::Config) {
        let input_handler = RtEngineInputHandler::<M>::new(&self.input_channel);
        self.simulator
            .simulate_rt_async(config, input_handler, |output| {
                M::map_output(output, &mut self.output_channel);
            })
            .await;
    }
}

/// Specialized implementation: Only exists if IC is a &'static Channel.
/// Note how I and N are declared here, not on the struct.
impl<M: RtEngineWrapper> RtEngine<M>
where
    M::InputChannel: RtEngineInputChannel,
{
    pub fn sender(&self) -> <M::InputChannel as RtEngineInputChannel>::Sender {
        self.input_channel.sender()
    }
}

/// Specialized implementation: Only exists if OC is a &'static PubSubChannel.
/// Note how O, CAP, and SUBS are declared here.
impl<M: RtEngineWrapper> RtEngine<M>
where
    M::OutputChannel: RtEngineOutputChannel,
{
    pub fn subscriber(
        &self,
    ) -> Result<<M::OutputChannel as RtEngineOutputChannel>::Subscriber, crate::SubscribeError>
    {
        self.output_channel.subscriber()
    }
}

struct RtEngineInputHandler<'a, M: RtEngineWrapper + AbstractSimulator> {
    input_channel: &'a M::InputChannel,
    last_rt: Option<crate::Instant>,
}

impl<'a, M: RtEngineWrapper + AbstractSimulator> RtEngineInputHandler<'a, M> {
    fn new(input_channel: &'a M::InputChannel) -> Self {
        Self {
            input_channel,
            last_rt: None,
        }
    }
}

impl<'a, M: AbstractSimulator + RtEngineWrapper> crate::traits::AsyncInput
    for RtEngineInputHandler<'a, M>
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
            M::map_input(self.input_channel, input).await;
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
