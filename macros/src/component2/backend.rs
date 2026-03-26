#[cfg(feature = "embassy-backend")]
mod embassy;

#[cfg(feature = "embassy-backend")]
pub use embassy::RtEngineBackend as RtEngine;

#[cfg(feature = "std-backend")]
mod tokio;

#[cfg(feature = "std-backend")]
pub use tokio::RtEngineBackend as RtEngine;

use proc_macro2::TokenStream as TokenStream2;

use crate::component2::CommonComponent;

pub trait Backend {
    fn check_compatibility(&self, model: &CommonComponent) -> Result<(), syn::Error>;
    fn input_channel(&self, model: &CommonComponent) -> (TokenStream2, TokenStream2, TokenStream2);
    fn output_channel(&self, model: &CommonComponent)
        -> (TokenStream2, TokenStream2, TokenStream2);
}
