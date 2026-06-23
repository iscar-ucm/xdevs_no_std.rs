use crate::combine_err;
use proc_macro2::TokenStream as TokenStream2;
use syn::{Error, ItemEnum, Result};

pub fn expand(mut item: ItemEnum) -> Result<TokenStream2> {
    let mut acc: Option<Error> = None;

    let (impl_generics, ty_generics, where_clause) = item.generics.split_for_impl();
    let item_ident = &item.ident;

    let mut variant_idents = Vec::new();
    let mut variant_tys = Vec::new();

    for variant in &item.variants {
        let fields = &variant.fields;
        match fields {
            syn::Fields::Unnamed(fields) => {
                if fields.unnamed.len() != 1 {
                    combine_err(
                        &mut acc,
                        Error::new_spanned(
                            &variant,
                            "enum variant must have exactly one type parameter",
                        ),
                    );
                    continue;
                }
                variant_idents.push(variant.ident.clone());
                variant_tys.push(fields.unnamed.first().unwrap().ty.clone());
            }
            _ => {
                combine_err(
                    &mut acc,
                    Error::new_spanned(
                        &variant,
                        "only tuple variants with a single type are supported for enum components",
                    ),
                );
            }
        }
    }

    if let Some(err) = acc {
        return Err(err);
    }

    let first_variant_ty = &variant_tys[0];

    for variant in item.variants.iter_mut() {
        if let syn::Fields::Unnamed(fields) = &mut variant.fields {
            if let Some(field) = fields.unnamed.first_mut() {
                let ty = &field.ty;
                field.ty = syn::parse_quote! {
                    <#ty as ::xdevs::simulation::SimpleSimulable>::Simulator
                };
            }
        }
    }

    let start_arms = variant_idents.iter().map(|ident| {
        quote::quote! {
            #item_ident::#ident(inner) => ::xdevs::simulation::AbstractSimulator::start(inner, t_start)
        }
    });

    let stop_arms = variant_idents.iter().map(|ident| {
        quote::quote! {
            #item_ident::#ident(inner) => ::xdevs::simulation::AbstractSimulator::stop(inner)
        }
    });

    let lambda_arms = variant_idents.iter().map(|ident| {
        quote::quote! {
            #item_ident::#ident(inner) => ::xdevs::simulation::AbstractSimulator::lambda(inner, output, t)
        }
    });

    let delta_arms = variant_idents.iter().map(|ident| {
        quote::quote! {
            #item_ident::#ident(inner) => ::xdevs::simulation::AbstractSimulator::delta(inner, input, output, t)
        }
    });

    let expanded = quote::quote! {
        #item

        impl #impl_generics ::xdevs::Component for #item_ident #ty_generics #where_clause {
            type Kind = ::xdevs::ComponentsKind;
            type Input = <#first_variant_ty as ::xdevs::Component>::Input;
            type Output = <#first_variant_ty as ::xdevs::Component>::Output;
        }

        unsafe impl #impl_generics ::xdevs::simulation::AbstractSimulator for #item_ident #ty_generics #where_clause {
            type Input = <#first_variant_ty as ::xdevs::Component>::Input;
            type Output = <#first_variant_ty as ::xdevs::Component>::Output;

            #[inline(always)]
            fn start(&mut self, t_start: f64) -> f64 {
                match self {
                    #(#start_arms),*
                }
            }

            #[inline(always)]
            fn stop(&mut self) {
                match self {
                    #(#stop_arms),*
                }
            }

            #[inline(always)]
            fn lambda(&mut self, output: &mut Self::Output, t: f64) {
                match self {
                    #(#lambda_arms),*
                }
            }

            #[inline(always)]
            fn delta(&mut self, input: &mut Self::Input, output: &mut Self::Output, t: f64) -> f64 {
                match self {
                    #(#delta_arms),*
                }
            }
        }
    };

    Ok(expanded)
}
