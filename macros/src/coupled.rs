use crate::combine_err;
use proc_macro2::TokenStream as TokenStream2;
use syn::{Error, FieldsNamed, Ident, ItemStruct, Result};

use super::to_component;

pub fn expand(mut item: ItemStruct) -> Result<TokenStream2> {
    let mut acc: Option<Error> = None;

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

    // Generate the raw components struct (fields keep their original types).
    // to_component will convert them to Simulator types and add Component + AbstractSimulator impls.
    let components_ident =
        Ident::new(&format!("{}Components", item_ident), item_ident.span());
    let raw_components = {
        let mut item = item.clone();
        item.ident = components_ident.clone();
        item
    };

    // Delegate to to_component::expand_struct for type conversions and trait impls.
    let component_tokens = to_component::expand_struct(raw_components)?;

    // Construct the initialization fields.
    let mut init_fields = Vec::new();
    for (ident, ty) in item_fields.iter().zip(item_tys.iter()) {
        let init_expr = quote::quote! {
            <#ty as ::xdevs::simulation::SimpleSimulable>::to_simulator(#ident)
        };
        init_fields.push(quote::quote! { #ident: #init_expr });
    }

    // Modify the original struct to hold the components.
    let new_fields: FieldsNamed = syn::parse_quote! {
        {
            components: #components_ident #ty_generics,
        }
    };
    item.fields = syn::Fields::Named(new_fields);

    let expanded = quote::quote! {
        #component_tokens

        /// Original model struct with a field that holds the inner components.
        #item

        impl #impl_generics #item_ident #ty_generics #where_clause {
            #[inline]
            pub fn build(#(#item_fields: #item_tys),*) -> Self {
                Self {
                    components: #components_ident #ty_generics_turbofish {
                        #(#init_fields),*
                    },
                }
            }
        }

        impl #impl_generics ::xdevs::component::coupled::PartialCoupled for #item_ident #ty_generics #where_clause {
            type Components = #components_ident #ty_generics;

            fn get_components(&self) -> &::xdevs::component::coupled::Components<Self> {
                &self.components
            }

            fn get_components_mut(&mut self) -> &mut ::xdevs::component::coupled::Components<Self> {
                &mut self.components
            }
        }
    };
    Ok(expanded.into())
}
