mod components;

use super::check_duplicate_fields;
use super::filter_generics;
use super::impl_component;
use super::port::Ports;
use super::Field;
use components::Components;
use proc_macro2::TokenStream as TokenStream2;
use syn::{Error, Generics, Ident, ItemStruct};

pub struct Component {
    pub ident: Ident,
    pub generics: Generics,
    pub components: Components,
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

    pub fn parse(_args: TokenStream2, item: TokenStream2) -> syn::Result<Self> {
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

        // Extract generics for impl
        let (impl_generics, ty_generics, _) = self.generics.split_for_impl();
        let input_generics = &self.inputs.get_generics();
        let output_generics = &self.outputs.get_generics();
        let components_generics = &self.components.get_generics();

        // Component trait implementation
        let component_impl = impl_component(
            ident,
            input_ident,
            output_ident,
            &self.generics,
            input_generics,
            output_generics,
        );

        // Generate wrapper structs for inner components' inputs and outputs
        // These structs hold references to all inner components' inputs/outputs,
        // allowing them to be passed as a single argument to trait methods without
        // exposing the component's state or internal structure.
        let component_inputs_ident =
            syn::Ident::new(&format!("{}ComponentsInput", ident), ident.span());
        let component_outputs_ident =
            syn::Ident::new(&format!("{}ComponentsOutput", ident), ident.span());

        let component_input_fields: Vec<TokenStream2> = self
            .components
            .components
            .iter()
            .map(|field| {
                let field_ident = &field.ident;
                let field_ty = &field.ty;
                quote::quote! {
                    pub #field_ident: <#field_ty as xdevs::traits::Component>::InputRef<'__xdevs_inner>
                }
            })
            .collect();

        let component_output_fields: Vec<TokenStream2> = self
            .components
            .components
            .iter()
            .map(|field| {
                let field_ident = &field.ident;
                let field_ty = &field.ty;
                quote::quote! {
                    pub #field_ident: <#field_ty as xdevs::traits::Component>::OutputRef<'__xdevs_inner>
                }
            })
            .collect();

        let component_ports_inits: Vec<TokenStream2> = self
            .components
            .components
            .iter()
            .map(|field| {
                let field_ident = &field.ident;
                let input_var = quote::format_ident!("{}_input", field_ident);
                let output_var = quote::format_ident!("{}_output", field_ident);
                quote::quote! {
                    let (#input_var, #output_var) = xdevs::traits::Component::get_ports(&mut self.components.#field_ident);
                }
            })
            .collect();

        // For lambda, we only use output refs via get_out_ports
        let component_out_ports_inits: Vec<TokenStream2> = self
            .components
            .components
            .iter()
            .map(|field| {
                let field_ident = &field.ident;
                let output_var = quote::format_ident!("{}_output", field_ident);
                quote::quote! {
                    let #output_var = xdevs::traits::Component::get_out_ports(&self.components.#field_ident);
                }
            })
            .collect();

        let component_input_inits: Vec<TokenStream2> = self
            .components
            .components
            .iter()
            .map(|field| {
                let field_ident = &field.ident;
                let input_var = quote::format_ident!("{}_input", field_ident);
                quote::quote! {
                    #field_ident: #input_var
                }
            })
            .collect();

        let component_output_inits: Vec<TokenStream2> = self
            .components
            .components
            .iter()
            .map(|field| {
                let field_ident = &field.ident;
                let output_var = quote::format_ident!("{}_output", field_ident);
                quote::quote! {
                    #field_ident: #output_var
                }
            })
            .collect();

        // Generate struct definition generics and usage generics for ComponentsInput/ComponentsOutput
        // We need to include ALL generic parameters from components (lifetimes, types, consts)
        // so that the field types can reference them.
        let components_params: Vec<_> = self.components.generics.params.iter().collect();
        let has_components_params = !components_params.is_empty();

        // Extract lifetime parameters to generate bounds (lifetime: '__xdevs_inner)
        let lifetime_params: Vec<_> = self
            .components
            .generics
            .params
            .iter()
            .filter_map(|p| {
                if let syn::GenericParam::Lifetime(lp) = p {
                    Some(&lp.lifetime)
                } else {
                    None
                }
            })
            .collect();
        let has_lifetime_params = !lifetime_params.is_empty();

        // Generate where clause for wrapper structs to bound component lifetimes
        let wrapper_where_clause = if has_lifetime_params {
            quote::quote! { where #(#lifetime_params: '__xdevs_inner),* }
        } else {
            quote::quote! {}
        };

        let (wrapper_def_generics, wrapper_use_generics, wrapper_trait_generics) =
            if has_components_params {
                (
                    quote::quote! { <'__xdevs_inner, #(#components_params),*> },
                    quote::quote! { <'_, #(#components_params),*> },
                    quote::quote! { <'__xdevs_inner, #(#components_params),*> },
                )
            } else {
                (
                    quote::quote! { <'__xdevs_inner> },
                    quote::quote! { <'_> },
                    quote::quote! { <'__xdevs_inner> },
                )
            };

        // Generate the expanded code
        let expanded = quote::quote! {
            #input_struct
            #output_struct
            #components_struct

            /// Wrapper struct holding mutable references to all inner components' inputs.
            pub struct #component_inputs_ident #wrapper_def_generics #wrapper_where_clause {
                #(#component_input_fields),*
            }

            /// Wrapper struct holding references to all inner components' outputs.
            pub struct #component_outputs_ident #wrapper_def_generics #wrapper_where_clause {
                #(#component_output_fields),*
            }

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
            unsafe impl #impl_generics xdevs::traits::PartialCoupled for #ident #ty_generics{
                type ComponentsInput<'__xdevs_inner> = #component_inputs_ident #wrapper_trait_generics where Self: '__xdevs_inner;
                type ComponentsOutput<'__xdevs_inner> = #component_outputs_ident #wrapper_trait_generics where Self: '__xdevs_inner;
            }
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
                        // propagate EOCs via Coupled trait
                        #(#component_out_ports_inits)*
                        let component_outputs: #component_outputs_ident #wrapper_use_generics = #component_outputs_ident {
                            #(#component_output_inits),*
                        };
                        <Self as xdevs::Coupled>::eoc(&component_outputs, &mut self.output);
                    }
                }

                #[inline]
                fn delta(&mut self, t: f64) -> f64 {
                    // propagate EICs and ICs via Coupled trait
                    {
                        #(#component_ports_inits)*
                        let component_outputs: #component_outputs_ident #wrapper_use_generics = #component_outputs_ident {
                            #(#component_output_inits),*
                        };
                        let mut component_inputs: #component_inputs_ident #wrapper_use_generics = #component_inputs_ident {
                            #(#component_input_inits),*
                        };
                        <Self as xdevs::Coupled>::eic(&self.input, &mut component_inputs);
                        <Self as xdevs::Coupled>::ic(&component_outputs, &mut component_inputs);
                    }
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
