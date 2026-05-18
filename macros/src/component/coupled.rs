mod components;

use super::filter_generics;
use super::impl_component;
use super::CommonComponent;
use super::ParsedComponentFields;
use components::Components;
use proc_macro2::TokenStream as TokenStream2;
use syn::{parse2, Error, GenericParam, Ident, ItemStruct, Result};

/// Parsed representation of a coupled2 component macro input.
pub struct Component {
    pub common: CommonComponent,
    pub components: Components,
}

impl Component {
    pub fn parse(args: TokenStream2, item: TokenStream2) -> Result<Self> {
        let component: ItemStruct = parse2(item)?;

        let ident = component.ident.clone();
        let ParsedComponentFields {
            input: inputs,
            output: outputs,
            state,
            components,
        } = ParsedComponentFields::parse(&component)?;

        // Coupled components must not declare state fields.
        if !state.is_empty() {
            return Err(Error::new_spanned(
                &state[0].ident,
                "Coupled components cannot define #[state] fields",
            ));
        }

        // Check that components is defined
        if components.is_empty() {
            return Err(Error::new_spanned(&component, "No components found"));
        }

        // Build variables for generation.
        let generics = component.generics.clone();
        let components_generics = filter_generics(&components, &generics);
        let components_ident = Ident::new(&format!("{}Components", &ident), ident.span());

        // Generate common component and components
        let common = CommonComponent::new(
            ident,
            generics,
            inputs,
            outputs,
            args,
            "unknown coupled component argument",
        )?;
        let components = Components::new(components, components_ident, components_generics);

        Ok(Component { common, components })
    }

