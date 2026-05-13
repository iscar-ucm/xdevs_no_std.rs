extern crate proc_macro;

mod expand;
use expand::*;
use proc_macro::TokenStream;
use syn::{
    Result, Token,
    parse::{Parse, ParseStream},
    parse_macro_input,
};

struct GenerateArgs {
    width: syn::LitInt,
    depth: syn::LitInt,
}

impl Parse for GenerateArgs {
    fn parse(input: ParseStream) -> Result<Self> {
        let width: syn::LitInt = input.parse()?;
        input.parse::<Token![,]>()?;
        let depth: syn::LitInt = input.parse()?;

        if !input.is_empty() {
            return Err(input.error("expected only two arguments: width, depth"));
        }

        Ok(Self { width, depth })
    }
}

#[proc_macro]
pub fn generate_li(input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(input as GenerateArgs);

    match expand_li(args) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

#[proc_macro]
pub fn generate_hi(input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(input as GenerateArgs);

    match expand_hi(args) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

#[proc_macro]
pub fn generate_ho(input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(input as GenerateArgs);

    match expand_hi(args) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}
