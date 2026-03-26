use proc_macro2::TokenStream as TokenStream2;
use syn::{parse::ParseStream, Error, Ident, LitInt, Token};

use crate::component2::CommonComponent;

/// Arguments for the `#[rt_engine]` attribute macro.
///
/// Supported arguments:
/// - `in_size`: capacity of the input channel
/// - `out_size`: capacity of the output channel
pub struct RtEngineBackend {
    in_size: usize,
    out_size: usize,
}

impl Default for RtEngineBackend {
    fn default() -> Self {
        Self {
            in_size: 1,
            out_size: 1,
        }
    }
}

impl syn::parse::Parse for RtEngineBackend {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut in_size = None;
        let mut out_size = None;

        while !input.is_empty() {
            let ident: Ident = input.parse()?;
            input.parse::<Token![=]>()?;
            let value: LitInt = input.parse()?;
            let value: usize = value.base10_parse()?;

            match ident.to_string().as_str() {
                "in_size" => {
                    if let Some(_) = in_size {
                        return Err(Error::new(ident.span(), "duplicate argument: in_size"));
                    } else {
                        in_size = Some(value)
                    }
                }
                "out_size" => {
                    if let Some(_) = out_size {
                        return Err(Error::new(ident.span(), "duplicate argument: out_size"));
                    } else {
                        out_size = Some(value)
                    }
                }
                "max_out_subs" => {
                    return Err(syn::Error::new(
                        proc_macro2::Span::call_site(),
                        "max_out_subs is not supported in the std backend",
                    ))
                }
                str => {
                    return Err(Error::new(
                        proc_macro2::Span::call_site(),
                        format!("unknown top argument: {}", str),
                    ))
                }
            }

            // Optional trailing comma
            if !input.is_empty() {
                input.parse::<Token![,]>()?;
            }
        }

        Ok(RtEngineBackend {
            in_size: in_size.unwrap_or(1),
            out_size: out_size.unwrap_or(1),
        })
    }
}

impl super::Backend for RtEngineBackend {
    fn check_compatibility(&self, _: &CommonComponent) -> Result<(), syn::Error> {
        Ok(())
    }

    fn input_channel(
        &self,
        _model: &CommonComponent,
    ) -> (TokenStream2, TokenStream2, TokenStream2) {
        let in_size = self.in_size;
        let channel_type = quote::quote! { ::xdevs::export::InputChannel<
            <Self as ::xdevs::traits::BagMux>::Mux,
            #in_size
        > };
        let channel_call = quote::quote! {::xdevs::export::InputChannel::new() };
        let private_channel = TokenStream2::new();
        (channel_type, channel_call, private_channel)
    }

    fn output_channel(
        &self,
        _model: &CommonComponent,
    ) -> (TokenStream2, TokenStream2, TokenStream2) {
        let out_size = self.out_size;
        let channel_type = quote::quote! { ::xdevs::export::OutputChannel<
            <Self as ::xdevs::traits::BagMux>::Mux,
            #out_size
        > };
        let channel_call = quote::quote! {::xdevs::export::OutputChannel::new() };
        let private_channel = TokenStream2::new();
        (channel_type, channel_call, private_channel)
    }
}
