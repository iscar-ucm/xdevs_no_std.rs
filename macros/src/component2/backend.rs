#[cfg(feature = "embassy-backend")]
mod embassy;

#[cfg(feature = "embassy-backend")]
pub(crate) use embassy::*;

#[cfg(feature = "std-backend")]
mod tokio;

#[cfg(feature = "std-backend")]
pub(crate) use tokio::*;
