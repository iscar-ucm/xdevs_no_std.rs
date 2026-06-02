use crate::component::ComponentArgs;

use super::filter_generics;
use super::impl_component;
use super::state::State;
use super::Component;
use super::ParsedComponentFields;
use proc_macro2::TokenStream as TokenStream2;
use syn::{Error, Ident, ItemStruct, Result};

/// Parsed representation of an atomic component macro input.
pub struct Atomic {
    pub component: Component,
    pub state: State,
}

impl Atomic {
    pub fn parse(args: ComponentArgs, item: ItemStruct) -> Result<Self> {
        let ident = item.ident.clone();
        let ParsedComponentFields {
            input: inputs,
            output: outputs,
            state,
            components,
        } = ParsedComponentFields::parse(&item)?;

        // Atomic components must not declare inner components.
        if !components.is_empty() {
            return Err(Error::new_spanned(
                &components[0].ident,
                "Atomic components cannot define #[components] fields",
            ));
        }

        // Check that state is defined
        if state.is_empty() {
            return Err(Error::new_spanned(&item, "No state definition found"));
        }

        // Build shared component metadata (rt_engine + top-level ports).
        let generics = item.generics.clone();
        let state_generics = filter_generics(&state, &generics);
        let state_ident = Ident::new(&format!("{ident}State"), ident.span());
        let component = Component::new(ident, generics, inputs, outputs, args)?;
        let state = State::new(state, state_ident, state_generics);

        Ok(Atomic { component, state })
    }

    pub fn quote(&self) -> TokenStream2 {
        let ident = &self.component.ident;

        // Prepare identifiers for code generation
        let input_ident = &self.component.input.ident;
        let output_ident = &self.component.output.ident;
        let state_ident = &self.state.ident;
        let state_fields = self.state.field_idents();
        let state_tys = self.state.field_tys();

        // Extract generics for impl
        let (impl_generics, ty_generics, where_clause) = self.component.generics.split_for_impl();
        let (_, input_generics, _) = &self.component.input.generics.split_for_impl();
        let (_, output_generics, _) = &self.component.output.generics.split_for_impl();
        let (_, state_generics, _) = &self.state.generics.split_for_impl();

        // Generate input, output, and state structs
        let is_bagmux = self.component.rt_engine.is_some();
        let input_struct = self.component.input.quote(is_bagmux);
        let output_struct = self.component.output.quote(is_bagmux);
        let state_struct = self.state.quote();

        // Generate rt_engine code if defined
        let rt_engine_impl = self.component.quote_rt_engine();

        // Component trait implementation
        let component_impl = impl_component(
            ident,
            &input_ident,
            &output_ident,
            &self.component.generics,
            input_generics,
            output_generics,
        );

        // Generate the expanded code
        let expanded = quote::quote! {
            #input_struct
            #output_struct
            #state_struct
            #rt_engine_impl
            pub struct #ident #impl_generics #where_clause {
                pub t_last: f64,
                pub t_next: f64,
                pub state: #state_ident #state_generics,
            }
            impl #impl_generics #ident #ty_generics #where_clause {
                #[inline]
                pub const fn build(#(#state_fields: #state_tys),*) -> Self {
                    Self {
                        t_last: 0.0,
                        t_next: f64::INFINITY,
                        state: #state_ident::new(#(#state_fields),*),
                    }
                }
            }
            #component_impl
            unsafe impl #impl_generics ::xdevs::traits::PartialAtomic for #ident #ty_generics #where_clause {
                type State = #state_ident #state_generics;
            }
            unsafe impl #impl_generics ::xdevs::traits::AbstractSimulator for #ident #ty_generics #where_clause {
                #[inline]
                fn start(&mut self, t_start: f64) -> f64 {
                    // set t_last to t_start
                    ::xdevs::traits::Component::set_t_last(self, t_start);
                    // start state and get t_next from ta
                    <Self as ::xdevs::Atomic>::start(&mut self.state);
                    let t_next = t_start + <Self as ::xdevs::Atomic>::ta(&self.state);
                    ::xdevs::traits::Component::set_t_next(self, t_next);

                    t_next
                }
                #[inline]
                fn stop(&mut self, t_stop: f64) {
                    // stop state
                    <Self as ::xdevs::Atomic>::stop(&mut self.state);
                    // set t_last to t_stop and t_next to infinity
                    ::xdevs::traits::Component::set_t_last(self, t_stop);
                    ::xdevs::traits::Component::set_t_next(self, f64::INFINITY);
                }
                #[inline]
                fn lambda(&mut self, output: &mut Self::Output, t: f64) {
                    if t >= ::xdevs::traits::Component::get_t_next(self) {
                        // execute atomic model's lambda if applies
                        <Self as ::xdevs::Atomic>::lambda(&self.state, output);
                    }
                }
                #[inline]
                fn delta(&mut self, input: &mut Self::Input, output: &mut Self::Output, t: f64) -> f64 {
                    let mut t_next = ::xdevs::traits::Component::get_t_next(self);
                    if !::xdevs::traits::Bag::is_empty(input) {
                        if t >= t_next {
                            // confluent transition
                            <Self as ::xdevs::Atomic>::delta_conf(&mut self.state, input);
                            // clear output events
                            <Self::Output as ::xdevs::traits::Bag>::clear(output);
                        } else {
                            // external transition
                            let e = t - ::xdevs::traits::Component::get_t_last(self);
                            <Self as ::xdevs::Atomic>::delta_ext(&mut self.state, e, input);
                        }
                        // clear input events
                        <Self::Input as ::xdevs::traits::Bag>::clear(input);
                    } else if t >= t_next {
                        // internal transition
                        <Self as ::xdevs::Atomic>::delta_int(&mut self.state);
                        // clear output events
                        <Self::Output as ::xdevs::traits::Bag>::clear(output);
                    } else {
                        return t_next; // nothing to do
                    }
                    // get t_next from ta and set new t_last and t_next
                    t_next = t + <Self as ::xdevs::Atomic>::ta(&self.state);
                    ::xdevs::traits::Component::set_t_last(self, t);
                    ::xdevs::traits::Component::set_t_next(self, t_next);

                    t_next
                }
            }
        };

        expanded.into()
    }
}
