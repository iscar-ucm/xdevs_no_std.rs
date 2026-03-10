use crate::traits::{
    AbstractSimulator, MapInput, MapOutput, RtEngineInputChannel, RtEngineOutputChannel,
};
use crate::{Duration, Instant};

/// Automated simulation engine for real-time execution of DEVS models.
/// Its interfaces are created through the use of the `rt_engine` macro.
pub struct RtEngine<M: AbstractSimulator>
where
    M::Input: MapInput,
    M::Output: MapOutput,
{
    simulator: crate::Simulator<M>,
    input_channel: <M::Input as MapInput>::InputChannel,
    output_channel: <M::Output as MapOutput>::OutputChannel,
}

impl<M: AbstractSimulator> RtEngine<M>
where
    M::Input: MapInput,
    M::Output: MapOutput,
{
    pub fn new(
        model: M,
        input_channel: <M::Input as MapInput>::InputChannel,
        output_channel: <M::Output as MapOutput>::OutputChannel,
    ) -> Self {
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
                unsafe { output.map_output(&mut self.output_channel) };
            })
            .await;
    }
}

/// Specialized implementation: Only exists if IC is a &'static Channel.
/// Note how I and N are declared here, not on the struct.
impl<M: AbstractSimulator> RtEngine<M>
where
    M::Input: MapInput,
    M::Output: MapOutput,
    <M::Input as MapInput>::InputChannel: RtEngineInputChannel,
{
    pub fn sender(&self) -> <<M::Input as MapInput>::InputChannel as RtEngineInputChannel>::Sender {
        self.input_channel.sender()
    }
}

/// Specialized implementation: Only exists if OC is a &'static PubSubChannel.
/// Note how O, CAP, and SUBS are declared here.
impl<M: AbstractSimulator> RtEngine<M>
where
    M::Input: MapInput,
    M::Output: MapOutput,
    <M::Output as MapOutput>::OutputChannel: RtEngineOutputChannel,
{
    pub fn subscriber(
        &self,
    ) -> Result<
        <<M::Output as MapOutput>::OutputChannel as RtEngineOutputChannel>::Subscriber,
        crate::SubscribeError,
    > {
        self.output_channel.subscriber()
    }
}

struct RtEngineInputHandler<'a, M: AbstractSimulator>
where
    M::Input: MapInput,
{
    input_channel: &'a <M::Input as MapInput>::InputChannel,
    last_rt: Option<crate::Instant>,
}

impl<'a, M: AbstractSimulator> RtEngineInputHandler<'a, M>
where
    M::Input: MapInput,
{
    fn new(input_channel: &'a <M::Input as MapInput>::InputChannel) -> Self {
        Self {
            input_channel,
            last_rt: None,
        }
    }
}

impl<'a, M: AbstractSimulator> crate::traits::AsyncInput for RtEngineInputHandler<'a, M>
where
    M::Input: MapInput,
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
            unsafe { input.map_input(self.input_channel) }.await;
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
