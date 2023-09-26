use proc_macro::TokenStream;

mod atomic;
mod component;
mod port;

#[proc_macro]
pub fn atomic(input: TokenStream) -> TokenStream {
    let atomic_meta: atomic::AtomicMeta = syn::parse_macro_input!(input);
    atomic_meta.quote().into()
}
