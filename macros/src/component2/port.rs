use proc_macro2::Ident;
use proc_macro2::TokenStream as TokenStream2;
use syn::Generics;
use syn::TypeGenerics;

use super::Field;

pub struct Ports {
    pub ports: Vec<Field>,
    pub generics: Generics,
}

impl Ports {
    pub fn new(ports: Vec<Field>, generics: Generics) -> Self {
        Ports { ports, generics }
    }

    pub fn get_generics(&self) -> TypeGenerics<'_> {
        let (_, ty_generics, _) = self.generics.split_for_impl();
        ty_generics
    }

    pub fn quote(&self, ident: &Ident) -> TokenStream2 {
        let ports_ident: Vec<_> = self.ports.iter().map(|f| &f.ident).collect();
        let ports_ty: Vec<_> = self.ports.iter().map(|f| &f.ty).collect();
        let (impl_generics, ty_generics, _) = self.generics.split_for_impl();

        quote::quote! {
            #[derive(Debug, Default)]
            pub struct #ident #impl_generics {
                #(pub #ports_ident: #ports_ty,)*
            }
            impl #impl_generics #ident #ty_generics {
                #[inline]
                pub const fn new() -> Self {
                    Self { #(#ports_ident: xdevs::port::Port::new()),* }
                }
            }
            unsafe impl #impl_generics xdevs::traits::Bag for #ident #ty_generics{
                #[inline]
                fn is_empty(&self) -> bool {
                    true #( && self.#ports_ident.is_empty() )*
                }
                #[inline]
                fn clear(&mut self) {
                    #( self.#ports_ident.clear(); )*
                }
            }
        }
    }
}
