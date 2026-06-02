use super::{Backend, ChannelTokens, RtEngineArgs};
use crate::component::{port::Ports, Component, ComponentArgs};
use proc_macro2::{Span, TokenStream as TokenStream2};
use syn::{
    parse::{Parse, ParseStream},
    Error, Result,
};

/// Arguments for the `#[rt_engine]` attribute macro.
#[derive(Debug, Clone)]
pub struct RtEngineBackend {
    /// Capacity of the input channel (`in_channel_size = ...`).
    in_channel_size: usize,
    /// Capacity of the output channel (`out_channel_size = ...`).
    out_channel_size: usize,
}

impl Default for RtEngineBackend {
    fn default() -> Self {
        Self {
            in_channel_size: 1,
            out_channel_size: 1,
        }
    }
}

impl Parse for RtEngineBackend {
    fn parse(input: ParseStream) -> Result<Self> {
        let parsed_args: RtEngineArgs = input.parse()?;

        if parsed_args.max_out_subs.is_some() {
            return Err(Error::new(
                Span::call_site(),
                "max_out_subs is not supported in the std backend",
            ));
        }

        Ok(RtEngineBackend {
            in_channel_size: parsed_args.in_channel_size.unwrap_or(1),
            out_channel_size: parsed_args.out_channel_size.unwrap_or(1),
        })
    }
}

impl Backend for RtEngineBackend {
    fn check_compatibility(&self, _: &ComponentArgs, _: &Ports, _: &Ports) -> Result<()> {
        Ok(())
    }

    fn input_channel(&self, _model: &Component) -> ChannelTokens {
        let in_channel_size = self.in_channel_size;
        let channel_type = quote::quote! { ::xdevs::export::InputChannel<
            <Self as ::xdevs::traits::BagMux>::Mux,
            #in_channel_size
        > };
        let channel_call = quote::quote! {::xdevs::export::InputChannel::new() };
        let private_channel = TokenStream2::new();
        ChannelTokens {
            channel_type,
            channel_call,
            private_channel,
        }
    }

    fn output_channel(&self, _model: &Component) -> ChannelTokens {
        let out_channel_size = self.out_channel_size;
        let channel_type = quote::quote! { ::xdevs::export::OutputChannel<
            <Self as ::xdevs::traits::BagMux>::Mux,
            #out_channel_size
        > };
        let channel_call = quote::quote! {::xdevs::export::OutputChannel::new() };
        let private_channel = TokenStream2::new();
        ChannelTokens {
            channel_type,
            channel_call,
            private_channel,
        }
    }
}
