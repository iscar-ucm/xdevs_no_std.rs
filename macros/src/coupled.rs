use crate::combine_err;
use proc_macro2::{Span, TokenStream as TokenStream2};
use syn::{Error, FieldsNamed, Ident, ItemStruct, Result};

pub fn expand(mut item: ItemStruct) -> Result<TokenStream2> {
    let mut acc: Option<Error> = None;

    // Generate initial variables for code generation
    let (impl_generics, ty_generics, where_clause) = item.generics.split_for_impl();
    let ty_generics_turbofish = ty_generics.as_turbofish();
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
                    "only named fields are supported for coupled components",
                ),
            );
        }
    }

    if let Some(err) = acc {
        return Err(err);
    }

    // Generate the components struct (with recursive unwrapping)
    let components_struct = {
        let mut item = item.clone();
        item.ident = Ident::new(&format!("{}Components", item.ident), item.ident.span());
        for field in item.fields.iter_mut() {
            field.ty = wrap_processor(&field.ty);
        }
        item
    };

    // Generate the components input wrapper struct (wrapping the outermost type)
    let components_input_struct = {
        let mut item = item.clone();
        item.attrs = Vec::new();
        item.ident = Ident::new(&format!("{}ComponentsInput", item.ident), item.ident.span());
        for field in item.fields.iter_mut() {
            let original_ty = field.ty.clone();
            field.ty = syn::parse_quote! {
                <#original_ty as ::xdevs::Component>::Input
            };
        }
        item
    };

    // Generate the components output wrapper struct (wrapping the outermost type)
    let components_output_struct = {
        let mut item = item.clone();
        item.attrs = Vec::new();
        item.ident = Ident::new(
            &format!("{}ComponentsOutput", item.ident),
            item.ident.span(),
        );
        for field in item.fields.iter_mut() {
            let original_ty = field.ty.clone();
            field.ty = syn::parse_quote! {
                <#original_ty as ::xdevs::Component>::Output
            };
        }
        item
    };

    // Construct the initialization fields iteratively to handle arrays.
    let mut init_fields = Vec::new();
    for (ident, ty) in item_fields.iter().zip(item_tys.iter()) {
        let init_expr = build_init_expr(quote::quote!(#ident), ty, 0);
        init_fields.push(quote::quote! { #ident: #init_expr });
    }

    // Modify the original struct fields
    let components_ident = &components_struct.ident;
    let components_fields_idents: Vec<Ident> = components_struct
        .fields
        .iter()
        .filter_map(|field| field.ident.clone())
        .collect();
    let components_input_ident = &components_input_struct.ident;
    let components_output_ident = &components_output_struct.ident;

    let new_fields: FieldsNamed = syn::parse_quote! {
        {
            components: #components_ident #ty_generics,
            components_input: #components_input_ident #ty_generics,
            components_output: #components_output_ident #ty_generics,
        }
    };
    item.fields = syn::Fields::Named(new_fields);

    // Generate the expanded code
    let expanded = quote::quote! {
        /// Struct holding all inner components as fields.
        #components_struct
        impl #impl_generics ::xdevs::Component for #components_ident #ty_generics #where_clause {
            type Input = #components_input_ident #ty_generics;
            type Output = #components_output_ident #ty_generics;
            type Kind = ::xdevs::CoupledKind;
        }
        unsafe impl #impl_generics ::xdevs::processor::AsProcessor for #components_ident #ty_generics #where_clause {
            #[inline(always)]
            fn starts(&mut self, t_start: f64) -> f64 {
                let mut t_next = f64::INFINITY;
                #(t_next = f64::min(t_next, ::xdevs::processor::AsProcessor::starts(&mut self.#components_fields_idents, t_start));)*
                t_next
            }

            #[inline(always)]
            fn stops(&mut self) {
                #(::xdevs::processor::AsProcessor::stops(&mut self.#components_fields_idents);)*
            }

            #[inline(always)]
            fn lambdas(&mut self, output: &mut Self::Output, t: f64) {
                #(::xdevs::processor::AsProcessor::lambdas(&mut self.#components_fields_idents, &mut output.#components_fields_idents, t);)*

            }

            #[inline(always)]
            fn deltas(&mut self, input: &mut Self::Input, output: &mut Self::Output, t: f64) -> f64 {
                let mut t_next = f64::INFINITY;
                #(t_next = f64::min(t_next, ::xdevs::processor::AsProcessor::deltas(
                        &mut self.#components_fields_idents,
                        &mut input.#components_fields_idents,
                        &mut output.#components_fields_idents,
                         t));)*
                t_next
            }
        }

        /// Wrapper struct holding mutable references to all inner components' inputs.
        #[derive(::xdevs::Bag)]
        #components_input_struct

        /// Wrapper struct holding references to all inner components' outputs.
        #[derive(::xdevs::Bag)]
        #components_output_struct

        /// Original model struct with added fields for inner components, their inputs and outputs, and simulation time.
        #item

        impl #impl_generics #item_ident #ty_generics #where_clause {
            #[inline]
            pub fn build(#(#item_fields: #item_tys),*) -> Self {
                Self {
                    components_input: <#components_input_ident #ty_generics as ::xdevs::port::Bag>::build(),
                    components_output: <#components_output_ident #ty_generics as ::xdevs::port::Bag>::build(),
                    components: #components_ident #ty_generics_turbofish {
                        #(#init_fields),*
                    },
                }
            }
        }

        unsafe impl #impl_generics ::xdevs::component::coupled::PartialCoupled for #item_ident #ty_generics #where_clause {
            type Components = #components_ident #ty_generics;
            type ComponentsInput = #components_input_ident #ty_generics;
            type ComponentsOutput = #components_output_ident #ty_generics;

            #[inline]
            fn components(&mut self) -> &mut Self::Components {
                &mut self.components
            }
            #[inline]
            fn inputs(&mut self) -> &mut Self::ComponentsInput {
                &mut self.components_input
            }
            #[inline]
            fn outputs(&mut self) -> &mut Self::ComponentsOutput {
                &mut self.components_output
            }
            #[inline]
            fn split(
                &mut self,
            ) -> (
                &mut Self::Components,
                &mut Self::ComponentsInput,
                &mut Self::ComponentsOutput,
            ) {
                (&mut self.components, &mut self.components_input, &mut self.components_output)
            }
        }
    };
    Ok(expanded.into())
}

// Recursively wrap only the innermost type in Processor<T> for Components
fn wrap_processor(ty: &syn::Type) -> syn::Type {
    match ty {
        syn::Type::Array(arr) => {
            let mut new_arr = arr.clone();
            *new_arr.elem = wrap_processor(&arr.elem);
            syn::Type::Array(new_arr)
        }
        _ => syn::parse_quote! { ::xdevs::processor::Processor<#ty> },
    }
}

// Generate nested .map(|x| ...) closure initializations for N-dimensional arrays
fn build_init_expr(expr: TokenStream2, ty: &syn::Type, level: usize) -> TokenStream2 {
    match ty {
        syn::Type::Array(arr) => {
            let var_name = Ident::new(&format!("x_{}", level), Span::call_site());
            let inner_expr = build_init_expr(quote::quote!(#var_name), &arr.elem, level + 1);
            quote::quote! {
                #expr.map(|#var_name| #inner_expr)
            }
        }
        _ => quote::quote! {
            ::xdevs::processor::Processor::new(#expr)
        },
    }
}
