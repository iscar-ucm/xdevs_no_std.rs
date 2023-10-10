use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use std::collections::HashSet;
use syn::parse::{Parse, ParseStream};
use syn::{Error, Ident, LitInt, Token, TypePath};

struct PortMeta {
    name: Ident,
    ty: TypePath,
    capacity: LitInt,
}

impl std::fmt::Debug for PortMeta {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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
    /// Port format is expected to look like `name<type,capacity>` or `name<type>`, where:
    /// - `name` is the name of the port.
    /// - `type` is the data type of the messages of the port.
    /// - `capacity` is the maximum number of messages that can be simultaneously stored in the port.
    ///
    /// If capacity is not specified, it defaults to 1.
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let name: syn::Ident = input.parse()?;
        input.parse::<Token![<]>()?; // consume the '<'
        let ty: TypePath = input.parse()?;
        let capacity = match input.parse::<Token![,]>() {
            Ok(_) => input.parse::<LitInt>()?,
            Err(_) => LitInt::new("1", input.span()), // default capacity is 1
        };
        input.parse::<Token![>]>()?; // consume the '>'

        Ok(PortMeta { name, ty, capacity })
    }
}

impl PortMeta {
    fn quote(&self) -> TokenStream2 {
        let name = &self.name;
        let ty = &self.ty;
        let capacity = &self.capacity;

        quote! {
            pub #name: xdevs::port::Port<#ty, #capacity>
        }
    }
}

#[derive(Debug, Default)]
pub struct PortsMeta(Vec<PortMeta>);

impl Parse for PortsMeta {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut ports = Vec::new();

        let mut cache = HashSet::new();

        let ports_buffer;
        syn::bracketed!(ports_buffer in input);
        while !ports_buffer.is_empty() {
            let port = ports_buffer.parse::<PortMeta>()?;
            // assert that the port is unique
            if cache.contains(&port.name) {
                return Err(Error::new(port.name.span(), "duplicate port name"));
            } else {
                cache.insert(port.name.clone());
            }

            ports.push(port);
            if !ports_buffer.is_empty() {
                ports_buffer.parse::<Token![,]>()?; // comma between ports
            }
        }

        Ok(Self(ports))
    }
}

impl PortsMeta {
    fn quote_struct(&self, ports_name: &syn::Ident) -> TokenStream2 {
        let struct_ports = self
            .0
            .iter()
            .map(|p| p.quote())
            .collect::<Vec<TokenStream2>>();

        quote! {
            #[derive(Debug, Default)]
            pub struct #ports_name {
                #(#struct_ports),*
            }
        }
    }

    fn quote_impl(&self, ports_name: &syn::Ident) -> TokenStream2 {
        let ports_names = self.0.iter().map(|p| &p.name).collect::<Vec<_>>();

        quote! {
            impl #ports_name {
                #[inline]
                pub const fn new() -> Self {
                    Self {
                        #(#ports_names: xdevs::port::Port::new()),*
                    }
                }
            }
        }
    }

    fn quote_impl_port(&self, ports_name: &syn::Ident) -> TokenStream2 {
        let ports_names = self.0.iter().map(|p| &p.name).collect::<Vec<_>>();

        if ports_names.is_empty() {
            quote! {
                unsafe impl xdevs::port::UnsafePort for #ports_name {
                    #[inline]
                    fn is_empty(&self) -> bool { true }

                    #[inline]
                    fn clear(&mut self) { }
                }
            }
        } else {
            quote! {
                unsafe impl xdevs::port::UnsafePort for #ports_name {
                    #[inline]
                    fn is_empty(&self) -> bool {
                        #(self.#ports_names.is_empty())&& *
                    }

                    #[inline]
                    fn clear(&mut self) {
                        #(self.#ports_names.clear();)*
                    }
                }
            }
        }
    }

    pub(crate) fn quote(&self, ports_name: &syn::Ident) -> TokenStream2 {
        let ports_struct = self.quote_struct(ports_name);
        let ports_impl = self.quote_impl(ports_name);
        let ports_impl_port = self.quote_impl_port(ports_name);

        quote! {
            #ports_struct
            #ports_impl
            #ports_impl_port
        }
    }
}
