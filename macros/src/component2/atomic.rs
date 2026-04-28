use super::filter_generics;
use super::impl_component;
use super::state::State;
use super::CommonComponent;
use super::ParsedComponentFields;
use proc_macro2::TokenStream as TokenStream2;
use syn::{parse2, Error, Ident, ItemStruct, Result};

/// Parsed representation of an atomic component macro input.
pub struct Component {
    pub common: CommonComponent,
    pub state: State,
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

        // Atomic components must not declare inner components.
        if !components.is_empty() {
            return Err(Error::new_spanned(
                &components[0].ident,
                "Atomic components cannot define #[components] fields",
            ));
        }

        // Check that state is defined
        if state.is_empty() {
            return Err(Error::new_spanned(&component, "No state definition found"));
        }

        // Build shared component metadata (rt_engine + top-level ports).
        let generics = component.generics.clone();
        let state_generics = filter_generics(&state, &generics);
        let state_ident = Ident::new(&format!("{ident}State"), ident.span());
        let common = CommonComponent::new(
            ident,
            generics,
            inputs,
            outputs,
            args,
            "unknown atomic component argument",
        )?;
        let state = State::new(state, state_ident, state_generics);

        Ok(Component { common, state })
    }

    pub fn quote(&self) -> TokenStream2 {
        let ident = &self.common.ident;

        // Prepare identifiers for code generation
        let input_ident = &self.common.input.ident;
        let output_ident = &self.common.output.ident;
        let state_ident = &self.state.ident;
        let state_fields = self.state.field_idents();
        let state_tys = self.state.field_tys();

        // Extract generics for impl
        let (impl_generics, ty_generics, _) = self.common.generics.split_for_impl();
        let (_, input_generics, _) = &self.common.input.generics.split_for_impl();
        let (_, output_generics, _) = &self.common.output.generics.split_for_impl();
        let (_, state_generics, _) = &self.state.generics.split_for_impl();

        // Generate input, output, and state structs
        let is_bagmux = self.common.rt_engine.is_some();
        let input_struct = self.common.input.quote(is_bagmux);
        let output_struct = self.common.output.quote(is_bagmux);
        let state_struct = self.state.quote();

        // Generate rt_engine code if defined
        let rt_engine_impl = self.common.quote_rt_engine();

        // Component trait implementation
        let component_impl = impl_component(
            ident,
            &input_ident,
            &output_ident,
            &self.common.generics,
            input_generics,
            output_generics,
        );

        // Generate the expanded code
        let expanded = quote::quote! {
            #input_struct
            #output_struct
            #state_struct
            #rt_engine_impl
            pub struct #ident #impl_generics{
                pub input: #input_ident #input_generics,
                pub output: #output_ident #output_generics,
                pub t_last: f64,
                pub t_next: f64,
                pub state: #state_ident #state_generics,
            }
            impl #impl_generics #ident #ty_generics {
                #[inline]
                pub fn build(#(#state_fields: #state_tys),*) -> Self {
                    Self {
                        input: #input_ident::new(),
                        output: #output_ident::new(),
                        t_last: 0.0,
                        t_next: f64::INFINITY,
                        state: #state_ident::new(#(#state_fields),*),
                    }
                }
            }
            #component_impl
            unsafe impl #impl_generics ::xdevs::traits::PartialAtomic for #ident #ty_generics{
                type State = #state_ident #state_generics;
            }
            unsafe impl #impl_generics ::xdevs::traits::AbstractSimulator for #ident #ty_generics{
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
                fn lambda(&mut self, t: f64) {
                    if t >= ::xdevs::traits::Component::get_t_next(self) {
                        // execute atomic model's lambda if applies
                        <Self as ::xdevs::Atomic>::lambda(&self.state, &mut self.output);
                    }
                }
                #[inline]
                fn delta(&mut self, t: f64) -> f64 {
                    let mut t_next = ::xdevs::traits::Component::get_t_next(self);
                    if !::xdevs::traits::Bag::is_empty(&self.input) {
                        if t >= t_next {
                            // confluent transition
                            <Self as ::xdevs::Atomic>::delta_conf(&mut self.state, &self.input);
                        } else {
                            // external transition
                            let e = t - ::xdevs::traits::Component::get_t_last(self);
                            <Self as ::xdevs::Atomic>::delta_ext(&mut self.state, e, &self.input);
                        }
                        // clear input events
                        ::xdevs::traits::Component::clear_input(self);
                    } else if t >= t_next {
                        // internal transition
                        <Self as ::xdevs::Atomic>::delta_int(&mut self.state);
                    } else {
                        return t_next; // nothing to do
                    }
                    // clear output events
                    ::xdevs::traits::Component::clear_output(self);
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
