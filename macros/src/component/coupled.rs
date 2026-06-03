pub mod components;

use crate::combine_err;
use crate::component::ComponentArgs;

use super::impl_component;
use super::Component;
use proc_macro2::TokenStream as TokenStream2;
use syn::{Error, Ident, ItemStruct, Result};

/// Parsed representation of a coupled component macro input.
pub struct Coupled {
    pub component: Component,
}

impl Coupled {
    pub fn parse(args: ComponentArgs, item: ItemStruct) -> Result<Self> {
        let component = Component::new(&args, &item)?;

        let mut acc: Option<Error> = None;
        // Coupled components must not declare state fields.
        if !component.state.fields.is_empty() {
            for field in &component.state.fields {
                combine_err(
                    &mut acc,
                    Error::new_spanned(
                        &field.ident,
                        "Coupled components cannot define #[state] fields",
                    ),
                );
            }
        }

        // Check that components is defined
        if component.components.fields.is_empty() {
            combine_err(&mut acc, Error::new_spanned(&item, "No components found"));
        }

        if let Some(err) = acc {
            return Err(err);
        }

        Ok(Coupled { component })
    }

    pub fn quote(&self) -> TokenStream2 {
        let component = &self.component;

        let vis = &component.vis;
        let ident = &component.ident;
        let span = ident.span();

        // Prepare identifiers for code generation
        let input = &component.input;
        let output = &component.output;
        let components = &component.components;

        let input_ident = &input.ident;
        let output_ident = &output.ident;
        let components_ident = &components.ident;

        let components_fields = components.field_idents();
        let components_tys = components.field_tys();

        // Extract generics for impl
        let (impl_generics, ty_generics, where_clause) = component.generics.split_for_impl();
        let (_, input_ty_generics, _) = input.generics.split_for_impl();
        let (_, output_ty_generics, _) = output.generics.split_for_impl();

        let (components_impl_generics, components_ty_generics, components_where_clause) =
            components.generics.split_for_impl();

        // Generate input, output, and components structs
        let is_bagmux = component.rt_engine.is_some();

        let input_struct = input.quote(is_bagmux);
        let output_struct = output.quote(is_bagmux);
        let components_struct = components.quote();

        // Component trait implementation
        let component_impl = impl_component(
            ident,
            &input_ident,
            &output_ident,
            &component.generics,
            &input_ty_generics,
            &output_ty_generics,
        );

        // Generate rt_engine code if defined
        let rt_engine_impl = component.quote_rt_engine();

        // Generate wrapper structs for inner components' inputs and outputs.
        // These structs hold all inner components' inputs/outputs,
        // allowing them to be passed as a single argument to trait methods.
        let components_input_ident = Ident::new(&format!("{ident}ComponentsInput"), span);
        let components_output_ident = Ident::new(&format!("{ident}ComponentsOutput"), span);
        let components_input_fields: Vec<TokenStream2> = components
            .fields
            .iter()
            .map(|field| {
                let field_ident = &field.ident;
                let field_ty = &field.ty;

                quote::quote! {
                    pub #field_ident: <#field_ty as ::xdevs::traits::Component>::Input
                }
            })
            .collect();

        let components_output_fields: Vec<TokenStream2> = components
            .fields
            .iter()
            .map(|field| {
                let field_ident = &field.ident;
                let field_ty = &field.ty;

                quote::quote! {
                    pub #field_ident: <#field_ty as ::xdevs::traits::Component>::Output
                }
            })
            .collect();

        // Generate the expanded code
        let expanded = quote::quote! {
            #input_struct
            #output_struct
            #components_struct
            #rt_engine_impl

            /// Wrapper struct holding mutable references to all inner components' inputs.
            #[derive(::xdevs::Bag)]
            pub struct #components_input_ident #components_impl_generics #components_where_clause {
                #(#components_input_fields),*
            }

            /// Wrapper struct holding references to all inner components' outputs.
            #[derive(::xdevs::Bag)]
            pub struct #components_output_ident #components_impl_generics #components_where_clause {
                #(#components_output_fields),*
            }

