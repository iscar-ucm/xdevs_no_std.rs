use crate::combine_err;
use crate::component::ComponentArgs;

use super::impl_component;
use super::Component;
use proc_macro2::TokenStream as TokenStream2;
use syn::{Error, ItemStruct, Result};

/// Parsed representation of an atomic component macro input.
pub struct Atomic {
    pub component: Component,
}

impl Atomic {
    pub fn parse(args: ComponentArgs, item: ItemStruct) -> Result<Self> {
        let component = Component::new(&args, &item)?;

        let mut acc: Option<Error> = None;

        // Atomic components must not declare inner components.
        if !component.components.fields.is_empty() {
            for field in &component.components.fields {
                combine_err(
                    &mut acc,
                    Error::new_spanned(
                        &field.ident,
                        "Atomic components cannot define #[components] fields",
                    ),
                );
            }
        }

        // Check that state is defined
        if component.state.fields.is_empty() {
            combine_err(
                &mut acc,
                Error::new_spanned(&item, "No state definition found"),
            );
        }

        Ok(Atomic { component })
    }

    pub fn quote(&self) -> TokenStream2 {
        let ident = &self.component.ident;

        // Prepare identifiers for code generation
        let input_ident = &self.component.input.ident;
        let output_ident = &self.component.output.ident;
        let state_ident = &self.component.state.ident;
        let state_fields = self.component.state.field_idents();
        let state_tys = self.component.state.field_tys();

        // Extract generics for impl
        let (impl_generics, ty_generics, where_clause) = self.component.generics.split_for_impl();
        let (_, input_generics, _) = &self.component.input.generics.split_for_impl();
        let (_, output_generics, _) = &self.component.output.generics.split_for_impl();
        let (_, state_generics, _) = &self.component.state.generics.split_for_impl();

        // Generate input, output, and state structs
        let is_bagmux = self.component.rt_engine.is_some();
        let input_struct = self.component.input.quote(is_bagmux);
        let output_struct = self.component.output.quote(is_bagmux);
        let state_struct = self.component.state.quote();

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
