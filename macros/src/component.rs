use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use std::collections::HashSet;
use syn::{
    parse::{Parse, ParseStream},
    Error, Ident, Token,
};

mod atomic;
mod coupled;
mod port;
use atomic::State;
use coupled::Coupled;
use port::Ports;

pub enum ComponentType {
    Atomic(State),
    Coupled(Coupled),
}

impl ComponentType {
    pub fn quote(&self, component: &Component) -> TokenStream2 {
        match self {
            Self::Atomic(meta) => meta.quote(component),
            Self::Coupled(meta) => meta.quote(component),
        }
    }

    pub fn component_ident(&self) -> Vec<TokenStream2> {
        match self {
            Self::Atomic(state) => state.ident(),
            Self::Coupled(coupled) => coupled.components.ident(),
        }
    }
    pub fn component_ty(&self) -> Vec<TokenStream2> {
        match self {
            Self::Atomic(state) => state.ty(),
            Self::Coupled(coupled) => coupled.components.ty(),
        }
    }
}

pub struct Component {
    pub ident: Ident,
    pub ty: ComponentType,
    pub input: Ports,
    pub output: Ports,
}

impl Parse for Component {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut ident = None;
        let mut state = None;
        let mut components = None;
        let mut couplings = None;
        let mut input_ports = None;
        let mut output_ports = None;

        let mut cache = HashSet::new();

        while !input.is_empty() {
            let token: Ident = input.parse()?;
            // assert that the token has not been parsed before
            if cache.contains(&token) {
                return Err(Error::new(
                    token.span(),
                    "duplicate component meta argument",
                ));
            } else {
                cache.insert(token.clone());
            }
            input.parse::<Token![=]>()?; // consume the '='

            if token == "ident" {
                ident = Some(input.parse()?);
            } else if token == "input" {
                input_ports = Some(input.parse()?);
            } else if token == "output" {
                output_ports = Some(input.parse()?);
            } else if token == "state" {
                if components.is_some() {
                    return Err(Error::new(
                        token.span(),
                        "state and components cannot be specified together",
                    ));
                }
                if couplings.is_some() {
                    return Err(Error::new(
                        token.span(),
                        "state and couplings cannot be specified together",
                    ));
                }
                state = Some(input.parse()?);
            } else if token == "components" {
                if state.is_some() {
                    return Err(Error::new(
                        token.span(),
                        "components and state cannot be specified together",
                    ));
                }
                components = Some(input.parse()?);
            } else if token == "couplings" {
                if state.is_some() {
                    return Err(Error::new(
                        token.span(),
                        "components and state cannot be specified together",
                    ));
                }
                couplings = Some(input.parse()?);
            } else {
                return Err(Error::new(token.span(), "unknown component meta argument"));
            }

            if !input.is_empty() {
                input.parse::<Token![,]>()?; // comma between meta arguments
            }
        }

        let ty = if let Some(state) = state {
            ComponentType::Atomic(state)
        } else if let Some(components) = components {
            ComponentType::Coupled(coupled::Coupled {
                components: components,
                couplings: couplings,
            })
        } else {
            return Err(Error::new(input.span(), "component type not specified"));
        };

        Ok(Self {
            ident: ident.unwrap(),
            ty,
            input: input_ports.unwrap_or_default(),
            output: output_ports.unwrap_or_default(),
        })
    }
}

impl Component {
    pub fn input_ident(&self) -> Ident {
        let ident = format!("{ident}Input", ident = self.ident);
        Ident::new(&ident, self.input.span())
    }

    pub fn output_ident(&self) -> Ident {
        let ident = format!("{ident}Output", ident = self.ident);
        syn::Ident::new(&ident, self.output.span())
    }

    pub fn quote(&self) -> TokenStream2 {
        let ident = &self.ident;
        let input_ident = self.input_ident();
        let output_ident = self.output_ident();

        let input_struct = self.input.quote(&input_ident);
        let output_struct = self.output.quote(&output_ident);

        let other_ident = self.ty.component_ident();
        let other_ty = self.ty.component_ty();
        let other_quote = self.ty.quote(self);

        quote! {
            #input_struct
            #output_struct
            pub struct #ident {
                pub input: #input_ident,
                pub output: #output_ident,
                pub t_last: f64,
                pub t_next: f64,
                #(#other_ty),*
            }
            impl #ident {
                #[inline]
                pub const fn new(#(#other_ty),*) -> Self {
                    Self {
                        input: #input_ident::new(),
                        output: #output_ident::new(),
                        t_last: 0.0,
                        t_next: f64::INFINITY,
                        #(#other_ident),*
                    }
                }
            }
            unsafe impl xdevs::traits::Component for #ident {
                type Input = #input_ident;
                type Output = #output_ident;
                #[inline]
                fn get_t_last(&self) -> f64 {
                    self.t_last
                }
                #[inline]
                fn set_t_last(&mut self, t_last: f64) {
                    self.t_last = t_last;
                }
                #[inline]
                fn get_t_next(&self) -> f64 {
                    self.t_next
                }
                #[inline]
                fn set_t_next(&mut self, t_next: f64) {
                    self.t_next = t_next;
                }
                #[inline]
                fn get_input(&self) -> &Self::Input {
                    &self.input
                }
                #[inline]
                fn get_input_mut(&mut self) -> &mut Self::Input {
                    &mut self.input
                }
                #[inline]
                fn get_output(&self) -> &Self::Output {
                    &self.output
                }
                #[inline]
                fn get_output_mut(&mut self) -> &mut Self::Output {
                    &mut self.output
                }
            }
            #other_quote
        }
    }
}
