#[cfg(feature = "embassy-backend")]
mod embassy;

#[cfg(feature = "embassy-backend")]
pub use embassy::RtEngineBackend;

#[cfg(feature = "std-backend")]
mod tokio;

#[cfg(feature = "std-backend")]
pub use tokio::RtEngineBackend;

#[cfg(not(any(feature = "embassy-backend", feature = "std-backend")))]
mod no_backend;

#[cfg(not(any(feature = "embassy-backend", feature = "std-backend")))]
pub use no_backend::RtEngineBackend;

use super::{ChannelTokens, RtEngineArgs};
use syn::{Ident, ItemImpl, MetaNameValue, Result};

pub trait Backend {
    fn check_arg_compatibility(arg: &MetaNameValue) -> Result<()>;
    fn check_item_compatibility(item: &ItemImpl) -> Result<()>;
    fn input_channel(args: &RtEngineArgs, model_ident: &Ident) -> ChannelTokens;
    fn output_channel(args: &RtEngineArgs, model_ident: &Ident) -> ChannelTokens;
}
