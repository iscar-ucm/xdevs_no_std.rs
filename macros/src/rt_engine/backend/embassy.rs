use super::{Backend, ChannelTokens, RtEngineArgs};
use heck::ToShoutySnakeCase;
use syn::{Ident, ItemImpl, MetaNameValue, Result};

/// Arguments for the `#[rt_engine]` attribute macro.
#[derive(Debug, Clone)]
pub struct RtEngineBackend;

impl Backend for RtEngineBackend {
    fn check_arg_compatibility(_arg: &MetaNameValue) -> Result<()> {
        Ok(())
    }
    fn check_item_compatibility(item: &ItemImpl) -> Result<()> {
        if item.generics.params.len() > 0 {
            return Err(syn::Error::new_spanned(
                &item.generics,
                "The `#[rt_engine]` macro embassy backend does not support generic parameters.",
            ));
        } else {
            Ok(())
        }
    }
    fn input_channel(args: &RtEngineArgs, model_ident: &Ident) -> ChannelTokens {
        let input_ident = quote::format_ident!("{}Input", model_ident);
        let in_channel_size = args.in_channel_size;

        let channel_type = quote::quote! { ::xdevs::export::InputChannel<'static,
            <Self as ::xdevs::port::BagMux>::Mux,
            #in_channel_size
        > };
        let upper_name = model_ident.to_string().to_shouty_snake_case();
        let channel_ident = quote::format_ident!("{}_IN_CHANNEL", upper_name);
        let channel_call = quote::quote! {::xdevs::export::InputChannel::new(&#channel_ident) };

        let private_channel = quote::quote! {
            /// Auto-generated static input channel.
            pub static #channel_ident: ::xdevs::export::Channel<
                <#input_ident as ::xdevs::port::BagMux>::Mux,
                #in_channel_size
            > = ::xdevs::export::Channel::new();
        };

        ChannelTokens {
            channel_type,
            channel_call,
            private_channel,
        }
    }

    fn output_channel(args: &RtEngineArgs, model_ident: &Ident) -> ChannelTokens {
        let output_ident = quote::format_ident!("{}Output", model_ident);
        let out_channel_size = args.out_channel_size;
        let max_out_subs = args.max_out_subs;

        let channel_type = quote::quote! { ::xdevs::export::OutputChannel<'static,
            <Self as ::xdevs::port::BagMux>::Mux,
            #out_channel_size,
            #max_out_subs
        > };
        let upper_name = model_ident.to_string().to_shouty_snake_case();
        let channel_ident = quote::format_ident!("{}_OUT_CHANNEL", upper_name);
        let channel_call = quote::quote! {::xdevs::export::OutputChannel::new(&#channel_ident) };

        let private_channel = quote::quote! {
            /// Auto-generated static output PubSub channel.
            pub static #channel_ident: ::xdevs::export::PubSubChannel<
                <#output_ident as ::xdevs::port::BagMux>::Mux,
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
