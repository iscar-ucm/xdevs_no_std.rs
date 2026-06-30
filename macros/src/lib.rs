use proc_macro::TokenStream;
use syn::{parse, parse_macro_input, Error, Ident, ItemStruct};

mod coupled;
mod derive;
mod devstone;
mod rt_engine;
mod to_component;

/// Macro to generate coupled DEVS components.
#[proc_macro_attribute]
pub fn coupled(args: TokenStream, item: TokenStream) -> TokenStream {
    let args = proc_macro2::TokenStream::from(args);
    if !args.is_empty() {
        return syn::Error::new_spanned(args, "#[coupled] does not accept arguments")
            .to_compile_error()
            .into();
    }
    let item = parse_macro_input!(item as syn::ItemStruct);

    match coupled::expand(item) {
        Ok(component) => component.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

/// Macro to generate DEVS components.
#[proc_macro_attribute]
pub fn to_component(args: TokenStream, item: TokenStream) -> TokenStream {
    let args = proc_macro2::TokenStream::from(args);
    if !args.is_empty() {
        return syn::Error::new_spanned(args, "#[to_component] does not accept arguments")
            .to_compile_error()
            .into();
    }
    let item2 = item.clone();

    // Try parsing as a struct first (coupled model)
    if let Ok(item_struct) = syn::parse::<syn::ItemStruct>(item2) {
        return match to_component::expand_struct(item_struct) {
            Ok(component) => component.into(),
            Err(err) => err.to_compile_error().into(),
        };
    }

    // Then try parsing as an enum (enum-based model)
    let item2 = item.clone();
    if let Ok(item_enum) = syn::parse::<syn::ItemEnum>(item2) {
        return match to_component::expand_enum(item_enum) {
            Ok(component) => component.into(),
            Err(err) => err.to_compile_error().into(),
        };
    }

    let err = Error::new(
        proc_macro2::Span::call_site(),
        "#[to_component] requires a struct (for coupled models) or an enum (for enum-based models)",
    );
    err.to_compile_error().into()
}

/// Macro to generate RT engine components
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

// Function to combine errors when parsing
pub(crate) fn combine_err(acc: &mut Option<Error>, err: Error) {
    match acc {
        Some(e) => e.combine(err),
        None => *acc = Some(err),
    }
}

/// DEVStone macros — ref version (default, no alloc needed)
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

/// DEVStone macros — box version (needs alloc feature)
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

/// Generate the input and output wrapper structs, and modify the original struct's fields to be of Simulator types
fn build_component_structs(mut item: ItemStruct) -> (ItemStruct, ItemStruct, ItemStruct) {
    let item_ident = &item.ident;

    // Generate the input wrapper struct
    let item_input_ident = Ident::new(&format!("{}Input", item_ident), item_ident.span());
    let input_struct = {
        let mut s = item.clone();
        s.attrs = Vec::new();
        s.ident = item_input_ident.clone();
        if let syn::Fields::Named(fields) = &mut s.fields {
            for field in &mut fields.named {
                let original_ty = field.ty.clone();
                field.ty = syn::parse_quote! {
                    <#original_ty as ::xdevs::Component>::Input
                };
            }
        }
        s
    };

    // Generate the output wrapper struct
    let item_output_ident = Ident::new(&format!("{}Output", item_ident), item_ident.span());
    let output_struct = {
        let mut s = item.clone();
        s.attrs = Vec::new();
        s.ident = item_output_ident.clone();
        if let syn::Fields::Named(fields) = &mut s.fields {
            for field in &mut fields.named {
                let original_ty = field.ty.clone();
                field.ty = syn::parse_quote! {
                    <#original_ty as ::xdevs::Component>::Output
                };
            }
        }
        s
    };

    // Convert the struct's own fields to Simulator types
    if let syn::Fields::Named(fields) = &mut item.fields {
        for field in &mut fields.named {
            let ty = &field.ty;
            field.ty = syn::parse_quote! {
                <#ty as ::xdevs::simulation::SimpleSimulable>::Simulator
            };
        }
    }

    (input_struct, output_struct, item)
}
