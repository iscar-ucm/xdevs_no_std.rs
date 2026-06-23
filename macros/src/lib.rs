use proc_macro::TokenStream;
use syn::{parse, parse_macro_input, Error};

mod coupled;
mod derive;
mod devstone;
mod modelenum;
mod rt_engine;

// Main macro to generate coupled DEVS models
#[proc_macro_attribute]
pub fn coupled(_args: TokenStream, item: TokenStream) -> TokenStream {
    let item = parse_macro_input!(item as syn::ItemStruct);

    match coupled::expand(item) {
        Ok(component) => component.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

// Macro to generate enum-based DEVS components
#[proc_macro_attribute]
pub fn modelenum(_args: TokenStream, item: TokenStream) -> TokenStream {
    let item = parse_macro_input!(item as syn::ItemEnum);

    match modelenum::expand(item) {
        Ok(component) => component.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

// Macro to generate RT engine components
#[proc_macro_attribute]
pub fn rt_engine(args: TokenStream, item: TokenStream) -> TokenStream {
    let args = match parse::<rt_engine::RtEngineArgs>(args) {
        Ok(args) => args,
        Err(err) => {
            let err = err.to_compile_error();
            let item = proc_macro2::TokenStream::from(item);
            return quote::quote! {
                #item
                #err
            }
            .into();
        }
    };
    let item = parse_macro_input!(item as syn::ItemImpl);

    match rt_engine::expand(args, item) {
        Ok(component) => component.into(),
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

// DEVStone macros — ref version (default, no alloc needed)
#[proc_macro]
pub fn generate_li(input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(input as devstone::GenerateArgs);

    match devstone::expand_li(args) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

#[proc_macro]
pub fn generate_hi(input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(input as devstone::GenerateArgs);

    match devstone::expand_hi(args) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

#[proc_macro]
pub fn generate_ho(input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(input as devstone::GenerateArgs);

    match devstone::expand_ho(args) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

// DEVStone macros — box version (needs alloc feature)
#[proc_macro]
pub fn generate_li_box(input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(input as devstone::GenerateArgs);

    match devstone::expand_li_box(args) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

#[proc_macro]
pub fn generate_hi_box(input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(input as devstone::GenerateArgs);

    match devstone::expand_hi_box(args) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

#[proc_macro]
pub fn generate_ho_box(input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(input as devstone::GenerateArgs);

    match devstone::expand_ho_box(args) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}
