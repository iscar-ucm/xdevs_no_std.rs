use proc_macro2::TokenStream as TokenStream2;
use syn::{
    Error, Ident, LitInt, Token, Type, parse::{Parse, ParseStream}
};

use crate::component2::Field;

use super::port::Ports;

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
        let mut in_size = None;
        let mut out_size = None;
        let mut max_out_subs = None;

        while !input.is_empty() {
            let ident: Ident = input.parse()?;
            input.parse::<Token![=]>()?;
            let value: LitInt = input.parse()?;
            let value: usize = value.base10_parse()?;

            match ident.to_string().as_str() {
                "in_size" => in_size = Some(value),
                "out_size" => out_size = Some(value),
                "max_out_subs" => max_out_subs = Some(value),
                _ => {
                    return Err(Error::new(
                        ident.span(),
                        "unknown top argument; expected `in_size`, `out_size`, or `max_out_subs`",
                    ))
                }
            }

            // Optional trailing comma
            if !input.is_empty() {
                input.parse::<Token![,]>()?;
            }
        }

        Ok(RtEngine {
            in_size: in_size
                .ok_or_else(|| Error::new(input.span(), "missing mandatory argument: in_size"))?,
            out_size: out_size
                .ok_or_else(|| Error::new(input.span(), "missing mandatory argument: out_size"))?,
            max_out_subs: max_out_subs.ok_or_else(|| {
                Error::new(input.span(), "missing mandatory argument: max_out_subs")
            })?,
        })
    }
}

