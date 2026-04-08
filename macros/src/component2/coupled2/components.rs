use crate::component2::ComponentField;
use proc_macro2::TokenStream as TokenStream2;
use syn::{Generics, Ident, Type};

/// Parsed inner component fields for coupled2 model generation.
pub struct Components {
    pub components: Vec<ComponentField>,
    pub ident: Ident,
    pub generics: Generics,
}

impl Components {
    pub fn new(components: Vec<ComponentField>, ident: Ident, generics: Generics) -> Self {
        Components {
            components,
            ident,
            generics,
        }
    }

    pub fn field_idents(&self) -> Vec<&Ident> {
        self.components.iter().map(|f| &f.ident).collect()
    }

    pub fn field_tys(&self) -> Vec<&Type> {
        self.components.iter().map(|f| &f.ty).collect()
    }

    pub fn quote(&self) -> TokenStream2 {
        let ident = &self.ident;
        let fields_ident = self.field_idents();
        let fields_ty = self.field_tys();
        let (impl_generics, ty_generics, where_clause) = self.generics.split_for_impl();

        quote::quote! {
            pub struct #ident #impl_generics #where_clause{
                #(#fields_ident: #fields_ty,)*
            }
            impl #impl_generics #ident #ty_generics{
                #[inline]
                pub fn new(#(#fields_ident: #fields_ty),*) -> Self {
                    Self { #(#fields_ident),* }
                }
            }
        }
    }
}
