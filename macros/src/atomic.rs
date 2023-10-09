use crate::component::ComponentMeta;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use std::collections::HashSet;
use syn::{
    parse::{Parse, ParseStream},
    Error, Ident, LitBool, Token, TypePath,
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
        let mut constant = false;

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

            if token == "component" {
                let content;
                syn::braced!(content in input);
                component = Some(content.parse::<ComponentMeta>()?);
            } else if token == "state" {
                state = Some(input.parse::<TypePath>()?);
            } else if token == "constant" {
                constant = input.parse::<LitBool>()?.value;
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
        if state.is_none() {
            return Err(Error::new(input.span(), "state not specified"));
        }

        Ok(Self {
            component: component.unwrap(),
            state: state.unwrap(),
            constant,
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
        let input_ident = self.component.input_ident();
        let output_ident = self.component.output_ident();

        if self.constant {
            quote! {
                impl #atomic_ident {
                    pub const fn new(state: #state_ident) -> Self {
                        Self {
                            state,
                            component: #component_ident::new(#input_ident::new(), #output_ident::new())
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
                            component: #component_ident::new(#input_ident::new(), #output_ident::new())
                        }
                    }
                }
            }
        }
    }

    pub(crate) fn quote_impl_atomic(&self) -> TokenStream2 {
        let atomic_ident = self.atomic_ident();
        let state_ident = self.state.path.get_ident().unwrap();
        let input_ident = self.component.input_ident();
        let output_ident = self.component.output_ident();

        quote! {
            unsafe impl xdevs::atomic::UnsafeAtomic for #atomic_ident {
                type State = #state_ident;
                type Input = #input_ident;
                type Output = #output_ident;

                fn divide(&self) -> (&Self::State, &xdevs::component::Component<Self::Input, Self::Output>) {
                    (&self.state, &self.component)
                }

                fn divide_mut(&mut self) -> (&mut Self::State, &mut xdevs::component::Component<Self::Input, Self::Output>) {
                    (&mut self.state, &mut self.component)
                }
            }
        }
    }

    pub(crate) fn quote(&self) -> TokenStream2 {
        let component = self.component.quote();
        let atomic_struct = self.quote_struct();
        let atomic_impl = self.quote_impl();
        let atomic_impl_atomic = self.quote_impl_atomic();

        quote! {
            #component
            #atomic_struct
            #atomic_impl
            #atomic_impl_atomic
        }
    }
}
