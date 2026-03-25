use proc_macro2::TokenStream as TokenStream2;
use heck::{ToSnakeCase};
use syn::{
    Error, Ident, LitInt, Token, parse::{Parse, ParseStream}
};

use crate::component2::{CommonComponent, backend::{Backend, ChannelGenerator}};

/// Arguments for the `#[rt_engine]` attribute macro.
///
/// Supported arguments:
/// - `in_size`: capacity of the input channel
/// - `out_size`: capacity of the output channel
/// - `max_out_subs`: number of subscribers for the output PubSubChannel
pub struct RtEngine {
    in_size: usize,
    out_size: usize,
    max_out_subs: usize,
}

impl Parse for RtEngine {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let channel_generator = ChannelGenerator::new();
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
                        return Err(Error::new(
                            ident.span(),
                            "duplicate argument: in_size",
                        ));
                    }
                    else{
                        in_size = Some(value)
                    }
                },
                "out_size" => {
                    if let Some(_) = out_size {
                        return Err(Error::new(
                            ident.span(),
                            "duplicate argument: out_size",
                        ));
                    }
                    else{
                        out_size = Some(value)
                    }
                },
                "max_out_subs" => channel_generator.parse_max_out_subs(&mut max_out_subs, value)?,
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

        Ok(RtEngine {
            in_size: in_size.unwrap_or(0),
            out_size: out_size.unwrap_or(0),
            max_out_subs: max_out_subs.unwrap_or(0),
        })
    }
}

impl CommonComponent {
    /// Generate the rt-engine infrastructure code:
    pub fn quote_rt_engine(&self) -> TokenStream2 {
        if let Some(rt_engine) = &self.rt_engine{
            let mut channel_generator = ChannelGenerator::new();
            let mut generated = TokenStream2::new();

            // Generate identifiers for code generation
            let model_ident = &self.ident;
            let input_enum_ident = quote::format_ident!("{}InputEnum", self.ident);
            let output_enum_ident = quote::format_ident!("{}OutputEnum", self.ident);
            let sender_ident = quote::format_ident!("{}Sender", self.ident);
            let subscriber_ident = quote::format_ident!("{}Subscriber", self.ident);

            let snake_name = self.ident.to_string().to_snake_case();
            let private_mod_ident =
            quote::format_ident!("__xdevs_no_std_private_{}_rt_engine", snake_name);

            // Extract model generics
            let (model_impl_generics, model_ty_generics, model_where_clause) =
                self.generics.split_for_impl();

            // Extract input and output parameters
            let input_ident = self.input.ident();
            let output_ident = self.output.ident();
            let input_ports = self.input.ports();
            let output_ports = self.output.ports();
            let (input_impl_generics, input_ty_generics, input_where_clause) =
                self.input.generics().split_for_impl();
            let (output_impl_generics, output_ty_generics, output_where_clause) =
                self.output.generics().split_for_impl();

            // Get sizes
            let in_size = rt_engine.in_size;
            let out_size = rt_engine.out_size;

            // Input generation
            let map_input_body;
            let input_channel_type;
            let input_channel_call;
            let private_input_channel;

            if !input_ports.is_empty() {
                generated.extend(quote::quote! {
                    // Auto-generated sender type alias for the RtEngine implementation.
                    pub type #sender_ident #model_impl_generics = <<<#model_ident #model_ty_generics as ::xdevs::traits::Component>::
                    Input as ::xdevs::traits::MapInput>::
                    InputChannel as ::xdevs::traits::RtEngineInputChannel>::Sender;

                    /// Auto-generated input enum for channel communication alias.
                    pub type #input_enum_ident #model_impl_generics = <<<#model_ident #model_ty_generics as ::xdevs::traits::Component>::
                    Input as ::xdevs::traits::BagMux>::Enum;
                });
                (input_channel_type, input_channel_call, private_input_channel) = channel_generator.input_channel(&model_ident, &input_enum_ident, &input_ty_generics, in_size);
                map_input_body = quote::quote!{
                    let input = in_channel.recv().await;
                    <self as ::xdevs::traits::BagMux>::enum_to_input(self, input);
                }
            } else {
                map_input_body = quote::quote! {};
                input_channel_type = quote::quote! { () };
                input_channel_call = quote::quote! { () };
                private_input_channel = TokenStream2::new();
            }

            // Output generation
            let map_output_body;
            let output_channel_type;
            let output_channel_call;
            let private_output_channel;

            if !output_ports.is_empty() {
                generated.extend(quote::quote! {
                    /// Auto-generated output subscriber type alias.
                    pub type #subscriber_ident #model_impl_generics = <<<#model_ident #model_ty_generics as ::xdevs::traits::Component>::
                    Output as ::xdevs::traits::MapOutput>::OutputChannel as 
                    ::xdevs::traits::RtEngineOutputChannel>::Subscriber;

                    /// Auto-generated output enum for channel communication alias.
                    pub type #output_enum_ident #model_impl_generics = <<<#model_ident #model_ty_generics as ::xdevs::traits::Component>::
                    Output as ::xdevs::traits::BagMux>::Enum;
                });
                (output_channel_type, output_channel_call, private_output_channel) = channel_generator.output_channel(&model_ident, &output_enum_ident, &output_ty_generics, out_size);
                map_output_body = quote::quote!{
                    let output = <self as ::xdevs::traits::BagMux>::output_from_enum(self);
                    out_channel.publish(output);
                }
            } else {
                map_output_body = quote::quote! {};
                output_channel_type = quote::quote! { () };
                output_channel_call = quote::quote! { () };
                private_output_channel = TokenStream2::new();
            }

            // RtEngine trait implementation
            generated.extend(quote::quote! {
                /// Auto-generated `MapInput` implementation for the top-level component input.
                unsafe impl #input_impl_generics ::xdevs::traits::MapInput for #input_ident #input_ty_generics #input_where_clause {
                    type InputChannel = #input_channel_type;
                    
                    async unsafe fn map_input(
                        &mut self,
                        in_channel: &Self::InputChannel,
                    ) {
                        #map_input_body
                    }
                }

                /// Auto-generated `MapOutput` implementation for the top-level component output.
                unsafe impl #output_impl_generics ::xdevs::traits::MapOutput for #output_ident #output_ty_generics #output_where_clause {
                    type OutputChannel = #output_channel_type;

                    unsafe fn map_output(
                        &self,
                        out_channel: &Self::OutputChannel,
                    ) {
                        #map_output_body
                    }
                }

                impl #model_impl_generics #model_ident #model_ty_generics #model_where_clause {
                    /// Constructor for RtEngine.
                    pub fn into_rt_engine(self) -> ::xdevs::rt_engine::RtEngine<Self> {
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

            generated
        }
    else{
        TokenStream2::new()
    }
    }
}