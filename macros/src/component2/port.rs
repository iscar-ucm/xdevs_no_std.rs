use proc_macro2::Ident;
use proc_macro2::TokenStream as TokenStream2;

use super::Field;

pub struct Ports {
    pub ports: Vec<Field>,
}

impl Ports {
    pub fn new(ports: Vec<Field>) -> Self {
        Ports { ports }
    }

    pub fn add_port(&mut self, field: Field) {
        self.ports.push(field);
    }

    pub fn quote(&self, ident: &Ident) -> TokenStream2 {
        let ports_ident: Vec<_> = self.ports.iter().map(|f| &f.ident).collect();
        let ports_ty: Vec<_> = self.ports.iter().map(|f| &f.ty).collect();

        quote::quote! {
            #[derive(Debug, Default)]
            pub struct #ident {
                #(pub #ports_ident: #ports_ty,)*
            }
            impl #ident {
                #[inline]
                pub const fn new() -> Self {
                    Self { #(#ports_ident: xdevs::port::Port::new()),* }
                }
            }
            unsafe impl xdevs::traits::Bag for #ident {
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
