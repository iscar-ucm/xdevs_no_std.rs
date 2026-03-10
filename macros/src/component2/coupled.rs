mod components;
mod coupling;

use super::check_duplicate_fields;
use super::filter_generics;
use super::impl_component;
use super::port::Ports;
use super::Field;
use components::Components;
use coupling::Couplings;
use proc_macro2::TokenStream as TokenStream2;
use syn::{
    parse::{Parse, ParseStream},
    Error, Generics, Ident, ItemStruct, Token,
};

struct CoupledArgs {
    couplings: Option<Couplings>,
}

impl Parse for CoupledArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut couplings = None;

        while !input.is_empty() {
            let token: Ident = input.parse()?;
            input.parse::<Token![=]>()?; // consume the '='
            if token == "couplings" {
                couplings = Some(syn::parse2(input.parse()?)?);
            } else {
                return Err(Error::new(
                    token.span(),
                    "unknown coupled component argument",
                ));
            }
        }
        Ok(CoupledArgs { couplings })
    }
}

pub struct Component {
    pub ident: Ident,
    pub generics: Generics,
    pub components: Components,
    pub couplings: Option<Couplings>,
    pub inputs: Ports,
    pub outputs: Ports,
}

impl Component {
    fn input_ident(&self) -> syn::Ident {
        syn::Ident::new(&format!("{}Input", self.ident), self.ident.span())
    }
    fn output_ident(&self) -> syn::Ident {
        syn::Ident::new(&format!("{}Output", self.ident), self.ident.span())
    }
    fn components_ident(&self) -> syn::Ident {
        syn::Ident::new(&format!("{}Components", self.ident), self.ident.span())
    }

    pub fn parse(args: TokenStream2, item: TokenStream2) -> syn::Result<Self> {
        let component: ItemStruct = syn::parse2(item).unwrap();

        let ident = component.ident.clone();
        let mut last_attr = None;
        let mut components = Vec::new();
        let mut inputs = Vec::new();
        let mut outputs = Vec::new();

        // Parse struct fields
        for field in &component.fields {
            let field_attrs = &field.attrs;

            if field_attrs.len() > 1 {
                return Err(Error::new_spanned(
                    &field,
                    "Each field may have at most one attribute",
                ));
            }

            if let Some(attr) = field_attrs.first() {
                last_attr = Some(attr);
            }
            if let Some(attr) = last_attr {
                if attr.path().is_ident("components") {
                    let field_ident = field.ident.clone().unwrap();
                    let field_ty = field.ty.clone();
                    components.push(Field {
                        ident: field_ident,
                        ty: field_ty,
                    });
                } else if attr.path().is_ident("input") {
                    let field_ident = field.ident.clone().unwrap();
                    let field_ty = field.ty.clone();
                    inputs.push(Field {
                        ident: field_ident,
                        ty: field_ty,
                    });
                } else if attr.path().is_ident("output") {
                    let field_ident = field.ident.clone().unwrap();
                    let field_ty = field.ty.clone();
                    outputs.push(Field {
                        ident: field_ident,
                        ty: field_ty,
                    });
                } else {
                    return Err(Error::new_spanned(attr, "Unknown attribute"));
                }
            }
        }

        // Check that components is defined
        if components.is_empty() {
            return Err(Error::new_spanned(&component, "No components found"));
        }

        // Check for duplicate field names across input, output, and components
        check_duplicate_fields(&inputs, &outputs, &components)?;

        // Parse arguments
        let args = syn::parse2::<CoupledArgs>(args)?;
        let couplings = args.couplings;

        // Get generics and assign them to each struct accordingly
        let generics = component.generics.clone();
        let input_generics = filter_generics(&inputs, &generics);
        let output_generics = filter_generics(&outputs, &generics);
        let components_generics = filter_generics(&components, &generics);

        let inputs = Ports::new(inputs, input_generics);
        let outputs = Ports::new(outputs, output_generics);
        let components = Components::new(components, components_generics);

        Ok(Component {
            ident,
            generics,
            components,
            couplings,
            inputs,
            outputs,
        })
    }

