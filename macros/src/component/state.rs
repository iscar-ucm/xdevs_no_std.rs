use proc_macro2::TokenStream as TokenStream2;
use syn::{Generics, Ident, Type};

use super::ComponentField;

/// Parsed component state fields used to generate the state wrapper type.
pub struct State {
    pub fields: Vec<ComponentField>,
    pub ident: Ident,
    pub generics: Generics,
}

impl State {
    pub fn new(fields: Vec<ComponentField>, ident: Ident, generics: Generics) -> Self {
        State {
            fields,
            ident,
            generics,
        }
    }

    pub fn field_vis(&self) -> Vec<&syn::Visibility> {
        self.fields.iter().map(|f| &f.vis).collect()
    }

    pub fn field_idents(&self) -> Vec<&Ident> {
        self.fields.iter().map(|f| &f.ident).collect()
    }

    pub fn field_tys(&self) -> Vec<&Type> {
        self.fields.iter().map(|f| &f.ty).collect()
    }

    pub fn quote(&self) -> TokenStream2 {
        let ident = &self.ident;
        let fields_vis = self.field_vis();
        let fields_ident = self.field_idents();
        let fields_ty = self.field_tys();
        let (impl_generics, ty_generics, where_clause) = self.generics.split_for_impl();

        // TODO determine what to do with state struct visibility
        quote::quote! {
            pub struct #ident #impl_generics #where_clause {
                #(#fields_vis #fields_ident: #fields_ty,)*
            }
            impl #impl_generics #ident #ty_generics #where_clause {
                #[inline]
                pub const fn new(#(#fields_ident: #fields_ty),*) -> Self {
                    Self { #(#fields_ident),* }
                }
            }
        }
    }
}
