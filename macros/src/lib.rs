use proc_macro::TokenStream;

mod component;

#[proc_macro]
pub fn component(input: TokenStream) -> TokenStream {
    let component: component::Component = syn::parse_macro_input!(input);
    component.quote().into()
}

// #[proc_macro]
// pub fn atomic(input: TokenStream) -> TokenStream {
//     let atomic_meta: atomic::AtomicMeta = syn::parse_macro_input!(input);
//     atomic_meta.quote().into()
// }

// #[proc_macro]
// pub fn coupled(input: TokenStream) -> TokenStream {
//     let coupled_meta: coupled::CoupledMeta = syn::parse_macro_input!(input);
//     coupled_meta.quote().into()
// }
