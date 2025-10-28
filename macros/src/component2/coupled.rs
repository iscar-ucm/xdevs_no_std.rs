mod components;
mod coupling;

use super::port::Ports;
use super::Field;
use components::Components;
use coupling::Couplings;
use proc_macro2::TokenStream as TokenStream2;
use syn::{parse::{ParseStream, Parse},Error, Ident, ItemStruct, Token};

struct CoupledArgs{
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
        let mut components = Components::new(Vec::new());
        let mut inputs = Ports::new(Vec::new());
        let mut outputs = Ports::new(Vec::new());

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
                    components.add_component(Field {
                        ident: field_ident,
                        ty: field_ty,
                    });
                } else if attr.path().is_ident("input") {
                    let field_ident = field.ident.clone().unwrap();
                    let field_ty = field.ty.clone();
                    inputs.add_port(Field {
                        ident: field_ident,
                        ty: field_ty,
                    });
                } else if attr.path().is_ident("output") {
                    let field_ident = field.ident.clone().unwrap();
                    let field_ty = field.ty.clone();
                    outputs.add_port(Field {
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

        // Parse arguments
        let args = syn::parse2::<CoupledArgs>(args)?;
        let couplings = args.couplings;

        Ok(Component {
            ident,
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
            couplings.quote()
        } else {
            (vec![], vec![])
        };

        // Generate the expanded code
        let expanded = quote::quote! {
            #input_struct
            #output_struct
            #components_struct
            pub struct #ident {
                pub input: #input_ident,
                pub output: #output_ident,
                pub t_last: f64,
                pub t_next: f64,
                pub components: #components_ident,
            }
            impl #ident {
                #[inline]
                pub fn new(#(#components_fields: #components_tys),*) -> Self {
                    Self {
                        input: #input_ident::new(),
                        output: #output_ident::new(),
                        t_last: 0.0,
                        t_next: f64::INFINITY,
                        components: #components_ident::new(#(#components_fields),*),
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
            unsafe impl xdevs::traits::AbstractSimulator for #ident {
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