            #vis struct #ident #impl_generics #where_clause {
                components_input: #components_input_ident #components_ty_generics,
                components_output: #components_output_ident #components_ty_generics,
                t_last: f64,
                t_next: f64,
                components: #components_ident #components_ty_generics,
            }
            impl #impl_generics #ident #ty_generics #where_clause {
                #[inline]
                pub fn build(#(#components_fields: #components_tys),*) -> Self {
                    Self {
                        components_input: <#components_input_ident #components_ty_generics as ::xdevs::traits::Bag>::build(),
                        components_output: <#components_output_ident #components_ty_generics as ::xdevs::traits::Bag>::build(),
                        t_last: 0.0,
                        t_next: f64::INFINITY,
                        components: #components_ident::new(#(#components_fields),*),
                    }
                }
            }
            #component_impl
            unsafe impl #impl_generics ::xdevs::traits::PartialCoupled for #ident #ty_generics #where_clause {
                type ComponentsInput = #components_input_ident #components_ty_generics;
                type ComponentsOutput = #components_output_ident #components_ty_generics;
            }
            unsafe impl #impl_generics ::xdevs::traits::AbstractSimulator for #ident #ty_generics #where_clause {
                #[inline]
                fn start(&mut self, t_start: f64) -> f64 {
                    // set t_last to t_start
                    ::xdevs::traits::Component::set_t_last(self, t_start);
                    // get minimum t_next from all components
                    let mut t_next = f64::INFINITY;
                    #(t_next = f64::min(t_next, ::xdevs::traits::AbstractSimulator::start(&mut self.components.#components_fields, t_start));)*
                    // set t_next to minimum t_next
                    ::xdevs::traits::Component::set_t_next(self, t_next);

                    t_next
                }

                #[inline]
                fn stop(&mut self, t_stop: f64) {
                    // stop all components
                    #(::xdevs::traits::AbstractSimulator::stop(&mut self.components.#components_fields, t_stop);)*
                    // set t_last to t_stop and t_next to infinity
                    ::xdevs::traits::Component::set_t_last(self, t_stop);
                    ::xdevs::traits::Component::set_t_next(self, f64::INFINITY);
                }

                #[inline]
                fn lambda(&mut self, output: &mut Self::Output, t: f64) {
                    if t >= ::xdevs::traits::Component::get_t_next(self) {
                        // propagate lambda to all components
                        #(::xdevs::traits::AbstractSimulator::lambda(&mut self.components.#components_fields, &mut self.components_output.#components_fields, t);)*
                        // propagate EOCs via Coupled trait
                        <Self as ::xdevs::Coupled>::eoc(&self.components_output, output);
                    }
                }

                #[inline]
                fn delta(&mut self, input: &mut Self::Input, output: &mut Self::Output, t: f64) -> f64 {
                    // propagate EICs and ICs via Coupled trait
                    <Self as ::xdevs::Coupled>::eic(input, &mut self.components_input);
                    <Self as ::xdevs::Coupled>::ic(&self.components_output, &mut self.components_input);

                    // get minimum t_next from all components after executing their delta
                    let mut t_next = f64::INFINITY;
                    #(t_next = f64::min(t_next, ::xdevs::traits::AbstractSimulator::delta(
                        &mut self.components.#components_fields,
                        &mut self.components_input.#components_fields,
                        &mut self.components_output.#components_fields,
                         t));)*

                    // set t_last to t and t_next to minimum t_next
                    ::xdevs::traits::Component::set_t_last(self, t);
                    ::xdevs::traits::Component::set_t_next(self, t_next);

                    // clear input and output events
                    <Self::Input as ::xdevs::traits::Bag>::clear(input);
                    <Self::Output as ::xdevs::traits::Bag>::clear(output);

                    t_next
                }
            }
        };
        expanded.into()
    }
}
