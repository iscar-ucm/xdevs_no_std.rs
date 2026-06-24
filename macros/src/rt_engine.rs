mod backend;

use backend::{Backend, RtEngineBackend};
use heck::ToSnakeCase;
use proc_macro2::TokenStream as TokenStream2;
use syn::{
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    Error, Ident, ItemImpl, Meta, Result, Token, Type, TypeTuple,
};

use crate::combine_err;

/// Generated token fragments used to construct backend channel code.
pub struct ChannelTokens {
    pub channel_type: TokenStream2,
    pub channel_call: TokenStream2,
    pub private_channel: TokenStream2,
}
impl ChannelTokens {
    fn split(self) -> (TokenStream2, TokenStream2, TokenStream2) {
        (self.channel_type, self.channel_call, self.private_channel)
    }
}

/// Generic parsed arguments for `rt_engine(...)` backend configuration.
#[derive(Default, Debug)]
pub struct RtEngineArgs {
    #[allow(dead_code)]
    pub in_channel_size: usize,
    #[allow(dead_code)]
    pub out_channel_size: usize,
    #[allow(dead_code)]
    pub max_out_subs: usize,
}

impl Parse for RtEngineArgs {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut acc: Option<Error> = None;
        let mut in_channel_size = None;
        let mut out_channel_size = None;
        let mut max_out_subs = None;

        // Parse built-in Meta items
        let parsed_args = Punctuated::<Meta, Token![,]>::parse_terminated(input)?;

        for meta in parsed_args {
            // We only care about `path = value` (MetaNameValue)
            let nv = match meta {
                syn::Meta::NameValue(nv) => nv,
                _ => {
                    let err = Error::new_spanned(meta, "expected `name = value` format");
                    combine_err(&mut acc, err);
                    continue;
                }
            };

            if let Err(err) = RtEngineBackend::check_arg_compatibility(&nv) {
                combine_err(&mut acc, err);
                continue;
            }

            let name = nv
                .path
                .require_ident()
                .map(|i| i.to_string())
                .unwrap_or_default();

            // Extract the LitInt from the Expr
            let lit_int = match &nv.value {
                syn::Expr::Lit(syn::ExprLit {
                    lit: syn::Lit::Int(lit),
                    ..
                }) => lit,
                _ => {
                    let err = Error::new_spanned(&nv.value, "expected an integer literal");
                    combine_err(&mut acc, err);
                    continue;
                }
            };

            let value = match lit_int.base10_parse::<usize>() {
                Ok(v) => v,
                Err(e) => {
                    let err = Error::new_spanned(lit_int, format!("invalid integer: {}", e));
                    combine_err(&mut acc, err);
                    continue;
                }
            };
            let name = name.as_str();
            match name {
                "in_channel_size" => {
                    if in_channel_size.is_some() {
                        let err =
                            Error::new_spanned(&nv.path, "duplicate argument: in_channel_size");
                        combine_err(&mut acc, err);
                    } else {
                        in_channel_size = Some(value);
                    }
                }
                "out_channel_size" => {
                    if out_channel_size.is_some() {
                        let err =
                            Error::new_spanned(&nv.path, "duplicate argument: out_channel_size");
                        combine_err(&mut acc, err);
                    } else {
                        out_channel_size = Some(value);
                    }
                }
                "max_out_subs" => {
                    if max_out_subs.is_some() {
                        let err = Error::new_spanned(&nv.path, "duplicate argument: max_out_subs");
                        combine_err(&mut acc, err);
                    } else {
                        max_out_subs = Some(value);
                    }
                }
                _ => {
                    let err =
                        Error::new_spanned(&nv.path, format!("unknown rt_engine argument: {name}"));
                    combine_err(&mut acc, err);
                }
            }
        }

        #[cfg(not(any(feature = "embassy-backend", feature = "std-backend")))]
        {
            return Err(syn::Error::new(
                input.span(),
                "No backend feature enabled. Please enable either the `embassy` or `std` feature.",
            ));
        }

        #[cfg(any(feature = "embassy-backend", feature = "std-backend"))]
        {
            if let Some(err) = acc {
                return Err(err);
            }

            Ok(RtEngineArgs {
                in_channel_size: in_channel_size.unwrap_or(1),
                out_channel_size: out_channel_size.unwrap_or(1),
                max_out_subs: max_out_subs.unwrap_or(1),
            })
        }
    }
}

