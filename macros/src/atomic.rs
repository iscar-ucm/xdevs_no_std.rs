use crate::component::ComponentMeta;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    Ident, LitBool, Token, TypePath,
};

pub(crate) struct AtomicMeta {
    pub(crate) component: ComponentMeta,
    pub(crate) state: TypePath,
    pub(crate) constant: bool,
}

impl std::fmt::Debug for AtomicMeta {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AtomicMeta")
            .field("component", &self.component)
            .field(
                "state",
                &format!("{:?}", self.state.path.get_ident().unwrap().to_string()),
            )
            .field("constant", &self.constant)
            .finish()
    }
}

impl Parse for AtomicMeta {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut component = None;
        let mut state = None;
        let mut constant = None;

        while !input.is_empty() {
            let token: syn::Ident = input.parse()?;
            if token == "component" {
                if component.is_some() {
                    return Err(syn::Error::new(
                        token.span(),
                        "duplicate atomic meta argument",
                    ));
                }
                input.parse::<Token![=]>()?; // consume the '='
                let content;
                syn::braced!(content in input);
                component = Some(content.parse::<ComponentMeta>()?);
            } else if token == "state" {
                if state.is_some() {
                    return Err(syn::Error::new(
                        token.span(),
                        "duplicate atomic meta argument",
                    ));
                }
                input.parse::<Token![=]>()?; // consume the '='
                state = Some(input.parse::<TypePath>()?);
            } else if token == "constant" {
                if constant.is_some() {
                    return Err(syn::Error::new(
                        token.span(),
                        "duplicate atomic meta argument",
                    ));
                }
                input.parse::<Token![=]>()?; // consume the '='
                constant = Some(input.parse::<LitBool>()?.value);
            } else {
                return Err(syn::Error::new(
                    token.span(),
                    "unknown atomic meta argument",
                ));
            }
            if !input.is_empty() {
                input.parse::<Token![,]>()?; // comma between meta arguments
            }
        }

        if component.is_none() {
            return Err(syn::Error::new(
                input.span(),
                "atomic component not specified",
            ));
        }
        if state.is_none() {
            return Err(syn::Error::new(input.span(), "atomic state not specified"));
        }

        Ok(Self {
            component: component.unwrap(),
            state: state.unwrap(),
            constant: constant.unwrap_or(false),
        })
    }
}

impl AtomicMeta {
    pub(crate) fn atomic_ident(&self) -> Ident {
        self.component.name.clone()
    }

    pub(crate) fn quote_struct(&self) -> TokenStream2 {
        let atomic_ident = self.atomic_ident();
        let state_ident = self.state.path.get_ident().unwrap();
        let component_ident = self.component.component_ident();

        quote! {
            pub struct #atomic_ident {
                pub state: #state_ident,
                pub component: #component_ident,
            }
        }
    }

    pub(crate) fn quote_impl(&self) -> TokenStream2 {
        let atomic_ident = self.atomic_ident();
        let state_ident = self.state.path.get_ident().unwrap();
        let component_ident = self.component.component_ident();

        if self.constant {
            quote! {
                impl #atomic_ident {
                    pub const fn new(state: #state_ident) -> Self {
                        Self {
                            state,
                            component: #component_ident::new()
                        }
                    }
                }
            }
        } else {
            quote! {
                impl #atomic_ident {
                    pub fn new(state: #state_ident) -> Self {
                        Self {
                            state,
                            component: #component_ident::new()
                        }
                    }
                }
            }
        }
    }

    pub(crate) fn quote(&self) -> TokenStream2 {
        let component = self.component.quote();
        let atomic_struct = self.quote_struct();
        let atomic_impl = self.quote_impl();

        quote! {
            #component
            #atomic_struct
            #atomic_impl
        }
    }
}
