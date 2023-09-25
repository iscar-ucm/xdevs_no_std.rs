use proc_macro::TokenStream;
use quote::quote;
use syn::parse::Parser;

mod component;
mod port;

#[proc_macro_attribute]
pub fn component(attr: TokenStream, input: TokenStream) -> TokenStream {
    let comp_meta: component::ComponentMeta = syn::parse_macro_input!(attr);
    let mut ast: syn::DeriveInput = syn::parse_macro_input!(input);

    let comp_name = ast.ident.to_string(); // name of the struct

    let inputs_name = syn::Ident::new(&format!("{comp_name}Inputs"), ast.ident.span());
    let outputs_name = syn::Ident::new(&format!("{comp_name}Outputs"), ast.ident.span());
    let inputs = comp_meta.input.quote(&inputs_name);
    let outputs = comp_meta.output.quote(&outputs_name);

    match &mut ast.data {
        syn::Data::Struct(struct_data) => match &mut struct_data.fields {
            syn::Fields::Named(fields) => {
                fields.named.push(
                    syn::Field::parse_named
                        .parse2(quote! {
                            pub input: #inputs_name
                        })
                        .unwrap(),
                );
                fields.named.push(
                    syn::Field::parse_named
                        .parse2(quote! {
                            pub output: #outputs_name
                        })
                        .unwrap(),
                );
            }
            _ => {}
        },
        _ => panic!("xdevs_no_std_macros::component only supports structs"),
    }

    quote! {
        #inputs
        #outputs
        #ast
    }
    .into()
}
