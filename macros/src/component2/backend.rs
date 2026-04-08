#[cfg(feature = "embassy-backend")]
mod embassy;

#[cfg(feature = "embassy-backend")]
pub use embassy::RtEngineBackend as RtEngine;

#[cfg(feature = "std-backend")]
mod tokio;

#[cfg(feature = "std-backend")]
pub use tokio::RtEngineBackend as RtEngine;

use proc_macro2::TokenStream as TokenStream2;
use syn::{
    parse::{Parse, ParseStream},
    Error, Ident, LitInt, Result, Token,
};

use crate::component2::CommonComponent;

/// Generated token fragments used to construct backend channel code.
pub struct ChannelTokens {
    pub channel_type: TokenStream2,
    pub channel_call: TokenStream2,
    pub private_channel: TokenStream2,
}

/// Generic parsed arguments for `rt_engine = { ... }` backend configuration.
pub struct RtEngineArgs {
    pub in_channel_size: Option<usize>,
    pub out_channel_size: Option<usize>,
    pub max_out_subs: Option<usize>,
}

impl Parse for RtEngineArgs {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut in_channel_size = None;
        let mut out_channel_size = None;
        let mut max_out_subs = None;

        while !input.is_empty() {
            let ident: Ident = input.parse()?;
            input.parse::<Token![=]>()?;
            let value: LitInt = input.parse()?;
            let value: usize = value.base10_parse()?;

            match ident.to_string().as_str() {
                "in_channel_size" => {
                    if in_channel_size.is_some() {
                        return Err(Error::new(
                            ident.span(),
                            "duplicate argument: in_channel_size",
                        ));
                    }
                    in_channel_size = Some(value);
                }
                "out_channel_size" => {
                    if out_channel_size.is_some() {
                        return Err(Error::new(
                            ident.span(),
                            "duplicate argument: out_channel_size",
                        ));
                    }
                    out_channel_size = Some(value);
                }
                "max_out_subs" => {
                    if max_out_subs.is_some() {
                        return Err(Error::new(ident.span(), "duplicate argument: max_out_subs"));
                    }
                    max_out_subs = Some(value);
                }
                str => {
                    return Err(Error::new(
                        ident.span(),
                        format!("unknown rt_engine argument: {str}"),
                    ));
                }
            }

            // Optional trailing comma
            if !input.is_empty() {
                input.parse::<Token![,]>()?;
            }
        }

        Ok(Self {
            in_channel_size,
            out_channel_size,
            max_out_subs,
        })
    }
}

pub trait Backend {
    fn check_compatibility(&self, model: &CommonComponent) -> Result<()>;
    fn input_channel(&self, model: &CommonComponent) -> ChannelTokens;
    fn output_channel(&self, model: &CommonComponent) -> ChannelTokens;
}
