use proc_macro2::Ident;
use proc_macro2::TokenStream as TokenStream2;

use super::Field;

pub struct State {
    pub fields: Vec<Field>,
}

impl State {
    pub fn new(fields: Vec<Field>) -> Self {
        State { fields }
    }

    pub fn add_field(&mut self, field: Field) {
        self.fields.push(field);
    }

    pub fn is_empty(&self) -> bool {
        self.fields.is_empty()
    }

    pub fn field_idents(&self) -> Vec<&syn::Ident> {
        self.fields.iter().map(|f| &f.ident).collect()
    }

    pub fn field_tys(&self) -> Vec<&syn::Type> {
        self.fields.iter().map(|f| &f.ty).collect()
    }

    pub fn quote(&self, ident: &Ident) -> TokenStream2 {
        let fields_ident = self.field_idents();
        let fields_ty = self.field_tys();

        quote::quote! {
            #[derive(Debug, Default)]
            pub struct #ident {
                #(#fields_ident: #fields_ty,)*
            }
            impl #ident {
                #[inline]
                pub fn new(#(#fields_ident: #fields_ty),*) -> Self {
                    Self { #(#fields_ident),* }
                }
            }
        }
    }
}
