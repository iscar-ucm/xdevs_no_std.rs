use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use std::collections::HashSet;
use syn::{
    parse::{Parse, ParseStream},
    token::Brace,
    Error, Ident, Result, Token, Type,
};

pub struct Component {
    pub ident: Ident,
    pub _colon: Token![:],
    pub ty: Type,
}

impl Parse for Component {
    fn parse(input: ParseStream) -> Result<Self> {
        let ident = input.parse()?;
        let colon = input.parse()?;
        let ty = input.parse()?;
        Ok(Self {
            ident,
            _colon: colon,
            ty,
        })
    }
}

pub struct Components {
    pub _brace: Brace,
    pub components: Vec<Component>,
}

impl Components {
    pub fn ident(&self) -> Vec<TokenStream2> {
        let mut res = Vec::new();
        for component in self.components.iter() {
            let ident = &component.ident;
            res.push(quote!(#ident));
        }
        res
    }

    pub fn ty(&self) -> Vec<TokenStream2> {
        let mut res = Vec::new();
        for component in self.components.iter() {
            let ident = &component.ident;
            let ty = &component.ty;
            res.push(quote!(#ident: #ty));
        }
        res
    }
}

impl Parse for Components {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;
        let brace = syn::braced!(content in input);
        let mut components = Vec::new();

        let mut cache = HashSet::new();
        while !content.is_empty() {
            let component = content.parse::<Component>()?;
            if cache.contains(&component.ident) {
                return Err(Error::new(
                    component.ident.span(),
                    "duplicate component ident",
                ));
            }
            cache.insert(component.ident.clone());

            components.push(component);
            if !content.is_empty() {
                content.parse::<Token![,]>()?; // comma between components
            }
        }

        Ok(Self {
            _brace: brace,
            components,
        })
    }
}
