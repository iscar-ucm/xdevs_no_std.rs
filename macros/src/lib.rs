use proc_macro::TokenStream;
use syn::{parse_macro_input, Error};

mod component;
mod derive;

#[proc_macro_attribute]
pub fn atomic(args: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as component::ComponentArgs);
    let atomic_component = component::atomic::Atomic::parse(args, item.into());
    match atomic_component {
        Ok(component) => component.quote().into(),
        Err(err) => err.to_compile_error().into(),
    }
}

#[proc_macro_attribute]
pub fn coupled(args: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as component::ComponentArgs);
    let coupled_component = component::coupled::Coupled::parse(args, item.into());
    match coupled_component {
        Ok(component) => component.quote().into(),
        Err(err) => err.to_compile_error().into(),
    }
}

#[proc_macro_derive(Bag)]
pub fn derive_bag(input: TokenStream) -> TokenStream {
    let input: syn::DeriveInput = syn::parse_macro_input!(input);
    match derive::derive_bag(input) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

#[proc_macro_derive(BagMux)]
pub fn derive_bagmux(input: TokenStream) -> TokenStream {
    let input: syn::DeriveInput = syn::parse_macro_input!(input);
    match derive::derive_bagmux(input) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

/// Function to combine errors when parsing
pub(crate) fn combine_err(acc: &mut Option<Error>, err: Error) {
    match acc {
        Some(e) => e.combine(err),
        None => *acc = Some(err),
    }
}
