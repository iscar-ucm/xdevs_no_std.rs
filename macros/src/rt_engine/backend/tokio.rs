use super::{Backend, ChannelTokens, RtEngineArgs};
use proc_macro2::TokenStream as TokenStream2;
use syn::{Ident, ItemImpl, MetaNameValue, Result};

/// Arguments for the `#[rt_engine]` attribute macro.
#[derive(Debug, Clone)]
pub struct RtEngineBackend;

impl Backend for RtEngineBackend {
    fn check_arg_compatibility(arg: &MetaNameValue) -> Result<()> {
        let path = &arg.path;
        let name = path
            .require_ident()
            .map(|i| i.to_string())
            .unwrap_or_default();
        // Only max_out_subs is not supported specifically in the std backend,
        // other incompatible arguments are handled in the main macro code.
        match name.as_str() {
            "max_out_subs" => Err(syn::Error::new_spanned(
                path,
                "max_out_subs is not supported in the std backend",
            )),
            _ => Ok(()),
        }
    }
    fn check_item_compatibility(_: &ItemImpl) -> Result<()> {
        Ok(())
    }

    fn input_channel(args: &RtEngineArgs, _model_ident: &Ident) -> ChannelTokens {
        let in_channel_size = args.in_channel_size;
        let channel_type = quote::quote! { ::xdevs::export::InputChannel<
            <Self as ::xdevs::port::BagMux>::Mux,
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

    fn output_channel(args: &RtEngineArgs, _model_ident: &Ident) -> ChannelTokens {
        let out_channel_size = args.out_channel_size;
        let channel_type = quote::quote! { ::xdevs::export::OutputChannel<
            <Self as ::xdevs::port::BagMux>::Mux,
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
