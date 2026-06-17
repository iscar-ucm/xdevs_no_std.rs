use super::{Backend, ChannelTokens, RtEngineArgs};
use proc_macro2::TokenStream as TokenStream2;
use syn::{Ident, MetaNameValue, Result};

/// Placeholder backend used when no backend feature is enabled.
#[derive(Debug, Clone)]
pub struct RtEngineBackend;

impl Backend for RtEngineBackend {
    fn check_arg_compatibility(_arg: &MetaNameValue) -> Result<()> {
        // This is ok because the error will be reported at the beginning of the macro expansion
        Ok(())
    }

    fn check_item_compatibility(_item: &syn::ItemImpl) -> Result<()> {
        // This is ok because the error will be reported at the beginning of the macro expansion
        Ok(())
    }

    fn input_channel(_args: &RtEngineArgs, _model_ident: &Ident) -> ChannelTokens {
        ChannelTokens {
            channel_type: quote::quote! { () },
            channel_call: quote::quote! { () },
            private_channel: TokenStream2::new(),
        }
    }

    fn output_channel(_args: &RtEngineArgs, _model_ident: &Ident) -> ChannelTokens {
        ChannelTokens {
            channel_type: quote::quote! { () },
            channel_call: quote::quote! { () },
            private_channel: TokenStream2::new(),
        }
    }
}
