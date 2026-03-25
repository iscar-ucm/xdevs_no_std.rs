use proc_macro2::TokenStream as TokenStream2;
use heck::{ToSnakeCase};
use syn::{
    Error, Ident, LitInt, Token, parse::{Parse, ParseStream}
};

use crate::component2::{CommonComponent, backend::{Backend, RtEngineBackend}};

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

impl Default for RtEngine {
    fn default() -> Self {
        Self {
            in_size: 1,
            out_size: 1,
            max_out_subs: 1,
        }
    }
}

impl Parse for RtEngine {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let rt_backend = RtEngineBackend::new();
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
                "max_out_subs" => rt_backend.parse_max_out_subs(&mut max_out_subs, value)?,
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
            in_size: in_size.unwrap_or(1),
            out_size: out_size.unwrap_or(1),
            max_out_subs: max_out_subs.unwrap_or(1),
        })
    }
}

impl RtEngine {
    pub fn in_size(&self) -> usize {
        self.in_size
    }
    pub fn out_size(&self) -> usize {
        self.out_size
    }
    pub fn max_out_subs(&self) -> usize {
        self.max_out_subs
    }

}

impl CommonComponent {
    /// Generate the rt-engine infrastructure code:
    pub fn quote_rt_engine(&self) -> TokenStream2 {
        if let Some(_) = &self.rt_engine{
            let rt_backend = RtEngineBackend::new();
            let mut generated = TokenStream2::new();

            // Check compatibility of the component with the selected rt-engine backend.
            let compatibility = rt_backend.check_compatibility(&self);
            if let Err(e) = compatibility {
                return e.to_compile_error();
            }

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

            // Input generation
            let map_input_body;
            let input_channel_type;
            let input_channel_call;
            let private_input_channel;

            if !input_ports.is_empty() {
                generated.extend(quote::quote! {
                    // Auto-generated sender type alias for the RtEngine implementation.
                    pub type #sender_ident #model_ty_generics = <<<#model_ident #model_ty_generics as ::xdevs::traits::Component>::
                    Input as ::xdevs::traits::MapInput>::
                    InputChannel as ::xdevs::traits::RtEngineInputChannel>::Sender;

                    /// Auto-generated input enum for channel communication alias.
                    pub type #input_enum_ident #model_ty_generics = <<#model_ident #model_ty_generics as ::xdevs::traits::Component>::
                    Input as ::xdevs::traits::BagMux>::Mux;
                });
                (input_channel_type, input_channel_call, private_input_channel) = rt_backend.input_channel(&self);
                map_input_body = quote::quote!{
                    let input = <Self::InputChannel as ::xdevs::traits::RtEngineInputChannel>::recv(in_channel).await;
                    <Self as ::xdevs::traits::BagMux>::inject_event(self, input);
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
                    pub type #subscriber_ident #model_ty_generics = <<<#model_ident #model_ty_generics as ::xdevs::traits::Component>::
                    Output as ::xdevs::traits::MapOutput>::OutputChannel as 
                    ::xdevs::traits::RtEngineOutputChannel>::Subscriber;

                    /// Auto-generated output enum for channel communication alias.
                    pub type #output_enum_ident #model_ty_generics = <<#model_ident #model_ty_generics as ::xdevs::traits::Component>::
                    Output as ::xdevs::traits::BagMux>::Mux;
                });
                (output_channel_type, output_channel_call, private_output_channel) = rt_backend.output_channel(&self);
                map_output_body = quote::quote!{
                    let out_func = |output| {
                        <Self::OutputChannel as ::xdevs::traits::RtEngineOutputChannel>::publish(
                            out_channel,
                            output,
                        );
                    };
                    <Self as ::xdevs::traits::BagMux>::eject_events(self, out_func);
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
                        in_channel: &mut Self::InputChannel,
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

            generated
        }
    else{
        TokenStream2::new()
    }
    }
}