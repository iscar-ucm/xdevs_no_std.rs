use heck::ToShoutySnakeCase;
use proc_macro2::TokenStream as TokenStream2;

use crate::component2::CommonComponent;

pub struct RtEngineBackend {}

impl RtEngineBackend {
    pub fn new() -> Self {
        Self {}
    }
}

impl super::Backend for RtEngineBackend {
    fn parse_max_out_subs(
        &self,
        max_out_subs: &mut Option<usize>,
        value: usize,
    ) -> Result<(), syn::Error> {
        if max_out_subs.is_some() {
            Err(syn::Error::new(
                proc_macro2::Span::call_site(),
                "duplicate argument: max_out_subs",
            ))
        } else {
            *max_out_subs = Some(value);
            Ok(())
        }
    }

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
        if let Some(rt_engine) = &model.rt_engine {
            let model_ident = &model.ident;
            let input_ident = model.input.ident();
            let in_size = rt_engine.in_size();

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
        } else {
            panic!("rt_engine configuration is required for input channel generation");
        }
    }

    fn output_channel(
        &self,
        model: &CommonComponent,
    ) -> (TokenStream2, TokenStream2, TokenStream2) {
        if let Some(rt_engine) = &model.rt_engine {
            let model_ident = &model.ident;
            let output_ident = model.output.ident();
            let out_size = rt_engine.out_size();
            let max_out_subs = rt_engine.max_out_subs();

            let channel_type = quote::quote! { ::xdevs::export::OutputChannel<'static,
                <Self as ::xdevs::traits::BagMux>::Mux,
                #out_size,
                #max_out_subs
            > };
            let upper_name = model_ident.to_string().to_shouty_snake_case();
            let channel_ident = quote::format_ident!("{}_OUT_CHANNEL", upper_name);
            let channel_call =
                quote::quote! {::xdevs::export::OutputChannel::new(&#channel_ident) };

            let private_channel = quote::quote! {
                /// Auto-generated static output PubSub channel.
                pub static #channel_ident: ::xdevs::export::PubSubChannel<
                    <#output_ident as ::xdevs::traits::BagMux>::Mux,
                    #out_size,
                    #max_out_subs,
                > = ::xdevs::export::PubSubChannel::new();
            };

            (channel_type, channel_call, private_channel)
        } else {
            panic!("rt_engine configuration is required for output channel generation");
        }
    }
}
