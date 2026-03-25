use proc_macro2::Ident;
use proc_macro2::TokenStream as TokenStream2;
use syn::Generics;

use super::ComponentField;

pub struct State {
    fields: Vec<ComponentField>,
    ident: Ident,
    generics: Generics,
}

impl State {
    pub fn new(fields: Vec<ComponentField>, ident: Ident, generics: Generics) -> Self {
        State {
            fields,
            ident,
            generics,
        }
    }

    pub fn ident(&self) -> &Ident {
        &self.ident
    }

    pub fn generics(&self) -> &Generics {
        &self.generics
    }

    pub fn field_idents(&self) -> Vec<&syn::Ident> {
        self.fields.iter().map(|f| &f.ident).collect()
    }

    pub fn field_tys(&self) -> Vec<&syn::Type> {
        self.fields.iter().map(|f| &f.ty).collect()
    }

    pub fn quote(&self) -> TokenStream2 {
        let ident = &self.ident;
        let fields_ident = self.field_idents();
        let fields_ty = self.field_tys();
        let (impl_generics, ty_generics, where_clause) = self.generics.split_for_impl();

        quote::quote! {
            #[derive(Debug)]
            pub struct #ident #impl_generics #where_clause {
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
