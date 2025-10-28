use proc_macro2::Ident;
use proc_macro2::TokenStream as TokenStream2;

use super::Field;

pub struct Components {
    pub components: Vec<Field>,
}

impl Components {
    pub fn new(components: Vec<Field>) -> Self {
        Components { components }
    }

    pub fn add_component(&mut self, component: Field) {
        self.components.push(component);
    }

    pub fn is_empty(&self) -> bool {
        self.components.is_empty()
    }

    pub fn field_idents(&self) -> Vec<&syn::Ident> {
        self.components.iter().map(|f| &f.ident).collect()
    }

    pub fn field_tys(&self) -> Vec<&syn::Type> {
        self.components.iter().map(|f| &f.ty).collect()
    }

    pub fn quote(&self, ident: &Ident) -> TokenStream2 {
        let fields_ident = self.field_idents();
        let fields_ty = self.field_tys();

        quote::quote! {
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
