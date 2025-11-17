use proc_macro2::Ident;
use proc_macro2::TokenStream as TokenStream2;
use syn::Generics;
use syn::Type;
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

    fn generate_news(&self, ports: &Vec<Field>) -> Vec<TokenStream2> {
        let mut news:Vec<TokenStream2> = Vec::new();
        for port in ports {
            let port_ident = &port.ident;
            let token = extract_new(&port.ty);
            let new = quote::quote!{
                #port_ident: #token
            };
            news.push(new);
        }
        news
    }

    pub fn quote(&self, ident: &Ident) -> TokenStream2 {
        let ports_ident: Vec<_> = self.ports.iter().map(|f| &f.ident).collect();
        let ports_ty: Vec<_> = self.ports.iter().map(|f| &f.ty).collect();
        let (impl_generics, ty_generics, _) = self.generics.split_for_impl();
        let new_fn = self.generate_news(&self.ports);

        quote::quote! {
            #[derive(Debug, Default)]
            pub struct #ident #impl_generics {
                #(pub #ports_ident: #ports_ty,)*
            }
            impl #impl_generics #ident #ty_generics {
                #[inline]
                pub const fn new() -> Self {
                    Self { #(#new_fn),* }
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

fn extract_new(ty: &Type) -> TokenStream2 {
    let token: TokenStream2 = match ty {
        Type::Array(array) => {
            let token = extract_new(&*array.elem);
            let length = &array.len;
            
            // Try to parse length as a literal integer
            if let syn::Expr::Lit(syn::ExprLit { lit: syn::Lit::Int(lit_int), .. }) = length {
                let n: usize = lit_int.base10_parse().unwrap();
                let repeated: Vec<_> = (0..n).map(|_| quote::quote! { #token }).collect();
                quote::quote! {
                    [ #( #repeated ),* ]
                }
            } else {
                quote::quote! {
                    compile_error!("Array length must be a literal integer for non-Copy types");
                }
            }
        }
        Type::Path(path) => {
            let mut path = path.path.clone();
            for segment in &mut path.segments {
                segment.arguments = syn::PathArguments::None;
            }
            quote::quote!{
                #path::new()
            }
        }
        &_ => unimplemented!()
    };
    token
}