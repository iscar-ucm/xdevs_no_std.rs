use super::port::Ports;
use super::state::State;
use super::Field;
use proc_macro2::TokenStream as TokenStream2;
use syn::{Error, Ident, ItemStruct};

pub struct Component {
    pub ident: Ident,
    pub state: State,
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
    fn state_ident(&self) -> syn::Ident {
        syn::Ident::new(&format!("{}State", self.ident), self.ident.span())
    }

    pub fn parse(item: TokenStream2) -> syn::Result<Self> {
        let component: ItemStruct = syn::parse2(item).unwrap();

        let ident = component.ident.clone();
        let mut last_attr = None;
        let mut state = State::new(Vec::new());
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
                if attr.path().is_ident("state") {
                    let field_ident = field.ident.clone().unwrap();
                    let field_ty = field.ty.clone();
                    state.add_field(Field {
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

        // Check that state is defined
        if state.is_empty() {
            return Err(Error::new_spanned(&component, "No state definition found"));
        }

        Ok(Component {
            ident,
            state,
            inputs,
            outputs,
        })
    }

    pub fn quote(&self) -> TokenStream2 {
        let ident = &self.ident;

        // Prepare identifiers for code generation
        let input_ident = &self.input_ident();
        let output_ident = &self.output_ident();
        let state_ident = &self.state_ident();
        let state_fields = self.state.field_idents();
        let state_tys = self.state.field_tys();
        let input_struct = self.inputs.quote(input_ident);
        let output_struct = self.outputs.quote(output_ident);
        let state_struct = self.state.quote(state_ident);

        // Generate the expanded code
        let expanded = quote::quote! {
            #input_struct
            #output_struct
            #state_struct
            pub struct #ident {
                pub input: #input_ident,
                pub output: #output_ident,
                pub t_last: f64,
                pub t_next: f64,
                pub state: #state_ident,
            }
            impl #ident {
                #[inline]
                pub fn new(#(#state_fields: #state_tys),*) -> Self {
                    Self {
                        input: #input_ident::new(),
                        output: #output_ident::new(),
                        t_last: 0.0,
                        t_next: f64::INFINITY,
                        state: #state_ident::new(#(#state_fields),*),
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

            unsafe impl xdevs::traits::PartialAtomic for #ident {
                type State = #state_ident;
            }
            unsafe impl xdevs::traits::AbstractSimulator for #ident {
                #[inline]
                fn start(&mut self, t_start: f64) -> f64 {
                    // set t_last to t_start
                    xdevs::traits::Component::set_t_last(self, t_start);
                    // start state and get t_next from ta
                    <Self as xdevs::Atomic>::start(&mut self.state);
                    let t_next = t_start + <Self as xdevs::Atomic>::ta(&self.state);
                    xdevs::traits::Component::set_t_next(self, t_next);

                    t_next
                }
                #[inline]
                fn stop(&mut self, t_stop: f64) {
                    // stop state
                    <Self as xdevs::Atomic>::stop(&mut self.state);
                    // set t_last to t_stop and t_next to infinity
                    xdevs::traits::Component::set_t_last(self, t_stop);
                    xdevs::traits::Component::set_t_next(self, f64::INFINITY);
                }
                #[inline]
                fn lambda(&mut self, t: f64) {
                    if t >= xdevs::traits::Component::get_t_next(self) {
                        // execute atomic model's lambda if applies
                        <Self as xdevs::Atomic>::lambda(&self.state, &mut self.output);
                    }
                }
                #[inline]
                fn delta(&mut self, t: f64) -> f64 {
                    let mut t_next = xdevs::traits::Component::get_t_next(self);
                    if !xdevs::traits::Bag::is_empty(&self.input) {
                        if t >= t_next {
                            // confluent transition
                            <Self as xdevs::Atomic>::delta_conf(&mut self.state, &self.input);
                        } else {
                            // external transition
                            let e = t - xdevs::traits::Component::get_t_last(self);
                            <Self as xdevs::Atomic>::delta_ext(&mut self.state, e, &self.input);
                        }
                        // clear input events
                        xdevs::traits::Component::clear_input(self);
                    } else if t >= t_next {
                        // internal transition
                        <Self as xdevs::Atomic>::delta_int(&mut self.state);
                    } else {
                        return t_next; // nothing to do
                    }
                    // clear output events
                    xdevs::traits::Component::clear_output(self);
                    // get t_next from ta and set new t_last and t_next
                    t_next = t + <Self as xdevs::Atomic>::ta(&self.state);
                    xdevs::traits::Component::set_t_last(self, t);
                    xdevs::traits::Component::set_t_next(self, t_next);

                    t_next
                }
            }
        };

        expanded.into()
    }
}
