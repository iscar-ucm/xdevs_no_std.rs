use proc_macro2::TokenStream as TokenStream2;
use std::fmt;
use syn::parse::{Parse, ParseStream};
use syn::{Ident, LitInt, Token, TypePath};

pub(crate) struct PortMeta {
    pub(crate) name: Ident,
    pub(crate) ty: TypePath,
    pub(crate) capacity: LitInt,
}

impl fmt::Debug for PortMeta {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PortMeta")
            .field("name", &self.name.to_string())
            .field(
                "ty",
                &format!("{:?}", self.ty.path.get_ident().unwrap().to_string()),
            )
            .field("capacity", &format!("{:?}", self.capacity.base10_digits()))
            .finish()
    }
}

impl From<&PortMeta> for TokenStream2 {
    fn from(value: &PortMeta) -> Self {
        let name = &value.name;
        let type_ = &value.ty;
        let capacity = &value.capacity;

        quote::quote! {
            pub #name: xdevs::port::Port<#type_, #capacity>
        }
    }
}

impl Parse for PortMeta {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let name: syn::Ident = input.parse()?;
        input.parse::<Token![<]>()?;
        let ty: TypePath = input.parse()?;
        input.parse::<Token![,]>()?;
        let capacity: LitInt = input.parse()?;
        input.parse::<Token![>]>()?;

        Ok(PortMeta { name, ty, capacity })
    }
}

#[derive(Debug, Default)]
pub(crate) struct PortsMeta(Vec<PortMeta>);

impl From<&PortsMeta> for TokenStream2 {
    fn from(value: &PortsMeta) -> Self {
        let ports = &value
            .0
            .iter()
            .map(|p| p.into())
            .collect::<Vec<TokenStream2>>();

        quote::quote! {
            #(#ports),*
        }
    }
}

impl Parse for PortsMeta {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut ports = Vec::new();

        let ports_buffer;
        syn::bracketed!(ports_buffer in input);
        while !ports_buffer.is_empty() {
            ports.push(ports_buffer.parse::<PortMeta>()?);
            if !ports_buffer.is_empty() {
                ports_buffer.parse::<Token![,]>()?; // comma between ports
            }
        }

        Ok(Self(ports))
    }
}

impl PortsMeta {
    pub(crate) fn quote(&self, ports_name: &syn::Ident) -> TokenStream2 {
        let struct_ports = self
            .0
            .iter()
            .map(|p| p.into())
            .collect::<Vec<TokenStream2>>();

        let ports_names = self.0.iter().map(|p| &p.name).collect::<Vec<_>>();

        quote::quote! {
            pub struct #ports_name {
                #(#struct_ports),*
            }

            impl #ports_name {
                pub const fn new() -> Self {
                    Self {
                        #(#ports_names: xdevs::port::Port::new()),*
                    }
                }

                pub fn is_empty(&self) -> bool {
                    #(
                        self.#ports_names.is_empty()
                    )&& *
                }

                pub fn clear(&mut self) {
                    #(
                        self.#ports_names.clear();
                    )*
                }
            }
        }
    }
}
