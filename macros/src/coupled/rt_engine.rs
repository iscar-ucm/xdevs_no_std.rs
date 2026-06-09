use super::backend::Backend;
use crate::coupled::ComponentArgs;
use heck::ToSnakeCase;
use proc_macro2::TokenStream as TokenStream2;
use syn::{ItemStruct, Result};

/// Generate the rt-engine infrastructure code:
pub fn expand_rt_engine(args: &ComponentArgs, item: &ItemStruct) -> Result<TokenStream2> {
    if let Some(rt_engine) = &args.rt_engine {
        let mut generated = TokenStream2::new();
        rt_engine.check_compatibility(args)?;

        // Generate identifiers for code generation
        let model_ident = &item.ident;
        let input_enum_ident = quote::format_ident!("{}InputEnum", item.ident);
        let output_enum_ident = quote::format_ident!("{}OutputEnum", item.ident);
        let sender_ident = quote::format_ident!("{}Sender", item.ident);
        let receiver_ident = quote::format_ident!("{}Receiver", item.ident);

        let snake_name = item.ident.to_string().to_snake_case();
        let private_mod_ident =
            quote::format_ident!("__xdevs_no_std_private_{}_rt_engine", snake_name);

        // Extract model generics
        let (model_impl_generics, model_ty_generics, model_where_clause) =
            item.generics.split_for_impl();

        // Input generation
        generated.extend(quote::quote! {
                    // Auto-generated sender type alias for the RtEngine implementation.
                    pub type #sender_ident #model_ty_generics = <<<#model_ident #model_ty_generics as ::xdevs::traits::Component>::
                    Input as ::xdevs::traits::InjectInput>::
                    InputChannel as ::xdevs::traits::RtEngineInputChannel>::Sender;

                    /// Auto-generated input enum for channel communication alias.
                    pub type #input_enum_ident #model_ty_generics = <<#model_ident #model_ty_generics as ::xdevs::traits::Component>::
                    Input as ::xdevs::traits::BagMux>::Mux;
                });

        let input_channel_tokens = rt_engine.input_channel(&item);

        let map_input_body = quote::quote! {
            let input = <Self::InputChannel as ::xdevs::traits::RtEngineInputChannel>::recv(in_channel).await;
            // TODO: Return Result when embassy time is merged
            let _ = <Self as ::xdevs::traits::BagMux>::inject_event(self, input);
        };
        let input_channel_type = input_channel_tokens.channel_type;
        let input_channel_call = input_channel_tokens.channel_call;
        let private_input_channel = input_channel_tokens.private_channel;

        // Output generation
        generated.extend(quote::quote! {
                    /// Auto-generated output receiver type alias.
                    pub type #receiver_ident #model_ty_generics = <<<#model_ident #model_ty_generics as ::xdevs::traits::Component>::
                    Output as ::xdevs::traits::EjectOutput>::OutputChannel as
                    ::xdevs::traits::RtEngineOutputChannel>::Receiver;

                    /// Auto-generated output enum for channel communication alias.
                    pub type #output_enum_ident #model_ty_generics = <<#model_ident #model_ty_generics as ::xdevs::traits::Component>::
                    Output as ::xdevs::traits::BagMux>::Mux;
                });
        let output_channel_tokens = rt_engine.output_channel(&item);

        let map_output_body = quote::quote! {
            let out_func = |output| {
                <Self::OutputChannel as ::xdevs::traits::RtEngineOutputChannel>::publish(
                    out_channel,
                    output,
                );
            };
            <Self as ::xdevs::traits::BagMux>::eject_events(self, out_func);
        };
        let output_channel_type = output_channel_tokens.channel_type;
        let output_channel_call = output_channel_tokens.channel_call;
        let private_output_channel = output_channel_tokens.private_channel;

        // RtEngine trait implementation
        generated.extend(quote::quote! {
                /// Auto-generated `InjectInput` implementation for the top-level component input.
                unsafe impl #model_impl_generics ::xdevs::traits::InjectInput for <#model_ident #model_ty_generics as ::xdevs::traits::Component>::Input #model_where_clause {
                    type InputChannel = #input_channel_type;
                    async fn map_input(
                        &mut self,
                        in_channel: &mut Self::InputChannel,
                    ) {
                        #map_input_body
                    }
                }

                /// Auto-generated `EjectOutput` implementation for the top-level component output.
                unsafe impl #model_impl_generics ::xdevs::traits::EjectOutput for <#model_ident #model_ty_generics as ::xdevs::traits::Component>::Output #model_where_clause {
                    type OutputChannel = #output_channel_type;

                    fn map_output(
                        &self,
                        out_channel: &Self::OutputChannel,
                    ) {
                        #map_output_body
                    }
                }

                impl #model_impl_generics #model_ident #model_ty_generics #model_where_clause {
                    /// Constructor for RtEngine.
                    pub fn into_rt_engine(self) -> ::xdevs::rt_engine::RtEngine<Self> {
                        use #private_mod_ident::*;
                        ::xdevs::rt_engine::RtEngine::new(
                            self,
                            #input_channel_call,
                            #output_channel_call,
                        )
                    }
                }

                mod #private_mod_ident {
                    use super::*;
                    #private_input_channel
                    #private_output_channel
                }
            });

        Ok(generated)
    } else {
        Ok(TokenStream2::new())
    }
}
