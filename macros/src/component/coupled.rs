use crate::component::Component;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;

mod components;
mod coupling;

pub use components::Components;
pub use coupling::Couplings;

pub struct Coupled {
    pub components: Components,
    pub couplings: Option<Couplings>,
}

impl Coupled {
    pub fn quote(&self, component: &Component) -> TokenStream2 {
        let coupled_ident = &component.ident;
        let component = self.components.ident();
        let (eoc, xic) = if let Some(couplings) = &self.couplings {
            couplings.quote()
        } else {
            (vec![], vec![])
        };

        quote! {
            unsafe impl xdevs::aux::AbstractSimulator for #coupled_ident {
                #[inline]
                fn start(&mut self, t_start: f64) -> f64 {
                    // set t_last to t_start
                    xdevs::aux::Component::set_t_last(self, t_start);
                    // get minimum t_next from all components
                    let mut t_next = f64::INFINITY;
                    #(t_next = f64::min(t_next, xdevs::aux::AbstractSimulator::start(&mut self.#component, t_start));)*
                    // set t_next to minimum t_next
                    xdevs::aux::Component::set_t_next(self, t_next);

                    t_next
                }

                #[inline]
                fn stop(&mut self, t_stop: f64) {
                    // stop all components
                    #(xdevs::aux::AbstractSimulator::stop(&mut self.#component, t_stop);)*
                    // set t_last to t_stop and t_next to infinity
                    xdevs::aux::Component::set_t_last(self, t_stop);
                    xdevs::aux::Component::set_t_next(self, f64::INFINITY);
                }

                #[inline]
                fn lambda(&mut self, t: f64) {
                    if t >= xdevs::aux::Component::get_t_next(self) {
                        // propagate lambda to all components
                        #(xdevs::aux::AbstractSimulator::lambda(&mut self.#component, t);)*
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
                    #(t_next = f64::min(t_next, xdevs::aux::AbstractSimulator::delta(&mut self.#component, t));)*
                    // clear input and output events
                    xdevs::aux::Component::clear_output(self);
                    xdevs::aux::Component::clear_input(self);
                    // set t_last to t and t_next to minimum t_next
                    xdevs::aux::Component::set_t_last(self, t);
                    xdevs::aux::Component::set_t_next(self, t_next);

                    t_next
                }
            }
        }
    }
}
