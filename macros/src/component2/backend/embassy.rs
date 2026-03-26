use heck::ToShoutySnakeCase;
use proc_macro2::TokenStream as TokenStream2;
use syn::{parse::ParseStream, Error, Ident, LitInt, Token};

use crate::component2::CommonComponent;

/// Arguments for the `#[rt_engine]` attribute macro.
///
/// Supported arguments:
/// - `in_size`: capacity of the input channel
/// - `out_size`: capacity of the output channel
/// - `max_out_subs`: number of subscribers for the output PubSubChannel
pub struct RtEngineBackend {
    in_size: usize,
    out_size: usize,
    max_out_subs: usize,
}

impl Default for RtEngineBackend {
    fn default() -> Self {
        Self {
            in_size: 1,
            out_size: 1,
            max_out_subs: 1,
        }
    }
}

impl syn::parse::Parse for RtEngineBackend {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut in_size = None;
        let mut out_size = None;
        let mut max_out_subs = None;

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
                    if max_out_subs.is_some() {
                        return Err(Error::new(
                            proc_macro2::Span::call_site(),
                            "duplicate argument: max_out_subs",
                        ));
                    } else {
                        max_out_subs = Some(value);
                    }
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
            max_out_subs: max_out_subs.unwrap_or(1),
        })
    }
}

impl super::Backend for RtEngineBackend {
    fn check_compatibility(&self, model: &CommonComponent) -> Result<(), syn::Error> {
        let has_input_generics = !model.input.generics().params.is_empty();
        let has_output_generics = !model.output.generics().params.is_empty();

        if has_input_generics || has_output_generics {
            Err(syn::Error::new_spanned(
                &model.ident,
                "rt_engine with embassy backend does not support generic input/output types",
            ))
        } else {
            Ok(())
        }
    }

    fn input_channel(&self, model: &CommonComponent) -> (TokenStream2, TokenStream2, TokenStream2) {
        let model_ident = &model.ident;
        let input_ident = model.input.ident();
        let in_size = self.in_size;

        let channel_type = quote::quote! { ::xdevs::export::InputChannel<'static,
            <Self as ::xdevs::traits::BagMux>::Mux,
            #in_size
        > };
        let upper_name = model_ident.to_string().to_shouty_snake_case();
        let channel_ident = quote::format_ident!("{}_IN_CHANNEL", upper_name);
        let channel_call = quote::quote! {::xdevs::export::InputChannel::new(&#channel_ident) };

        let private_channel = quote::quote! {
            /// Auto-generated static input channel.
            pub static #channel_ident: ::xdevs::export::Channel<
                <#input_ident as ::xdevs::traits::BagMux>::Mux,
                #in_size
            > = ::xdevs::export::Channel::new();
        };

        (channel_type, channel_call, private_channel)
    }

    fn output_channel(
        &self,
        model: &CommonComponent,
    ) -> (TokenStream2, TokenStream2, TokenStream2) {
        let model_ident = &model.ident;
        let output_ident = model.output.ident();
        let out_size = self.out_size;
        let max_out_subs = self.max_out_subs;

        let channel_type = quote::quote! { ::xdevs::export::OutputChannel<'static,
            <Self as ::xdevs::traits::BagMux>::Mux,
            #out_size,
            #max_out_subs
        > };
        let upper_name = model_ident.to_string().to_shouty_snake_case();
        let channel_ident = quote::format_ident!("{}_OUT_CHANNEL", upper_name);
        let channel_call = quote::quote! {::xdevs::export::OutputChannel::new(&#channel_ident) };

        let private_channel = quote::quote! {
            /// Auto-generated static output PubSub channel.
            pub static #channel_ident: ::xdevs::export::PubSubChannel<
                <#output_ident as ::xdevs::traits::BagMux>::Mux,
                #out_size,
                #max_out_subs,
            > = ::xdevs::export::PubSubChannel::new();
        };

        (channel_type, channel_call, private_channel)
    }
}