    pub fn quote(&self) -> TokenStream2 {
        let ident = &self.ident;

        // Prepare identifiers for code generation
        let input_ident = &self.input_ident();
        let output_ident = &self.output_ident();
        let components_ident = &self.components_ident();
        let components_fields = self.components.field_idents();
        let components_tys = self.components.field_tys();
        let input_struct = self.inputs.quote(input_ident);
        let output_struct = self.outputs.quote(output_ident);
        let components_struct = self.components.quote(components_ident);

        let (eoc, xic) = if let Some(couplings) = &self.couplings {
            couplings.quote(
                &self.inputs.ports,
                &self.outputs.ports,
                &self.components.components,
            )
        } else {
            (vec![], vec![])
        };

        // Extract generics for impl
        let (impl_generics, ty_generics, _) = self.generics.split_for_impl();
        let (_, input_generics, _) = &self.inputs.generics.split_for_impl();
        let (_, output_generics, _) = &self.outputs.generics.split_for_impl();
        let (_, components_generics, _) = &self.components.generics.split_for_impl();

        // Component trait implementation
        let component_impl = impl_component(
            ident,
            input_ident,
            output_ident,
            &self.generics,
            input_generics,
            output_generics,
        );

        // Generate the expanded code
        let expanded = quote::quote! {
            #input_struct
            #output_struct
            #components_struct
            pub struct #ident #impl_generics {
                pub input: #input_ident #input_generics,
                pub output: #output_ident #output_generics,
                pub t_last: f64,
                pub t_next: f64,
                pub components: #components_ident #components_generics,
            }
            impl #impl_generics #ident #ty_generics {
                #[inline]
                pub fn build(#(#components_fields: #components_tys),*) -> Self {
                    Self {
                        input: #input_ident::new(),
                        output: #output_ident::new(),
                        t_last: 0.0,
                        t_next: f64::INFINITY,
                        components: #components_ident::new(#(#components_fields),*),
                    }
                }
            }
            #component_impl
            unsafe impl #impl_generics xdevs::traits::AbstractSimulator for #ident #ty_generics{
                #[inline]
                fn start(&mut self, t_start: f64) -> f64 {
                    // set t_last to t_start
                    xdevs::traits::Component::set_t_last(self, t_start);
                    // get minimum t_next from all components
                    let mut t_next = f64::INFINITY;
                    #(t_next = f64::min(t_next, xdevs::traits::AbstractSimulator::start(&mut self.components.#components_fields, t_start));)*
                    // set t_next to minimum t_next
                    xdevs::traits::Component::set_t_next(self, t_next);

                    t_next
                }

                #[inline]
                fn stop(&mut self, t_stop: f64) {
                    // stop all components
                    #(xdevs::traits::AbstractSimulator::stop(&mut self.components.#components_fields, t_stop);)*
                    // set t_last to t_stop and t_next to infinity
                    xdevs::traits::Component::set_t_last(self, t_stop);
                    xdevs::traits::Component::set_t_next(self, f64::INFINITY);
                }

                #[inline]
                fn lambda(&mut self, t: f64) {
                    if t >= xdevs::traits::Component::get_t_next(self) {
                        // propagate lambda to all components
                        #(xdevs::traits::AbstractSimulator::lambda(&mut self.components.#components_fields, t);)*
                        // propagate EOCs
                        #(#eoc);*
                    }
                }

                #[inline]
                fn delta(&mut self, t: f64) -> f64 {
                    // propagate EICs and ICs
                     #(#xic);*
                    // get minimum t_next from all components after executing their delta
                    let mut t_next = f64::INFINITY;
                    #(t_next = f64::min(t_next, xdevs::traits::AbstractSimulator::delta(&mut self.components.#components_fields, t));)*
                    // clear input and output events
                    xdevs::traits::Component::clear_output(self);
                    xdevs::traits::Component::clear_input(self);
                    // set t_last to t and t_next to minimum t_next
                    xdevs::traits::Component::set_t_last(self, t);
                    xdevs::traits::Component::set_t_next(self, t_next);

                    t_next
                }
            }
        };
        expanded.into()
    }
}
