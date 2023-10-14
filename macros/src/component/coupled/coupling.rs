use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use std::collections::HashSet;
use syn::{
    parse::{Parse, ParseStream},
    token::Brace,
    Error, Ident, Token,
};

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Coupling {
    pub component_from: Option<Ident>,
    pub port_from: Ident,
    pub component_to: Option<Ident>,
    pub port_to: Ident,
}

impl Coupling {
    pub fn is_eoc(&self) -> bool {
        self.component_to.is_none()
    }

    pub fn quote(&self) -> TokenStream2 {
        let port_from = &self.port_from;
        let port_to = &self.port_to;

        let origin = if let Some(component_from) = &self.component_from {
            quote!(self.#component_from.output.#port_from)
        } else {
            quote!(self.input.#port_from)
        };
        let destination = if let Some(component_to) = &self.component_to {
            quote!(self.#component_to.input.#port_to)
        } else {
            quote!(self.output.#port_to)
        };
        quote! {
            #destination.add_values(&#origin.get_values()).expect("unable to propagate messages; destination port is full");
        }
    }

    pub fn span(&self) -> proc_macro2::Span {
        let start = if let Some(component_from) = &self.component_from {
            component_from.span()
        } else {
            self.port_from.span()
        };
        let end = self.port_to.span();

        start.join(end).unwrap_or_else(|| start)
    }
}

impl Parse for Coupling {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let source_1: Ident = input.parse()?;
        if let Ok(_) = input.parse::<Token![.]>() {
            // this is an IC or EOC
            let source_2: Ident = input.parse()?;
            input.parse::<Token![->]>()?; // consume the '->'
            let destination_1: Ident = input.parse()?;
            if let Ok(_) = input.parse::<Token![.]>() {
                // this is an IC
                let destination_2: Ident = input.parse()?;
                return Ok(Self {
                    component_from: Some(source_1),
                    port_from: source_2,
                    component_to: Some(destination_1),
                    port_to: destination_2,
                });
            } else {
                // this is an EOC
                return Ok(Self {
                    component_from: Some(source_1),
                    port_from: source_2,
                    component_to: None,
                    port_to: destination_1,
                });
            }
        } else {
            // this is an EIC
            input.parse::<Token![->]>()?; // consume the '->'
            let destination_1: Ident = input.parse()?;
            input.parse::<Token![.]>()?; // consume the '.'
            let destination_2: Ident = input.parse()?;
            return Ok(Self {
                component_from: None,
                port_from: source_1,
                component_to: Some(destination_1),
                port_to: destination_2,
            });
        }
    }
}

pub struct Couplings {
    pub brace: Brace,
    pub couplings: Vec<Coupling>,
}

impl Couplings {
    pub fn quote(&self) -> (Vec<TokenStream2>, Vec<TokenStream2>) {
        let mut eoc = Vec::new();
        let mut xic = Vec::new();

        for coupling in &self.couplings {
            if coupling.is_eoc() {
                eoc.push(coupling.quote());
            } else {
                xic.push(coupling.quote());
            }
        }
        (eoc, xic)
    }
}

impl Parse for Couplings {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        let brace = syn::braced!(content in input);
        let mut couplings = Vec::new();
        let mut cache = HashSet::new();

        while !content.is_empty() {
            let coupling = content.parse::<Coupling>()?;
            if cache.contains(&coupling) {
                return Err(Error::new(coupling.span(), "duplicate coupling"));
            }
            cache.insert(coupling.clone());

            couplings.push(coupling);
            if !content.is_empty() {
                content.parse::<Token![,]>()?; // comma between meta arguments
            }
        }
        Ok(Self { brace, couplings })
    }
}