    pub fn quote(&self) -> TokenStream2 {
        let ident = &self.common.ident;

        // Prepare identifiers for code generation
        let input_ident = Ident::new(
            &format!("{}Input", &self.common.ident),
            self.common.ident.span(),
        );
        let output_ident = Ident::new(
            &format!("{}Output", &self.common.ident),
            self.common.ident.span(),
        );
        let components_ident = Ident::new(
            &format!("{}Components", &self.common.ident),
            self.common.ident.span(),
        );
        let components_fields = self.components.field_idents();
        let components_tys = self.components.field_tys();

        // Extract generics for impl
        let (impl_generics, ty_generics, where_clause) = self.common.generics.split_for_impl();
        let (_, input_generics, _) = &self.common.input.generics.split_for_impl();
        let (_, output_generics, _) = &self.common.output.generics.split_for_impl();
        let (_, components_generics, _) = &self.components.generics.split_for_impl();

        // Generate input, output, and components structs
        let is_bagmux = self.common.rt_engine.is_some();
        let input_struct = self.common.input.quote(is_bagmux);
        let output_struct = self.common.output.quote(is_bagmux);
        let components_struct = self.components.quote();
        // Component trait implementation
        let component_impl = impl_component(
            ident,
            &input_ident,
            &output_ident,
            &self.common.generics,
            input_generics,
            output_generics,
        );

        // Generate rt_engine code if defined
        let rt_engine_impl = self.common.quote_rt_engine();

        // Generate wrapper structs for inner components' inputs and outputs
        // These structs hold references to all inner components' inputs/outputs,
        // allowing them to be passed as a single argument to trait methods without
        // exposing the component's state or internal structure.
        let component_inputs_ident = Ident::new(&format!("{}ComponentsInput", ident), ident.span());
        let component_outputs_ident =
            Ident::new(&format!("{}ComponentsOutput", ident), ident.span());

        let component_input_fields: Vec<TokenStream2> = self
            .components
            .components
            .iter()
            .map(|field| {
                let field_ident = &field.ident;
                let field_ty = &field.ty;
                quote::quote! {
                    pub #field_ident: <#field_ty as ::xdevs::traits::Component>::InputRef<'__xdevs_inner>
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
                    pub #field_ident: <#field_ty as ::xdevs::traits::Component>::OutputRef<'__xdevs_inner>
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
                    let (#input_var, #output_var) = ::xdevs::traits::Component::get_ports(&mut self.components.#field_ident);
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
                    let #output_var = ::xdevs::traits::Component::get_out_ports(&self.components.#field_ident);
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
        let components_ty_args: Vec<TokenStream2> = self
            .components
            .generics
            .params
            .iter()
            .map(|p| match p {
                GenericParam::Type(tp) => {
                    let ident = &tp.ident;
                    quote::quote! { #ident }
                }
                GenericParam::Lifetime(lp) => {
                    let lifetime = &lp.lifetime;
                    quote::quote! { #lifetime }
                }
                GenericParam::Const(cp) => {
                    let ident = &cp.ident;
                    quote::quote! { #ident }
                }
            })
            .collect();
        let has_components_params = !components_params.is_empty();

        // Extract lifetime parameters to generate bounds (lifetime: '__xdevs_inner)
        let lifetime_params: Vec<_> = self
            .components
            .generics
            .params
            .iter()
            .filter_map(|p| {
                if let GenericParam::Lifetime(lp) = p {
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
                    quote::quote! { <'_, #(#components_ty_args),*> },
                    quote::quote! { <'__xdevs_inner, #(#components_ty_args),*> },
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
            #rt_engine_impl

            /// Wrapper struct holding mutable references to all inner components' inputs.
            pub struct #component_inputs_ident #wrapper_def_generics #wrapper_where_clause {
                #(#component_input_fields),*
            }

            /// Wrapper struct holding references to all inner components' outputs.
            pub struct #component_outputs_ident #wrapper_def_generics #wrapper_where_clause {
                #(#component_output_fields),*
            }

            pub struct #ident #impl_generics #where_clause {
                pub input: #input_ident #input_generics,
                pub output: #output_ident #output_generics,
                pub t_last: f64,
                pub t_next: f64,
                pub components: #components_ident #components_generics,
            }
            impl #impl_generics #ident #ty_generics #where_clause {
                #[inline]
                pub const fn build(#(#components_fields: #components_tys),*) -> Self {
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
            unsafe impl #impl_generics ::xdevs::traits::PartialCoupled for #ident #ty_generics #where_clause {
                type ComponentsInput = #component_inputs_ident #wrapper_trait_generics;
                type ComponentsOutput = #component_outputs_ident #wrapper_trait_generics;
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
                fn lambda(&mut self, t: f64) {
                    if t >= ::xdevs::traits::Component::get_t_next(self) {
                        // propagate lambda to all components
                        #(::xdevs::traits::AbstractSimulator::lambda(&mut self.components.#components_fields, t);)*
                        // propagate EOCs via Coupled trait
                        #(#component_out_ports_inits)*
                        let component_outputs: #component_outputs_ident #wrapper_use_generics = #component_outputs_ident {
                            #(#component_output_inits),*
                        };
                        <Self as ::xdevs::Coupled>::eoc(&component_outputs, &mut self.output);
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
                        <Self as ::xdevs::Coupled>::eic(&self.input, &mut component_inputs);
                        <Self as ::xdevs::Coupled>::ic(&component_outputs, &mut component_inputs);
                    }
                    // get minimum t_next from all components after executing their delta
                    let mut t_next = f64::INFINITY;
                    #(t_next = f64::min(t_next, ::xdevs::traits::AbstractSimulator::delta(&mut self.components.#components_fields, t));)*
                    // clear input and output events
                    ::xdevs::traits::Component::clear_output(self);
                    ::xdevs::traits::Component::clear_input(self);
                    // set t_last to t and t_next to minimum t_next
                    ::xdevs::traits::Component::set_t_last(self, t);
                    ::xdevs::traits::Component::set_t_next(self, t_next);

                    t_next
                }
            }
        };
        expanded.into()
    }
}
