use super::Field;
use proc_macro2::Ident;
use proc_macro2::TokenStream as TokenStream2;
use syn::Generics;

pub struct Components {
    pub components: Vec<Field>,
    pub generics: Generics,
}

impl Components {
    pub fn new(components: Vec<Field>, generics: Generics) -> Self {
        Components {
            components,
            generics,
        }
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
        let (impl_generics, ty_generics, _) = self.generics.split_for_impl();

        quote::quote! {
            pub struct #ident #impl_generics{
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