pub fn expand(args: RtEngineArgs, item: ItemImpl) -> Result<TokenStream2> {
    let mut acc: Option<Error> = None;
    let mut generated = TokenStream2::new();
    match RtEngineBackend::check_item_compatibility(&item) {
        Ok(()) => {}
        Err(e) => {
            combine_err(&mut acc, e);
        }
    }

    let mut is_input_unit = false;
    let mut is_output_unit = false;

    // Search implementations for the Input and Output associated types to see if they are the unit type ()
    for impl_item in &item.items {
        if let syn::ImplItem::Type(impl_type) = impl_item {
            if impl_type.ident == "Input" {
                is_input_unit = is_unit_type(&impl_type.ty);
            } else if impl_type.ident == "Output" {
                is_output_unit = is_unit_type(&impl_type.ty);
            }
        }
    }

    let model_ident = match ident_from_type(&item.self_ty) {
        Ok(ident) => ident.clone(),
        Err(e) => {
            combine_err(&mut acc, e);
            quote::format_ident!("Placeholder")
        }
    };

    if let Some(err) = acc {
        return Err(err);
    }

    // Generate identifiers for code generation
    let input_enum_ident = quote::format_ident!("{}InputEnum", model_ident);
    let output_enum_ident = quote::format_ident!("{}OutputEnum", model_ident);
    let sender_ident = quote::format_ident!("{}Sender", model_ident);
    let receiver_ident = quote::format_ident!("{}Receiver", model_ident);

    let snake_name = model_ident.to_string().to_snake_case();
    let private_mod_ident = quote::format_ident!("__xdevs_no_std_private_{}_rt_engine", snake_name);

    // Extract model generics
    let (model_impl_generics, model_ty_generics, model_where_clause) =
        item.generics.split_for_impl();

    // Conditionally implement `InjectInput`
    let mut input_channel_type = quote::quote! {()};
    let mut input_channel_call = quote::quote! {()};
    let mut private_input_channel = quote::quote! {};
    let mut inject_input_impl = quote::quote! {};

    if !is_input_unit {
        generated.extend(quote::quote! {
        // Auto-generated sender type alias for the RtEngine implementation.
        pub type #sender_ident #model_ty_generics = <<<#model_ident #model_ty_generics as ::xdevs::Component>::
        Input as ::xdevs::rt_engine::InjectInput>::
        InputChannel as ::xdevs::rt_engine::RtEngineInputChannel>::Sender;

        /// Auto-generated input enum for channel communication alias.
        pub type #input_enum_ident #model_ty_generics = <<#model_ident #model_ty_generics as ::xdevs::Component>::
        Input as ::xdevs::port::BagMux>::Mux;
        });

        let input_channel_tokens = RtEngineBackend::input_channel(&args, &model_ident);
        (
            input_channel_type,
            input_channel_call,
            private_input_channel,
        ) = input_channel_tokens.split();

        let map_input_body = quote::quote! {
            let input = <Self::InputChannel as ::xdevs::rt_engine::RtEngineInputChannel>::recv(in_channel).await;
            // TODO: Return Result when embassy time is merged
            let _ = <Self as ::xdevs::port::BagMux>::inject_event(self, input);
        };

        inject_input_impl = quote::quote! {
            /// Auto-generated `InjectInput` implementation for the top-level component input.
            unsafe impl #model_impl_generics ::xdevs::rt_engine::InjectInput for <#model_ident #model_ty_generics as ::xdevs::Component>::Input #model_where_clause {
                type InputChannel = #input_channel_type;
                async fn map_input(
                    &mut self,
                    in_channel: &mut Self::InputChannel,
                ) {
                    #map_input_body
                }
            }
        };
    };

    // Conditionally implement `EjectOutput`
    let mut output_channel_type = quote::quote! {()};
    let mut output_channel_call = quote::quote! {()};
    let mut private_output_channel = quote::quote! {};
    let mut eject_output_impl = quote::quote! {};

    if !is_output_unit {
        generated.extend(quote::quote! {
            /// Auto-generated output receiver type alias.
            pub type #receiver_ident #model_ty_generics = <<<#model_ident #model_ty_generics as ::xdevs::Component>::
            Output as ::xdevs::rt_engine::EjectOutput>::OutputChannel as
            ::xdevs::rt_engine::RtEngineOutputChannel>::Receiver;

            /// Auto-generated output enum for channel communication alias.
            pub type #output_enum_ident #model_ty_generics = <<#model_ident #model_ty_generics as ::xdevs::Component>::
            Output as ::xdevs::port::BagMux>::Mux;
        });

        let output_channel_tokens = RtEngineBackend::output_channel(&args, &model_ident);
        output_channel_type = output_channel_tokens.channel_type;
        output_channel_call = output_channel_tokens.channel_call;
        private_output_channel = output_channel_tokens.private_channel;

        let map_output_body = quote::quote! {
            let out_func = |output| {
                <Self::OutputChannel as ::xdevs::rt_engine::RtEngineOutputChannel>::publish(
                    out_channel,
                    output,
                );
            };
            <Self as ::xdevs::port::BagMux>::eject_events(self, out_func);
        };

        eject_output_impl = quote::quote! {
            /// Auto-generated `EjectOutput` implementation for the top-level component output.
            unsafe impl #model_impl_generics ::xdevs::rt_engine::EjectOutput for <#model_ident #model_ty_generics as ::xdevs::Component>::Output #model_where_clause {
                type OutputChannel = #output_channel_type;

                fn map_output(
                    &self,
                    out_channel: &Self::OutputChannel,
                ) {
                    #map_output_body
                }
            }
        };
    };

    // RtEngine trait implementation
    generated.extend(quote::quote! {
        /// Original impl block
        #item

        #inject_input_impl

        #eject_output_impl

        impl #model_impl_generics #model_ident #model_ty_generics #model_where_clause {
            /// Constructor for RtEngine.
            pub fn into_rt_engine(self) -> ::xdevs::rt_engine::RtEngine<<Self as ::xdevs::Component>::Kind,Self> {
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
}

fn is_unit_type(ty: &Type) -> bool {
    match ty {
        Type::Tuple(TypeTuple { elems, .. }) => elems.is_empty(),
        _ => false,
    }
}

pub fn ident_from_type(ty: &Type) -> Result<&Ident> {
    match ty {
        Type::Path(type_path) => Ok(type_path
            .path
            .segments
            .last()
            .map(|segment| &segment.ident)
            .unwrap()),

        Type::Paren(type_paren) => ident_from_type(&type_paren.elem),

        Type::Group(type_group) => ident_from_type(&type_group.elem),

        Type::Reference(type_reference) => ident_from_type(&type_reference.elem),

        _ => {
            return Err(Error::new_spanned(
                ty,
                "unsupported type for rt_engine component",
            ))
        }
    }
}
