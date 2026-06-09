use super::{Backend, ChannelTokens, RtEngineArgs};
use crate::coupled::ComponentArgs;
use heck::ToShoutySnakeCase;
use syn::{
    parse::{Parse, ParseStream},
    ItemStruct, Result,
};

/// Arguments for the `#[rt_engine]` attribute macro.
#[derive(Debug, Clone)]
pub struct RtEngineBackend {
    /// Capacity of the input channel (`in_channel_size = ...`).
    in_channel_size: usize,
    /// Capacity of the output channel (`out_channel_size = ...`).
    out_channel_size: usize,
    /// Number of subscribers for the output PubSubChannel (`max_out_subs = ...`).
    max_out_subs: usize,
}

impl Default for RtEngineBackend {
    fn default() -> Self {
        Self {
            in_channel_size: 1,
            out_channel_size: 1,
            max_out_subs: 1,
        }
    }
}

impl Parse for RtEngineBackend {
    fn parse(input: ParseStream) -> Result<Self> {
        let parsed_args: RtEngineArgs = input.parse()?;

        Ok(RtEngineBackend {
            in_channel_size: parsed_args.in_channel_size.unwrap_or(1),
            out_channel_size: parsed_args.out_channel_size.unwrap_or(1),
            max_out_subs: parsed_args.max_out_subs.unwrap_or(1),
        })
    }
}

impl Backend for RtEngineBackend {
    fn check_compatibility(&self, _args: &ComponentArgs) -> Result<()> {
        Ok(())
    }

    fn input_channel(&self, model: &ItemStruct) -> ChannelTokens {
        let model_ident = &model.ident;
        let input_ident = quote::format_ident!("{}Input", model_ident);
        let in_channel_size = self.in_channel_size;

        let channel_type = quote::quote! { ::xdevs::export::InputChannel<'static,
            <Self as ::xdevs::traits::BagMux>::Mux,
            #in_channel_size
        > };
        let upper_name = model_ident.to_string().to_shouty_snake_case();
        let channel_ident = quote::format_ident!("{}_IN_CHANNEL", upper_name);
        let channel_call = quote::quote! {::xdevs::export::InputChannel::new(&#channel_ident) };

        let private_channel = quote::quote! {
            /// Auto-generated static input channel.
            pub static #channel_ident: ::xdevs::export::Channel<
                <#input_ident as ::xdevs::traits::BagMux>::Mux,
                #in_channel_size
            > = ::xdevs::export::Channel::new();
        };

        ChannelTokens {
            channel_type,
            channel_call,
            private_channel,
        }
    }

    fn output_channel(&self, model: &ItemStruct) -> ChannelTokens {
        let model_ident = &model.ident;
        let output_ident = quote::format_ident!("{}Output", model_ident);
        let out_channel_size = self.out_channel_size;
        let max_out_subs = self.max_out_subs;

        let channel_type = quote::quote! { ::xdevs::export::OutputChannel<'static,
            <Self as ::xdevs::traits::BagMux>::Mux,
            #out_channel_size,
            #max_out_subs
        > };
        let upper_name = model_ident.to_string().to_shouty_snake_case();
        let channel_ident = quote::format_ident!("{}_OUT_CHANNEL", upper_name);
        let channel_call = quote::quote! {::xdevs::export::OutputChannel::new(&#channel_ident) };

        let private_channel = quote::quote! {
            /// Auto-generated static output PubSub channel.
            pub static #channel_ident: ::xdevs::export::PubSubChannel<
                <#output_ident as ::xdevs::traits::BagMux>::Mux,
                #out_channel_size,
                #max_out_subs,
            > = ::xdevs::export::PubSubChannel::new();
        };

        ChannelTokens {
            channel_type,
            channel_call,
            private_channel,
        }
    }
}
