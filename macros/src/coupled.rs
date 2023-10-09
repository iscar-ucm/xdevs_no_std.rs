use crate::component::ComponentMeta;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use std::collections::HashSet;
use syn::{
    parse::{Parse, ParseStream},
    Error, Ident, LitBool, Token, TypePath,
};
mod eic;
mod eoc;
mod ic;

pub struct CoupledMeta {
    pub constant: bool,
    pub component: ComponentMeta,
    pub components: TypePath,
    pub eic: eic::EICsMeta,
    pub ic: ic::ICsMeta,
    pub eoc: eoc::EOCsMeta,
}

impl Parse for CoupledMeta {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut constant = false;
        let mut component = None;
        let mut components = None;
        let mut eic = None;
        let mut ic = None;
        let mut eoc = None;

        let mut cache: HashSet<String> = HashSet::new();

        while !input.is_empty() {
            let token: Ident = input.parse()?;
            // assert that the token has not been parsed before
            if cache.contains(&token.to_string()) {
                return Err(Error::new(token.span(), "duplicate meta argument"));
            } else {
                cache.insert(token.to_string());
            }
            input.parse::<Token![=]>()?; // consume the '='

            if token == "constant" {
                constant = input.parse::<LitBool>()?.value;
            } else if token == "component" {
                let content;
                syn::braced!(content in input);
                component = Some(content.parse::<ComponentMeta>()?);
            } else if token == "components" {
                components = Some(input.parse::<TypePath>()?);
            } else if token == "eic" {
                eic = Some(input.parse::<eic::EICsMeta>()?);
            } else if token == "ic" {
                ic = Some(input.parse::<ic::ICsMeta>()?);
            } else if token == "eoc" {
                eoc = Some(input.parse::<eoc::EOCsMeta>()?);
            } else {
                return Err(Error::new(token.span(), "unknown meta argument"));
            }
            if !input.is_empty() {
                input.parse::<Token![,]>()?; // comma between meta arguments
            }
        }

        if component.is_none() {
            return Err(Error::new(input.span(), "component not specified"));
        }
        if components.is_none() {
            return Err(Error::new(input.span(), "components not specified"));
        }

        Ok(Self {
            constant,
            component: component.unwrap(),
            components: components.unwrap(),
            eic: eic.unwrap_or_default(),
            ic: ic.unwrap_or_default(),
            eoc: eoc.unwrap_or_default(),
        })
    }
}

impl CoupledMeta {
    pub(crate) fn coupled_ident(&self) -> Ident {
        self.component.name.clone()
    }

    pub(crate) fn quote_struct(&self) -> TokenStream2 {
        let coupled_ident = self.coupled_ident();
        // let state_ident = self.state.path.get_ident().unwrap();
        let component_ident = self.component.component_ident();
        let components_ident = self.components.path.get_ident().unwrap();

        quote! {
            pub struct #coupled_ident {
                pub component: #component_ident,
                pub components: #components_ident,
            }
        }
    }

    pub fn quote(&self) -> TokenStream2 {
        let coupled_ident = self.coupled_ident();

        let component = self.component.quote();
        let coupled_struct = self.quote_struct();
        let eic = self.eic.quote(&coupled_ident);
        let ic = self.ic.quote(&coupled_ident);
        let eoc = self.eoc.quote(&coupled_ident);

        quote! {
            #component
            #coupled_struct
            #eic
            #ic
            #eoc
        }
    }
}
