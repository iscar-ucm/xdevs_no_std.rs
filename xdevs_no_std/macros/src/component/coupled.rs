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
            unsafe impl xdevs::traits::AbstractSimulator for #coupled_ident {
                #[inline]
                fn start(&mut self, t_start: ::embassy_time::Instant) -> ::embassy_time::Instant {
                    // set t_last to t_start
                    xdevs::traits::Component::set_t_last(self, t_start);
                    // get minimum t_next from all components
                    let mut t_next = ::embassy_time::Instant::MAX;
                    #(t_next = ::embassy_time::Instant::min(t_next, xdevs::traits::AbstractSimulator::start(&mut self.#component, t_start));)*
                    // set t_next to minimum t_next
                    xdevs::traits::Component::set_t_next(self, t_next);

                    t_next
                }

                #[inline]
                fn stop(&mut self, t_stop: ::embassy_time::Instant) {
                    // stop all components
                    #(xdevs::traits::AbstractSimulator::stop(&mut self.#component, t_stop);)*
                    // set t_last to t_stop and t_next to infinity
                    xdevs::traits::Component::set_t_last(self, t_stop);
                    xdevs::traits::Component::set_t_next(self, ::embassy_time::Instant::MAX);
                }

                #[inline]
                fn lambda(&mut self, t: ::embassy_time::Instant) {
                    if t >= xdevs::traits::Component::get_t_next(self) {
                        // propagate lambda to all components
                        #(xdevs::traits::AbstractSimulator::lambda(&mut self.#component, t);)*
                        // propagate EOCs
                        #(#eoc);*
                    }
                }

                #[inline]
                fn delta(&mut self, t: ::embassy_time::Instant) -> ::embassy_time::Instant {
                    // propagate EICs and ICs
                     #(#xic);*
                    // get minimum t_next from all components after executing their delta
                    let mut t_next = ::embassy_time::Instant::MAX;
                    #(t_next = ::embassy_time::Instant::min(t_next, xdevs::traits::AbstractSimulator::delta(&mut self.#component, t));)*
                    // clear input and output events
                    xdevs::traits::Component::clear_output(self);
                    xdevs::traits::Component::clear_input(self);
                    // set t_last to t and t_next to minimum t_next
                    xdevs::traits::Component::set_t_last(self, t);
                    xdevs::traits::Component::set_t_next(self, t_next);

                    t_next
                }
            }
        }
    }
}
