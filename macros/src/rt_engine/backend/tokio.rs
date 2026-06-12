use super::{Backend, ChannelTokens, RtEngineArgs};
use proc_macro2::TokenStream as TokenStream2;
use syn::{Ident, ItemImpl, Result};

/// Arguments for the `#[rt_engine]` attribute macro.
#[derive(Debug, Clone)]
pub struct RtEngineBackend;

impl Backend for RtEngineBackend {
    fn check_args_compatibility(max_out_subs: Option<usize>) -> Result<()> {
        match max_out_subs {
            Some(_) => Err(syn::Error::new(
                proc_macro2::Span::call_site(),
                "max_out_subs is not supported in the std backend",
            )),
            None => Ok(()),
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
