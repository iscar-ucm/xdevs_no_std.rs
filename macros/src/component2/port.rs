use proc_macro2::TokenStream as TokenStream2;
use syn::{Expr, ExprLit, Generics, Ident, Lit, PathArguments, Type};

use super::ComponentField;

pub struct Ports {
    pub ports: Vec<ComponentField>,
    pub ident: Ident,
    pub generics: Generics,
}

impl Ports {
    pub fn new(ports: Vec<ComponentField>, ident: Ident, generics: Generics) -> Self {
        Ports {
            ports,
            ident,
            generics,
        }
    }

    fn field_idents(&self) -> Vec<&Ident> {
        self.ports.iter().map(|f| &f.ident).collect()
    }

    fn field_tys(&self) -> Vec<&Type> {
        self.ports.iter().map(|f| &f.ty).collect()
    }

    fn generate_news(&self, ports: &Vec<ComponentField>) -> Vec<TokenStream2> {
        let mut news: Vec<TokenStream2> = Vec::new();
        for port in ports {
            let port_ident = &port.ident;
            let token = extract_new(&port.ty);
            let new = quote::quote! {
                #port_ident: #token
            };
            news.push(new);
        }
        news
    }

    pub fn quote(&self, is_bagmux: bool) -> TokenStream2 {
        let ident = &self.ident;
        let ports_ident = self.field_idents();
        let ports_ty = self.field_tys();
        let (impl_generics, ty_generics, where_clause) = self.generics.split_for_impl();
        let new_fn = self.generate_news(&self.ports);
        let bagmux = if is_bagmux {
            quote::quote! {
                , ::xdevs::BagMux
            }
        } else {
            TokenStream2::new()
        };

        quote::quote! {
            #[derive(Debug, Default, ::xdevs::Bag #bagmux)]
            pub struct #ident #impl_generics #where_clause {
                #(pub #ports_ident: #ports_ty,)*
            }
            impl #impl_generics #ident #ty_generics {
                #[inline]
                pub const fn new() -> Self {
                    Self { #(#new_fn),* }
                }
            }
        }
    }
}

fn extract_new(ty: &Type) -> TokenStream2 {
    let token: TokenStream2 = match ty {
        Type::Array(array) => {
            let token = extract_new(&array.elem);
            let length = &array.len;

            // Try to parse length as a literal integer
            if let Expr::Lit(ExprLit {
                lit: Lit::Int(lit_int),
                ..
            }) = length
            {
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
                segment.arguments = PathArguments::None;
            }
            quote::quote! {
                #path::new()
            }
        }
        &_ => unimplemented!(),
    };
    token
}
