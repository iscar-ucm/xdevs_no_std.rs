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
        syn::braced!(content in input);
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
        syn::braced!(content in input);
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

#[derive(Debug)]
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
        syn::braced!(content in input);
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
