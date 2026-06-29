use crate::combine_err;
use proc_macro2::TokenStream as TokenStream2;
use syn::{Error, Ident, ItemEnum, ItemStruct, Result};

pub fn expand_struct(mut item: ItemStruct) -> Result<TokenStream2> {
    let mut acc: Option<Error> = None;

    let (impl_generics, ty_generics, where_clause) = item.generics.split_for_impl();
    let item_ident = &item.ident;

    let mut item_fields = Vec::new();
    let mut item_tys = Vec::new();

    match &item.fields {
        syn::Fields::Named(fields) => {
            for field in &fields.named {
                let Some(field_ident) = &field.ident else {
                    combine_err(&mut acc, Error::new_spanned(field, "expected named field"));
                    continue;
                };
                item_fields.push(field_ident.clone());
                item_tys.push(field.ty.clone());
            }
        }
        _ => {
            combine_err(
                &mut acc,
                Error::new_spanned(
                    &item.fields,
                    "only named fields are supported",
                ),
            );
        }
    }

    if let Some(err) = acc {
        return Err(err);
    }

    // Generate the input wrapper struct
    let item_input_ident =
        Ident::new(&format!("{}Input", item_ident), item_ident.span());
    let input_struct = {
        let mut s = item.clone();
        s.attrs = Vec::new();
        s.ident = item_input_ident.clone();
        if let syn::Fields::Named(fields) = &mut s.fields {
            for field in &mut fields.named {
                let original_ty = field.ty.clone();
                field.ty = syn::parse_quote! {
                    <#original_ty as ::xdevs::Component>::Input
                };
            }
        }
        s
    };

    // Generate the output wrapper struct
    let item_output_ident =
        Ident::new(&format!("{}Output", item_ident), item_ident.span());
    let output_struct = {
        let mut s = item.clone();
        s.attrs = Vec::new();
        s.ident = item_output_ident.clone();
        if let syn::Fields::Named(fields) = &mut s.fields {
            for field in &mut fields.named {
                let original_ty = field.ty.clone();
                field.ty = syn::parse_quote! {
                    <#original_ty as ::xdevs::Component>::Output
                };
            }
        }
        s
    };

    // Convert the struct's own fields to Simulator types
    if let syn::Fields::Named(fields) = &mut item.fields {
        for field in &mut fields.named {
            let ty = &field.ty;
            field.ty = syn::parse_quote! {
                <#ty as ::xdevs::simulation::SimpleSimulable>::Simulator
            };
        }
    }

    let expanded = quote::quote! {
        #[derive(::xdevs::Bag)]
        #input_struct

        #[derive(::xdevs::Bag)]
        #output_struct

        #item

        impl #impl_generics ::xdevs::Component for #item_ident #ty_generics #where_clause {
            type Kind = ::xdevs::ComponentsKind;
            type Input = #item_input_ident #ty_generics;
            type Output = #item_output_ident #ty_generics;
        }

        unsafe impl #impl_generics ::xdevs::simulation::AbstractSimulator for #item_ident #ty_generics #where_clause {
            type Input = <Self as ::xdevs::Component>::Input;
            type Output = <Self as ::xdevs::Component>::Output;

            #[inline(always)]
            fn start(&mut self, t_start: f64) -> f64 {
                let mut t_next = f64::INFINITY;
                #(t_next = f64::min(t_next, ::xdevs::simulation::AbstractSimulator::start(&mut self.#item_fields, t_start));)*
                t_next
            }

            #[inline(always)]
            fn stop(&mut self) {
                #(::xdevs::simulation::AbstractSimulator::stop(&mut self.#item_fields);)*
            }

            #[inline(always)]
            fn lambda(&mut self, output: &mut Self::Output, t: f64) {
                #(::xdevs::simulation::AbstractSimulator::lambda(&mut self.#item_fields, &mut output.#item_fields, t);)*
            }

            #[inline(always)]
            fn delta(&mut self, input: &mut Self::Input, output: &mut Self::Output, t: f64) -> f64 {
                let mut t_next = f64::INFINITY;
                #(t_next = f64::min(t_next, ::xdevs::simulation::AbstractSimulator::delta(
                        &mut self.#item_fields,
                        &mut input.#item_fields,
                        &mut output.#item_fields,
                        t));)*
                t_next
            }
        }
    };

    Ok(expanded)
}
pub fn expand_enum(mut item: ItemEnum) -> Result<TokenStream2> {
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

    if variant_tys.is_empty() {
        return Err(Error::new_spanned(
            &item.ident,
            "enum component must have at least one variant",
        ));
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
