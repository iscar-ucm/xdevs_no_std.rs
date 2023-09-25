use crate::port::PortsMeta;
use syn::parse::{Parse, ParseStream};
use syn::Token;

#[derive(Debug)]
pub(crate) struct ComponentMeta {
    pub(crate) input: PortsMeta,
    pub(crate) output: PortsMeta,
}

impl Parse for ComponentMeta {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut input_ports = None;
        let mut output_ports = None;

        while !input.is_empty() {
            let name: syn::Ident = input.parse()?;
            if name == "input" {
                if input_ports.is_some() {
                    return Err(syn::Error::new(
                        name.span(),
                        "duplicate component meta argument",
                    ));
                }
                input.parse::<Token![=]>()?; // consume the '='
                input_ports = Some(input.parse::<PortsMeta>()?);
            } else if name == "output" {
                if output_ports.is_some() {
                    return Err(syn::Error::new(
                        name.span(),
                        "duplicate component meta argument",
                    ));
                }
                input.parse::<Token![=]>()?; // consume the '='
                output_ports = Some(input.parse::<PortsMeta>()?);
            } else {
                return Err(syn::Error::new(
                    name.span(),
                    "unknown component meta argument",
                ));
            }
            if !input.is_empty() {
                input.parse::<Token![,]>()?; // comma between meta arguments
            }
        }

        Ok(Self {
            input: input_ports.unwrap_or_default(),
            output: output_ports.unwrap_or_default(),
        })
    }
}
