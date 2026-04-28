#[cfg(feature = "embassy")]
mod embassy;

#[cfg(feature = "embassy")]
pub use embassy::*;

#[cfg(feature = "std")]
mod tokio;

#[cfg(feature = "std")]
pub use tokio::*;
