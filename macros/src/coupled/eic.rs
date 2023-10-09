use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use std::collections::HashSet;
use syn::{
    parse::{Parse, ParseStream},
    Error, Ident, Token,
};

#[derive(Debug)]
pub struct EICMeta {
    pub port_from: Ident,
    pub component_to: Ident,
    pub port_to: Ident,
}

impl Parse for EICMeta {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let port_from = input.parse::<Ident>()?;
        input.parse::<Token![->]>()?;
        let component_to = input.parse::<Ident>()?;
        input.parse::<Token![.]>()?;
        let port_to = input.parse::<Ident>()?;

        Ok(Self {
            port_from,
            component_to,
            port_to,
        })
    }
}

#[derive(Debug, Default)]
pub struct EICsMeta(Vec<EICMeta>);

impl core::ops::Deref for EICsMeta {
    type Target = Vec<EICMeta>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Parse for EICsMeta {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut eics = Vec::new();

        let mut cache = HashSet::new();

        let content;
        syn::bracketed!(content in input);
        while !content.is_empty() {
            let eic = content.parse::<EICMeta>()?;
            // assert that the EIC is unique
            let cache_key = (
                eic.port_from.to_string(),
                eic.component_to.to_string(),
                eic.port_to.to_string(),
            );
            if cache.contains(&cache_key) {
                return Err(Error::new(eic.component_to.span(), "duplicate EIC"));
            } else {
                cache.insert(cache_key);
            }

            eics.push(eic);
            if !content.is_empty() {
                content.parse::<Token![,]>()?; // comma between meta arguments
            }
        }
        Ok(Self(eics))
    }
}

impl EICsMeta {
    pub fn quote(&self, component: &Ident) -> TokenStream2 {
        let eics = self.iter().map(|eic| {
            let port_from = &eic.port_from;
            let component_to = &eic.component_to;
            let port_to = &eic.port_to;
            quote! {
                self.components.#component_to.component.input.#port_to.add_values(&self.component.input.#port_from.get_values());
            }
        });
        quote! {
            impl #component {
                pub fn propagate_eic(&mut self) {
                    #(#eics;)*
                }
            }
        }
    }
}
