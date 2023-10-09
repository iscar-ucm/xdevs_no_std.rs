use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use std::collections::HashSet;
use syn::{
    parse::{Parse, ParseStream},
    Error, Ident, Token,
};

#[derive(Debug)]
pub struct ICMeta {
    pub component_from: Ident,
    pub port_from: Ident,
    pub component_to: Ident,
    pub port_to: Ident,
}

impl Parse for ICMeta {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let component_from = input.parse::<Ident>()?;
        input.parse::<Token![.]>()?;
        let port_from = input.parse::<Ident>()?;
        input.parse::<Token![->]>()?;
        let component_to = input.parse::<Ident>()?;
        input.parse::<Token![.]>()?;
        let port_to = input.parse::<Ident>()?;

        Ok(Self {
            component_from,
            port_from,
            component_to,
            port_to,
        })
    }
}

#[derive(Debug, Default)]
pub struct ICsMeta(Vec<ICMeta>);

impl core::ops::Deref for ICsMeta {
    type Target = Vec<ICMeta>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Parse for ICsMeta {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut ics = Vec::new();

        let mut cache = HashSet::new();

        let content;
        syn::bracketed!(content in input);
        while !content.is_empty() {
            let ic = content.parse::<ICMeta>()?;
            // assert that the IC is unique
            let cache_key = (
                ic.component_from.to_string(),
                ic.port_from.to_string(),
                ic.component_to.to_string(),
                ic.port_to.to_string(),
            );
            if cache.contains(&cache_key) {
                return Err(Error::new(ic.component_to.span(), "duplicate IC"));
            } else {
                cache.insert(cache_key);
            }

            ics.push(ic);
            if !content.is_empty() {
                content.parse::<Token![,]>()?; // comma between meta arguments
            }
        }
        Ok(Self(ics))
    }
}

impl ICsMeta {
    pub fn quote(&self, component: &Ident) -> TokenStream2 {
        let ics = self.iter().map(|ic| {
            let component_from = &ic.component_from;
            let port_from = &ic.port_from;
            let component_to = &ic.component_to;
            let port_to = &ic.port_to;
            quote! {
                self.components.#component_to.component.input.#port_to.add_values(&self.components.#component_from.component.output.#port_from.get_values());
            }
        });
        quote! {
            impl #component {
                pub fn propagate_ic(&mut self) {
                    #(#ics;)*
                }
            }
        }
    }
}