impl RtEngine {
    /// Generate the top-level infrastructure code:
    /// - Input/Output enums for channel communication
    /// - Static input `Channel` and output `PubSubChannel`
    /// - `unsafe impl RtEngine` with `map_input` and `map_output`
    ///
    /// The original item (with inner attributes like `#[atomic]`, `#[coupled]`, etc.)
    /// is passed through unchanged so those macros can process the struct next.
    pub fn quote(&self, model_ident: &syn::Ident, model_generics: &syn::Generics, input: &Ports, output: &Ports, input_ident: &syn::Ident, output_ident: &syn::Ident) -> TokenStream2 {
        let mut generated = TokenStream2::new();

        // Generate identifiers for code generation
        let input_enum_ident = quote::format_ident!("{}InputEnum", model_ident);
        let output_enum_ident = quote::format_ident!("{}OutputEnum", model_ident);

        let sender_ident = quote::format_ident!("{}Sender", model_ident);
        let subscriber_ident = quote::format_ident!("{}Subscriber", model_ident);

        let upper_name = to_screaming_snake_case(&model_ident.to_string());
        let in_channel_ident = quote::format_ident!("{}_IN_CHANNEL", upper_name);
        let out_channel_ident = quote::format_ident!("{}_OUT_CHANNEL", upper_name);

        let snake_name = to_snake_case(&model_ident.to_string());
        let private_mod_ident =
        quote::format_ident!("__xdevs_no_std_private_{}_rt_engine", snake_name);

        // Extract model generics
        let (model_impl_generics, model_ty_generics, model_where_clause) =
            model_generics.split_for_impl();

        // Extract input and output parameters
        let input_ports = &input.ports;
        let output_ports = &output.ports;
        let (input_impl_generics, input_ty_generics, input_where_clause) =
            input.generics.split_for_impl();
        let (output_impl_generics, output_ty_generics, output_where_clause) =
            output.generics.split_for_impl();

        // Get sizes
        let in_size = &self.in_size;
        let out_size = &self.out_size;
        let max_out_subs = &self.max_out_subs;

        let mut private = TokenStream2::new();

        // Input generation
        let map_input_body;
        let input_channel_type;
        let input_channel_call;

        if !input_ports.is_empty() {
            let input_variants: Vec<TokenStream2> = input_ports
                .iter()
                .map(|info| {
                    let variant = to_pascal_case_ident(&info.ident);
                    let ty = &info.ty;
                    quote::quote! { #variant(<#ty as ::xdevs::traits::TypedBag>::Item) }
                })
                .collect();

            let match_arms: Vec<TokenStream2> = input_ports
                .iter()
                .map(|info| {
                    let variant = to_pascal_case_ident(&info.ident);
                    let arm_body = expand_input_match_arm(info);
                    quote::quote!{
                        #input_enum_ident::#variant(value) => #arm_body
                    }
                })
                .collect();

            generated.extend(quote::quote! {
                /// Auto-generated input enum for top-level channel communication.
                pub enum #input_enum_ident #input_impl_generics #input_where_clause {
                    #(#input_variants),*
                }

                // Auto-generated sender type alias for the RtEngine implementation.
                pub type #sender_ident #model_impl_generics = <<<#model_ident #model_ty_generics as ::xdevs::traits::Component>::
                Input as ::xdevs::traits::MapInput>::InputChannel as 
                ::xdevs::traits::RtEngineInputChannel>::Sender;
            });
            private.extend(quote::quote! {
                /// Auto-generated static input channel.
                pub static #in_channel_ident #input_impl_generics: ::xdevs::export::Channel<
                    ::xdevs::export::Mutex,
                    #input_enum_ident #input_ty_generics,
                    #in_size
                > = ::xdevs::export::Channel::new();
            });

            map_input_body = quote::quote! {
                match ::xdevs::traits::RtEngineInputChannel::recv(in_channel).await {
                    #(#match_arms),*
                }
            };
            input_channel_type = quote::quote! { ::xdevs::export::InputChannel<'static,
                    #input_enum_ident #input_ty_generics,
                    #in_size
                > };
            input_channel_call = quote::quote! {::xdevs::export::InputChannel::new(&#private_mod_ident::#in_channel_ident) };
        } else {
            map_input_body = quote::quote! {};
            input_channel_type = quote::quote! { () };
            input_channel_call = quote::quote! { () };
        }

        // Output generation
        let map_output_body;
        let output_channel_type;
        let output_channel_call;

        if !output_ports.is_empty() {
            let output_variants: Vec<TokenStream2> = output_ports
                .iter()
                .map(|info| {
                    let variant = to_pascal_case_ident(&info.ident);
                    let ty = &info.ty;
                    quote::quote! { #variant(<#ty as ::xdevs::traits::TypedBag>::Item) }
                })
                .collect();

            let propagations: Vec<TokenStream2> = output_ports
                .iter()
                .map(|info| {
                    let variant = to_pascal_case_ident(&info.ident);
                    let for_body = expand_output_for(info);
                    quote::quote! {
                        let publish_fn = |value| {
                             ::xdevs::traits::RtEngineOutputChannel::publish(
                                out_channel, 
                                #output_enum_ident::#variant(value));
                        };
                        #for_body
                    }
                })
                .collect();

            generated.extend(quote::quote! {
                /// Auto-generated output enum for top-level channel communication.
                #[derive(Clone)]
                pub enum #output_enum_ident #output_impl_generics #output_where_clause {
                    #(#output_variants),*
                }

                /// Auto-generated output subscriber type alias.
                pub type #subscriber_ident #model_impl_generics = <<<#model_ident #model_ty_generics as ::xdevs::traits::Component>::
                Output as ::xdevs::traits::MapOutput>::OutputChannel as 
                ::xdevs::traits::RtEngineOutputChannel>::Subscriber;
            });
            private.extend(quote::quote! {
                /// Auto-generated static output PubSub channel.
                pub static #out_channel_ident #output_impl_generics: ::xdevs::export::PubSubChannel<
                    ::xdevs::export::Mutex,
                    #output_enum_ident #output_ty_generics,
                    #out_size,
                    #max_out_subs,
                    1
                > = ::xdevs::export::PubSubChannel::new();
            });

            map_output_body = quote::quote! {
                #(#propagations)*
            };
            output_channel_type = quote::quote! { ::xdevs::export::OutputChannel<'static,
                    #output_enum_ident #output_ty_generics,
                    #out_size,
                    #max_out_subs                
                > };
            output_channel_call = quote::quote! {::xdevs::export::OutputChannel::new(&#private_mod_ident::#out_channel_ident) };
        } else {
            map_output_body = quote::quote! {};
            output_channel_type = quote::quote! { () };
            output_channel_call = quote::quote! { () };
        }

        // === RtEngine trait implementation ===
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
        });

        // Combine all generated code into the final output
        generated.extend(quote::quote! {            
            /// Hidden module containing auto-generated infrastructure for the top-level component.
            mod #private_mod_ident {
                use super::*;
                #private
            }
        });

        generated
    }
}

/// Generate a match arm for the input enum to add received values to the corresponding input port.
fn expand_input_match_arm(info: &Field) -> TokenStream2 {
    fn input_match_arm_body(ty: &Type) -> TokenStream2 {
        match ty{
            Type::Path(_) => {quote::quote! {
                port.add_value(value).unwrap();
            }},
            Type::Array(array) => {
                let elem_ty = &array.elem;
                let body = input_match_arm_body(elem_ty);
                quote::quote! {
                    let (index, value) = value;
                    if let Some(port) = port.get_mut(index)
                    {
                        #body
                    }
                }
            },
            _ => {
                quote::quote! {
                    compile_error!("unsupported input port type; expected array or Port");
                }
            },
        }
    }
    let field = &info.ident;
    let ty = &info.ty;
    let body = input_match_arm_body(ty);
    quote::quote! {
        {
            let port = &mut self.#field;
            {
                #body
            }
        }
    }
}

/// Generate a for loop for the output enum to publish values from the corresponding output port.
fn expand_output_for(info: &Field) -> TokenStream2 {
    fn output_for_body(ty: &Type, from_array: bool) -> TokenStream2 {
        match ty{
            Type::Path(_) => {          
                if from_array {
                    quote::quote! {
                        for value in port.iter() {
                            publish_fn((index, value.clone()));
                        }
                    }
                } else {
                    quote::quote! {
                        for value in port.get_values() {
                            publish_fn(value.clone());
                        }
                    }
                }
            },
            Type::Array(array) => {
                let body = output_for_body(&array.elem, true);
                quote::quote!{
                    for (index, port) in port.iter().enumerate() {
                        #body
                    }
                }
            },
            _ => {
                quote::quote! {
                    compile_error!("unsupported output port type; expected array or Port");
                }
            },
        }
    }   
    let field = &info.ident;
    let ty = &info.ty;
    let body = output_for_body(ty, false);
    quote::quote! {
        let port = &self.#field;
        {
            #body
        }
    }
}

/// Converts a snake_case identifier to PascalCase.
fn to_pascal_case_ident(ident: &Ident) -> Ident {
    let s = ident.to_string();
    let pascal: String = s
        .split('_')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                Some(c) => c.to_uppercase().collect::<String>() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect();
    Ident::new(&pascal, ident.span())
}

/// Converts PascalCase to SCREAMING_SNAKE_CASE.
fn to_screaming_snake_case(s: &str) -> String {
    let mut result = String::new();
    for (i, ch) in s.chars().enumerate() {
        if ch.is_uppercase() && i > 0 {
            result.push('_');
        }
        result.push(ch.to_ascii_uppercase());
    }
    result
}

/// Converts PascalCase to snake_case.
fn to_snake_case(s: &str) -> String {
    let mut result = String::new();
    for (i, ch) in s.chars().enumerate() {
        if ch.is_uppercase() && i > 0 {
            result.push('_');
        }
        result.push(ch.to_ascii_lowercase());
    }
    result
}
