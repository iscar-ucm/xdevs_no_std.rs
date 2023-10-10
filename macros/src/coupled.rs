use crate::{component::ComponentMeta, coupling::Couplings};
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use std::collections::HashSet;
use syn::{
    parse::{Parse, ParseStream},
    Error, Ident, LitBool, Token, TypePath,
};

pub struct CoupledMeta {
    pub constant: bool,
    pub component: ComponentMeta,
    pub components: TypePath,
    pub couplings: Couplings,
}

impl Parse for CoupledMeta {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut constant = false;
        let mut component = None;
        let mut components = None;
        let mut couplings = None;

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
            } else if token == "couplings" {
                couplings = Some(input.parse::<Couplings>()?);
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
            couplings: couplings.unwrap_or_default(),
        })
    }
}

impl CoupledMeta {
    pub(crate) fn coupled_ident(&self) -> Ident {
        self.component.name.clone()
    }

    pub(crate) fn quote_struct(&self) -> TokenStream2 {
        let coupled_ident = self.coupled_ident();
        let component_ident = self.component.component_ident();
        let components_ident = self.components.path.get_ident().unwrap();

        quote! {
            pub struct #coupled_ident {
                pub component: #component_ident,
                pub components: #components_ident,
            }
        }
    }

    pub(crate) fn quote_impl(&self) -> TokenStream2 {
        let coupled_ident = self.coupled_ident();
        let component_ident = self.component.component_ident();
        let components_ident = self.components.path.get_ident().unwrap();
        let input_ident = self.component.input_ident();
        let output_ident = self.component.output_ident();

        if self.constant {
            quote! {
                impl #coupled_ident {
                    pub const fn new(components: #components_ident) -> Self {
                        Self {
                            components,
                            component: #component_ident::new(#input_ident::new(), #output_ident::new())
                        }
                    }
                }
            }
        } else {
            quote! {
                impl #coupled_ident {
                    pub fn new(components: #components_ident) -> Self {
                        Self {
                            components,
                            component: #component_ident::new(#input_ident::new(), #output_ident::new())
                        }
                    }
                }
            }
        }
    }

    pub fn quote_simulator(&self) -> TokenStream2 {
        let coupled_ident = self.coupled_ident();

        quote! {
            unsafe impl xdevs::simulator::AbstractSimulator for #coupled_ident {
                fn start(&mut self, t_start: f64) -> f64 {
                    let t_next = xdevs::simulator::AbstractSimulator::start(&mut self.components, t_start);
                    self.component.set_sim_t(t_start, t_next);
                    t_next
                }

                fn stop(&mut self, t_stop: f64) {
                    xdevs::simulator::AbstractSimulator::stop(&mut self.components, t_stop);
                    self.component.set_sim_t(t_stop, f64::INFINITY);
                }

                fn lambda(&mut self, t: f64) {
                    if t >= self.component.t_next {
                        xdevs::simulator::AbstractSimulator::lambda(&mut self.components, t);
                        self.propagate_eoc();
                    }
                }

                fn delta(&mut self, t: f64) -> f64 {
                    self.propagate_xic();
                    let t_next = xdevs::simulator::AbstractSimulator::delta(&mut self.components, t);
                    self.component.set_sim_t(t, t_next);
                    self.component.clear_ports();
                    t_next
                }
            }
        }
    }

    pub fn quote(&self) -> TokenStream2 {
        let coupled_ident = self.coupled_ident();

        let component = self.component.quote();
        let coupled_struct = self.quote_struct();
        let coupled_impl = self.quote_impl();
        let couplings = self.couplings.quote(&coupled_ident);
        let simulator = self.quote_simulator();

        quote! {
            #component
            #coupled_struct
            #coupled_impl
            #couplings
            #simulator
        }
    }
}
