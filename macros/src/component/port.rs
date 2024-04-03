use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::quote;
use std::collections::HashSet;
use syn::parse::{Parse, ParseStream};
use syn::{braced, Error, Ident, LitInt, Token, Type};

pub struct Port {
    pub ident: Ident,
    pub ty: Type,
    pub capacity: LitInt,
    rangle: Token![>],
}

impl Port {
    pub fn arg(&self) -> TokenStream2 {
        let ident = &self.ident;
        let ty = &self.ty;
        let capacity = &self.capacity;

        quote! { #ident: xdevs::port::Port<#ty, #capacity> }
    }

    pub fn span(&self) -> Span {
        let start = self.ident.span();
        let end = self.rangle.span;

        start.join(end).unwrap_or_else(|| start)
    }
}

impl Parse for Port {
    /// Port format is expected to look like name<type,capacity> or name<type>.
    /// If capacity is not specified, it defaults to 1.
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let ident = input.parse()?;
        input.parse::<Token![<]>()?;
        let ty = input.parse()?;
        let capacity = match input.parse::<Token![,]>() {
            Ok(_) => input.parse()?,
            Err(_) => LitInt::new("1", input.span()), // default capacity is 1
        };
        let rangle = input.parse()?;

        Ok(Self {
            ident,
            ty,
            capacity,
            rangle,
        })
    }
}

#[derive(Default)]
pub struct Ports {
    braces: syn::token::Brace,
    pub ports: Vec<Port>,
}

impl Ports {
    pub fn span(&self) -> Span {
        self.braces.span.join()
    }

    pub fn quote(&self, ident: &Ident) -> TokenStream2 {
        let ports_ident: Vec<_> = self.ports.iter().map(|p| &p.ident).collect();
        let ports_arg: Vec<_> = self.ports.iter().map(|p| p.arg()).collect();

        quote! {
            #[derive(Debug, Default)]
            pub struct #ident {
                #(pub #ports_arg),*
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

impl Parse for Ports {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        let braces = braced!(content in input);
        let mut ports = Vec::new();
        let mut cache = HashSet::new();

        while !content.is_empty() {
            let port = content.parse::<Port>()?;
            if cache.contains(&port.ident) {
                return Err(Error::new(port.span(), "duplicate port name"));
            }
            cache.insert(port.ident.clone());

            ports.push(port);
            if !content.is_empty() {
                content.parse::<Token![,]>()?; // comma between ports
            }
        }

        Ok(Self { braces, ports })
    }
}
