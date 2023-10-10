use proc_macro::TokenStream;

mod atomic;
mod component;
mod coupled;
mod coupling;
mod port;

#[proc_macro]
pub fn atomic(input: TokenStream) -> TokenStream {
    let atomic_meta: atomic::AtomicMeta = syn::parse_macro_input!(input);
    atomic_meta.quote().into()
}

#[proc_macro]
pub fn coupled(input: TokenStream) -> TokenStream {
    let coupled_meta: coupled::CoupledMeta = syn::parse_macro_input!(input);
    coupled_meta.quote().into()
}

#[proc_macro_derive(Components)]
pub fn derive_components(item: TokenStream) -> TokenStream {
    // parse the input tokens into a syntax tree
    let input: syn::DeriveInput = syn::parse_macro_input!(item);
    // get the name of the struct and the fields
    let struct_ident = input.ident;
    let fields = match input.data {
        syn::Data::Struct(s) => match s.fields {
            syn::Fields::Named(fields) => fields,
            _ => panic!("Components can only be derived for structs with named fields"),
        },
        _ => panic!("Components can only be derived for structs"),
    };
    // get the names of the fields
    let field_names = fields
        .named
        .iter()
        .map(|f| f.ident.as_ref().unwrap())
        .collect::<Vec<_>>();
    quote::quote!(
        unsafe impl xdevs::simulator::AbstractSimulator for #struct_ident {
            fn start(&mut self, t_start: f64) -> f64 {
                let mut t_next = f64::INFINITY;
                #(t_next = f64::min(t_next, xdevs::simulator::AbstractSimulator::start(&mut self.#field_names, t_start));)*
                t_next
            }

            fn stop(&mut self, t_stop: f64) {
                #(xdevs::simulator::AbstractSimulator::stop(&mut self.#field_names, t_stop);)*
            }

            fn lambda(&mut self, t: f64) {
                #(xdevs::simulator::AbstractSimulator::lambda(&mut self.#field_names, t);)*
            }

            fn delta(&mut self, t: f64) -> f64 {
                let mut t_next = f64::INFINITY;
                #(t_next = f64::min(t_next, xdevs::simulator::AbstractSimulator::delta(&mut self.#field_names, t));)*
                t_next
            }
        }
    )
    .into()
}
