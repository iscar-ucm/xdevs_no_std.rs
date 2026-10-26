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

        #[cfg(not(any(feature = "embassy-backend", feature = "tokio-backend")))]
        {
            return Err(syn::Error::new(
                input.span(),
                "No backend feature enabled. Please enable either the `embassy` or `tokio` feature.",
            ));
        }

        #[cfg(any(feature = "embassy-backend", feature = "tokio-backend"))]
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

    // Conditionally create the input channel and its public aliases.
    let mut input_channel_type = quote::quote! {()};
    let mut input_channel_call = quote::quote! {()};
    let mut private_input_channel = quote::quote! {};

    if !is_input_unit {
        let input_channel_tokens =
            RtEngineBackend::input_channel(&args, &model_ident, &model_ty_generics);
        (
            input_channel_type,
            input_channel_call,
            private_input_channel,
        ) = input_channel_tokens.split();

        generated.extend(quote::quote! {
            // Auto-generated sender type alias for the RtEngine implementation.
            pub type #sender_ident #model_ty_generics =
                <#input_channel_type as ::xdevs::rt_engine::RtEngineInputChannel>::Sender;

            /// Auto-generated input enum for channel communication alias.
            pub type #input_enum_ident #model_ty_generics = <<#model_ident #model_ty_generics as ::xdevs::Component>::
            Input as ::xdevs::bag::Bag>::Value;
        });
    };

    // Conditionally create the output channel and its public aliases.
    let mut output_channel_type = quote::quote! {()};
    let mut output_channel_call = quote::quote! {()};
    let mut private_output_channel = quote::quote! {};

    if !is_output_unit {
        let output_channel_tokens =
            RtEngineBackend::output_channel(&args, &model_ident, &model_ty_generics);
        (
            output_channel_type,
            output_channel_call,
            private_output_channel,
        ) = output_channel_tokens.split();

        generated.extend(quote::quote! {
            /// Auto-generated output receiver type alias.
            pub type #receiver_ident #model_ty_generics =
                <#output_channel_type as ::xdevs::rt_engine::RtEngineOutputChannel>::Receiver;

            /// Auto-generated output enum for channel communication alias.
            pub type #output_enum_ident #model_ty_generics = <<#model_ident #model_ty_generics as ::xdevs::Component>::
            Output as ::xdevs::bag::Bag>::Value;
        });
    };

    generated.extend(quote::quote! {
        /// Original impl block
        #item

        impl #model_impl_generics #model_ident #model_ty_generics #model_where_clause {
            /// Constructor for RtEngine.
            pub fn into_rt_engine(self) -> ::xdevs::rt_engine::RtEngine<
                <Self as ::xdevs::Component>::Kind,
                Self,
                #input_channel_type,
                #output_channel_type,
            > {
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
