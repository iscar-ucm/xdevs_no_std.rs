#[cfg(feature = "embassy-backend")]
mod embassy;

#[cfg(feature = "embassy-backend")]
pub use embassy::RtEngineBackend as RtEngine;

#[cfg(feature = "std-backend")]
mod tokio;

#[cfg(feature = "std-backend")]
pub use tokio::RtEngineBackend as RtEngine;

#[cfg(not(any(feature = "embassy-backend", feature = "std-backend")))]
mod no_backend;

#[cfg(not(any(feature = "embassy-backend", feature = "std-backend")))]
pub use no_backend::RtEngineBackend as RtEngine;

use proc_macro2::TokenStream as TokenStream2;
use syn::{
    parse::{Parse, ParseStream},
    Error, Result, Token,
};

use crate::component::{combine_err, Component};

/// Generated token fragments used to construct backend channel code.
pub struct ChannelTokens {
    pub channel_type: TokenStream2,
    pub channel_call: TokenStream2,
    pub private_channel: TokenStream2,
}

/// Generic parsed arguments for `rt_engine(...)` backend configuration.
#[derive(Default, Debug)] // Added Debug here too!
pub struct RtEngineArgs {
    pub in_channel_size: Option<usize>,
    pub out_channel_size: Option<usize>,
    pub max_out_subs: Option<usize>,
}

impl Parse for RtEngineArgs {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut args = RtEngineArgs::default();
        let mut acc: Option<Error> = None;

        // Parse built-in Meta items
        let parsed_args =
            syn::punctuated::Punctuated::<syn::Meta, Token![,]>::parse_terminated(input)?;

        for meta in parsed_args {
            // We only care about `path = value` (MetaNameValue)
            let nv = match meta {
                syn::Meta::NameValue(nv) => nv,
                _ => {
                    let err = Error::new_spanned(meta, "expected `name = value` format");
                    combine_err(&mut acc, err);
                    continue;
                }
            };

            let name = nv
                .path
                .require_ident()
                .map(|i| i.to_string())
                .unwrap_or_default();

            // Extract the LitInt from the Expr
            let lit_int = match &nv.value {
                syn::Expr::Lit(syn::ExprLit {
                    lit: syn::Lit::Int(lit),
                    ..
                }) => lit,
                _ => {
                    let err = Error::new_spanned(&nv.value, "expected an integer literal");
                    combine_err(&mut acc, err);
                    continue;
                }
            };

            let value = match lit_int.base10_parse::<usize>() {
                Ok(v) => v,
                Err(e) => {
                    let err = Error::new_spanned(lit_int, format!("invalid integer: {}", e));
                    combine_err(&mut acc, err);
                    continue;
                }
            };

            match name.as_str() {
                "in_channel_size" => {
                    if args.in_channel_size.is_some() {
                        let err =
                            Error::new_spanned(&nv.path, "duplicate argument: in_channel_size");
                        combine_err(&mut acc, err);
                    }
                    args.in_channel_size = Some(value);
                }
                "out_channel_size" => {
                    if args.out_channel_size.is_some() {
                        let err =
                            Error::new_spanned(&nv.path, "duplicate argument: out_channel_size");
                        combine_err(&mut acc, err);
                    }
                    args.out_channel_size = Some(value);
                }
                "max_out_subs" => {
                    if args.max_out_subs.is_some() {
                        let err = Error::new_spanned(&nv.path, "duplicate argument: max_out_subs");
                        combine_err(&mut acc, err);
                    }
                    args.max_out_subs = Some(value);
                }
                _ => {
                    let err =
                        Error::new_spanned(&nv.path, format!("unknown rt_engine argument: {name}"));
                    combine_err(&mut acc, err);
                }
            }
        }

        if let Some(err) = acc {
            return Err(err);
        }

        Ok(args)
    }
}

pub trait Backend {
    fn check_compatibility(&self, model: &Component) -> Result<()>;
    fn input_channel(&self, model: &Component) -> ChannelTokens;
    fn output_channel(&self, model: &Component) -> ChannelTokens;
}
