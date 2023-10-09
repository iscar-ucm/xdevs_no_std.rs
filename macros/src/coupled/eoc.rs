use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use std::collections::HashSet;
use syn::{
    parse::{Parse, ParseStream},
    Error, Ident, Token,
};

#[derive(Debug)]
pub struct EOCMeta {
    pub component_from: Ident,
    pub port_from: Ident,
    pub port_to: Ident,
}

impl Parse for EOCMeta {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let component_from = input.parse::<Ident>()?;
        input.parse::<Token![.]>()?;
        let port_from = input.parse::<Ident>()?;
        input.parse::<Token![->]>()?;
        let port_to = input.parse::<Ident>()?;

        Ok(Self {
            component_from,
            port_from,
            port_to,
        })
    }
}

#[derive(Debug, Default)]
pub struct EOCsMeta(Vec<EOCMeta>);

impl core::ops::Deref for EOCsMeta {
    type Target = Vec<EOCMeta>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Parse for EOCsMeta {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut eocs = Vec::new();

        let mut cache = HashSet::new();

        let content;
        syn::bracketed!(content in input);
        while !content.is_empty() {
            let eoc = content.parse::<EOCMeta>()?;
            // assert that the EOC is unique
            let cache_key = (
                eoc.component_from.to_string(),
                eoc.port_from.to_string(),
                eoc.port_to.to_string(),
            );
            if cache.contains(&cache_key) {
                return Err(Error::new(eoc.port_to.span(), "duplicate EOC"));
            } else {
                cache.insert(cache_key);
            }

            eocs.push(eoc);
            if !content.is_empty() {
                content.parse::<Token![,]>()?; // comma between meta arguments
            }
        }
        Ok(Self(eocs))
    }
}

impl EOCsMeta {
    pub fn quote(&self, component: &Ident) -> TokenStream2 {
        let eocs = self.iter().map(|eoc| {
            let component_from = &eoc.component_from;
            let port_from = &eoc.port_from;
            let port_to = &eoc.port_to;
            quote! {
                self.component.output.#port_to.add_values(&self.components.#component_from.component.output.#port_from.get_values());
            }
        });
        quote! {
            impl #component {
                pub fn propagate_eoc(&mut self) {
                    #(#eocs;)*
                }
            }
        }
    }
}
