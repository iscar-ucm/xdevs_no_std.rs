use proc_macro::TokenStream;

mod atomic;
mod component;
mod coupled;
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
    // assert that it is a structure
    let fields = match input.data {
        syn::Data::Struct(s) => match s.fields {
            syn::Fields::Named(fields) => fields,
            _ => panic!("Components can only be derived for structs with named fields"),
        },
        _ => panic!("Components can only be derived for structs"),
    };
    todo!()
}
