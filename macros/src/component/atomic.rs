use super::Component;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    Type,
};

pub enum State {
    Type(Type),
}

impl State {
    pub fn ident(&self) -> Vec<TokenStream2> {
        match self {
            Self::Type(_) => vec![quote!(state)],
        }
    }
    pub fn ty(&self) -> Vec<TokenStream2> {
        match self {
            Self::Type(ty) => vec![quote!(state: #ty)],
        }
    }
}

impl Parse for State {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        return Ok(Self::Type(input.parse()?));
    }
}

impl State {
    pub(crate) fn quote(&self, component: &Component) -> TokenStream2 {
        let atomic_ident = &component.ident;
        let state_ty = match self {
            Self::Type(ty) => ty,
        };

        quote! {
            unsafe impl xdevs::aux::PartialAtomic for #atomic_ident {
                type State = #state_ty;
            }
            unsafe impl xdevs::aux::AbstractSimulator for #atomic_ident {
                #[inline]
                fn start(&mut self, t_start: f64) -> f64 {
                    // set t_last to t_start
                    xdevs::aux::Component::set_t_last(self, t_start);
                    // start state and get t_next from ta
                    <Self as xdevs::Atomic>::start(&mut self.state);
                    let t_next = t_start + <Self as xdevs::Atomic>::ta(&self.state);
                    xdevs::aux::Component::set_t_next(self, t_next);

                    t_next
                }
                #[inline]
                fn stop(&mut self, t_stop: f64) {
                    // stop state
                    <Self as xdevs::Atomic>::stop(&mut self.state);
                    // set t_last to t_stop and t_next to infinity
                    xdevs::aux::Component::set_t_last(self, t_stop);
                    xdevs::aux::Component::set_t_next(self, f64::INFINITY);
                }
                #[inline]
                fn lambda(&mut self, t: f64) {
                    if t >= xdevs::aux::Component::get_t_next(self) {
                        // execute atomic model's lambda if applies
                        <Self as xdevs::Atomic>::lambda(&self.state, &mut self.output);
                    }
                }
                #[inline]
                fn delta(&mut self, t: f64) -> f64 {
                    let mut t_next = xdevs::aux::Component::get_t_next(self);
                    if !xdevs::aux::Port::is_empty(&self.input) {
                        if t >= t_next {
                            // confluent transition
                            <Self as xdevs::Atomic>::delta_conf(&mut self.state, &self.input);
                        } else {
                            // external transition
                            let e = t - xdevs::aux::Component::get_t_last(self);
                            <Self as xdevs::Atomic>::delta_ext(&mut self.state, e, &self.input);
                        }
                        // clear input events
                        xdevs::aux::Component::clear_input(self);
                    } else if t >= t_next {
                        // internal transition
                        <Self as xdevs::Atomic>::delta_int(&mut self.state);
                    } else {
                        return t_next; // nothing to do
                    }
                    // clear output events
                    xdevs::aux::Component::clear_output(self);
                    // get t_next from ta and set new t_last and t_next
                    t_next = t + <Self as xdevs::Atomic>::ta(&self.state);
                    xdevs::aux::Component::set_t_last(self, t);
                    xdevs::aux::Component::set_t_next(self, t_next);

                    t_next
                }
            }
        }
    }
}
