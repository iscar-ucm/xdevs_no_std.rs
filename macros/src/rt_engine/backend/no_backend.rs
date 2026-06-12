use super::{Backend, ChannelTokens, RtEngineArgs};
use proc_macro2::TokenStream as TokenStream2;
use syn::{Error, Ident, Result};

/// Placeholder backend used when no backend feature is enabled.
#[derive(Debug, Clone)]
pub struct RtEngineBackend;

impl Backend for RtEngineBackend {
    fn check_args_compatibility(max_out_subs: Option<usize>) -> Result<()> {
        Err(Error::new_spanned(
            max_out_subs,
            "rt_engine requires enabling one backend feature: `std-backend` or `embassy-backend`",
        ))
    }

    fn check_item_compatibility(_item: &syn::ItemImpl) -> Result<()> {
        // This is ok because the error will be reported in check_args_compatibility
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
