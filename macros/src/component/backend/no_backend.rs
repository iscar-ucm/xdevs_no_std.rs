use super::{Backend, ChannelTokens, RtEngineArgs};
use crate::component::{port::Ports, Component, ComponentArgs};
use proc_macro2::TokenStream as TokenStream2;
use syn::{
    parse::{Parse, ParseStream},
    Error, Result,
};

/// Placeholder backend used when no backend feature is enabled.
#[derive(Debug, Clone)]
pub struct RtEngineBackend;

impl Default for RtEngineBackend {
    fn default() -> Self {
        Self
    }
}

impl Parse for RtEngineBackend {
    fn parse(input: ParseStream) -> Result<Self> {
        // Keep argument parsing behavior consistent (unknown/duplicate args still error).
        let parsed_args: RtEngineArgs = input.parse()?;
        let _ = (
            parsed_args.in_channel_size,
            parsed_args.out_channel_size,
            parsed_args.max_out_subs,
        );
        Ok(Self)
    }
}

impl Backend for RtEngineBackend {
    fn check_compatibility(
        &self,
        args: &ComponentArgs,
        _input: &Ports,
        _output: &Ports,
    ) -> Result<()> {
        let span = args
            .rt_engine_span
            .unwrap_or_else(|| proc_macro2::Span::call_site());
        Err(Error::new(
            span,
            "rt_engine requires enabling one backend feature: `std-backend` or `embassy-backend`",
        ))
    }

    fn input_channel(&self, _model: &Component) -> ChannelTokens {
        ChannelTokens {
            channel_type: quote::quote! { () },
            channel_call: quote::quote! { () },
            private_channel: TokenStream2::new(),
        }
    }

    fn output_channel(&self, _model: &Component) -> ChannelTokens {
        ChannelTokens {
            channel_type: quote::quote! { () },
            channel_call: quote::quote! { () },
            private_channel: TokenStream2::new(),
        }
    }
}
