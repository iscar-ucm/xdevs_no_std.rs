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

impl PortMeta {
    pub(crate) fn quote(&self) -> TokenStream2 {
        let name = &self.name;
        let type_ = &self.ty;
        let capacity = &self.capacity;

        quote::quote! {
            pub #name: xdevs::port::Port<#type_, #capacity>
        }
    }
}

#[derive(Debug, Default)]
pub(crate) struct PortsMeta(Vec<PortMeta>);

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
    pub(crate) fn quote_struct(&self, ports_name: &syn::Ident) -> TokenStream2 {
        let struct_ports = self
            .0
            .iter()
            .map(|p| p.quote())
            .collect::<Vec<TokenStream2>>();

        quote::quote! {
            #[derive(Debug, Default)]
            pub struct #ports_name {
                #(#struct_ports),*
            }
        }
    }

    pub(crate) fn quote_impl(&self, ports_name: &syn::Ident) -> TokenStream2 {
        let ports_names = self.0.iter().map(|p| &p.name).collect::<Vec<_>>();

        quote::quote! {
            impl #ports_name {
                pub const fn new() -> Self {
                    Self {
                        #(#ports_names: xdevs::port::Port::new()),*
                    }
                }
            }

            unsafe impl xdevs::port::UnsafePort for #ports_name {
                #[inline]
                fn is_empty(&self) -> bool {
                    #(
                        self.#ports_names.is_empty()
                    )&& *
                }

                #[inline]
                fn clear(&mut self) {
                    #(
                        self.#ports_names.clear();
                    )*
                }
            }
        }
    }

    pub(crate) fn quote(&self, ports_name: &syn::Ident) -> TokenStream2 {
        let ports_struct = self.quote_struct(ports_name);
        let ports_impl = self.quote_impl(ports_name);

        quote::quote! {
            #ports_struct
            #ports_impl
        }
    }
}
