use crate::port::PortsMeta;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use std::collections::HashSet;
use syn::{
    parse::{Parse, ParseStream},
    Error, Ident, Token,
};

#[derive(Debug)]
pub(crate) struct ComponentMeta {
    pub(crate) name: Ident,
    pub(crate) input: PortsMeta,
    pub(crate) output: PortsMeta,
}

impl Parse for ComponentMeta {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut name = None;
        let mut input_ports = None;
        let mut output_ports = None;

        let mut cache = HashSet::new();

        while !input.is_empty() {
            let token: syn::Ident = input.parse()?;
            // assert that the token has not been parsed before
            if cache.contains(&token.to_string()) {
                return Err(Error::new(
                    token.span(),
                    "duplicate component meta argument",
                ));
            } else {
                cache.insert(token.to_string());
            }
            input.parse::<Token![=]>()?; // consume the '='

            if token == "name" {
                name = Some(input.parse::<Ident>()?);
            } else if token == "input" {
                input_ports = Some(input.parse::<PortsMeta>()?);
            } else if token == "output" {
                output_ports = Some(input.parse::<PortsMeta>()?);
            } else {
                return Err(Error::new(token.span(), "unknown component meta argument"));
            }
            if !input.is_empty() {
                input.parse::<Token![,]>()?; // comma between meta arguments
            }
        }

        if name.is_none() {
            return Err(Error::new(input.span(), "component name not specified"));
        }

        Ok(Self {
            name: name.unwrap(),
            input: input_ports.unwrap_or_default(),
            output: output_ports.unwrap_or_default(),
        })
    }
}

impl ComponentMeta {
    pub(crate) fn input_ident(&self) -> Ident {
        let name = format!("{name}Inputs", name = self.name);
        syn::Ident::new(&name, self.name.span())
    }

    pub(crate) fn output_ident(&self) -> Ident {
        let name = format!("{name}Outputs", name = self.name);
        syn::Ident::new(&name, self.name.span())
    }

    pub(crate) fn component_ident(&self) -> Ident {
        let name = format!("{name}Component", name = self.name);
        syn::Ident::new(&name, self.name.span())
    }

    pub(crate) fn quote_ports(&self) -> TokenStream2 {
        let input_ident = self.input_ident();
        let output_ident = self.output_ident();

        let input_ports = self.input.quote(&input_ident);
        let output_ports = self.output.quote(&output_ident);

        quote! {
            #input_ports
            #output_ports
        }
    }

    pub(crate) fn quote(&self) -> TokenStream2 {
        let component_ident = self.component_ident();
        let input_ident = self.input_ident();
        let output_ident = self.output_ident();

        let ports = self.quote_ports();

        quote! {
            #ports
            pub type #component_ident = xdevs::component::Component<#input_ident, #output_ident>;
        }
    }
}
