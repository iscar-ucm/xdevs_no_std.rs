use heck::ToSnakeCase;
use heck::ToUpperCamelCase;
use proc_macro2::Ident;
use proc_macro2::TokenStream as TokenStream2;
use syn::{Data, DeriveInput, Field, Fields, Type};

pub fn derive_bag(input: DeriveInput) -> syn::Result<TokenStream2> {
    let ident = input.ident;
    let generics = input.generics;

    let fields = match input.data {
        Data::Struct(data) => data.fields,
        _ => {
            return Err(syn::Error::new_spanned(
                ident,
                "Bag can only be derived for structs",
            ))
        }
    };

    let accesses: Vec<TokenStream2> = match &fields {
        Fields::Named(fields) => fields
            .named
            .iter()
            .map(|field| {
                let field_ident = field.ident.as_ref().expect("named field must have ident");
                quote::quote!(self.#field_ident)
            })
            .collect(),
        Fields::Unnamed(fields) => fields
            .unnamed
            .iter()
            .enumerate()
            .map(|(i, _)| {
                let index = syn::Index::from(i);
                quote::quote!(self.#index)
            })
            .collect(),
        Fields::Unit => Vec::new(),
    };

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let is_empty_body = if accesses.is_empty() {
        quote::quote! {
            true
        }
    } else {
        quote::quote! {
            #(#accesses.is_empty())&&*
        }
    };

    Ok(quote::quote! {
        unsafe impl #impl_generics ::xdevs::traits::Bag for #ident #ty_generics #where_clause {
            #[inline]
            fn is_empty(&self) -> bool {
                #is_empty_body
            }

            #[inline]
            fn clear(&mut self) {
                #( #accesses.clear(); )*
            }
        }
    })
}

pub fn derive_bagmux(input: DeriveInput) -> syn::Result<TokenStream2> {
    // Prepare the struct fields and generics
    let ident = input.ident;
    let snake_case_ident = Ident::new(&ident.to_string().to_snake_case(), ident.span());
    let private_mod_ident = Ident::new(
        &format!("_xdevs_no_std_{}_bagmux", snake_case_ident),
        ident.span(),
    );
    let generics = input.generics;

    let fields = match input.data {
        Data::Struct(data) => data.fields,
        _ => {
            return Err(syn::Error::new_spanned(
                ident,
                "Bag can only be derived for structs",
            ))
        }
    };

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    match fields {
        Fields::Unnamed(_) => Err(syn::Error::new_spanned(
            ident,
            "BagMux cannot be derived for tuple structs",
        )),
        Fields::Unit => {
            Ok(quote::quote! {
            unsafe impl #impl_generics ::xdevs::traits::BagMux for #ident #ty_generics #where_clause {
                type Mux = ();
                fn enum_to_input(&mut self, _input: Self::Mux) {
                    // No ports to receive input
                }
                fn output_to_enum(&self, _output_fn: impl FnMut(Self::Mux)) {
                    // No ports to produce output
                }
            }})
        }

        Fields::Named(fields) => {
            // Input match arms and output propagations
            let variants: Vec<TokenStream2> = fields
                .named
                .iter()
                .map(|info| {
                    let variant = to_pascal_case_ident(
                        info.ident.as_ref().expect("named field must have ident"),
                    );
                    let ty = &info.ty;
                    quote::quote! { #variant(<#ty as ::xdevs::traits::AsPort>::Item) }
                })
                .collect();

            let match_arms: Vec<TokenStream2> = fields
                .named
                .iter()
                .map(|info| {
                    let variant = to_pascal_case_ident(
                        info.ident.as_ref().expect("named field must have ident"),
                    );
                    let arm_body = expand_input_match_arm(info);
                    quote::quote! {
                        Self::Mux::#variant(value) => #arm_body
                    }
                })
                .collect();

            let propagations: Vec<TokenStream2> = fields
                .named
                .iter()
                .map(|info| {
                    let variant = to_pascal_case_ident(
                        info.ident.as_ref().expect("named field must have ident"),
                    );
                    let for_body = expand_output_for(info, variant);
                    quote::quote! {
                        #for_body
                    }
                })
                .collect();

            let enum_to_input_body = quote::quote! {
                fn enum_to_input(&mut self, input: Self::Mux) {
                    match input {
                        #(#match_arms),*
                    }
                }
            };

            let output_to_enum_body = quote::quote! {
                fn output_to_enum(&self, mut output_fn: impl FnMut(Self::Mux)) {
                    #(#propagations)*
                }
            };

            Ok(quote::quote! {
                unsafe impl #impl_generics ::xdevs::traits::BagMux for #ident #ty_generics #where_clause {
                    type Mux = #private_mod_ident::PortMux #ty_generics;
                    #enum_to_input_body
                    #output_to_enum_body
                }

                mod #private_mod_ident {
                    use super::*;
                    /// Auto-generated enum for top-level channel communication.
                    #[derive(Clone)]
                    pub enum PortMux #ty_generics #where_clause {
                        #(#variants),*
                    }
                }
            })
        }
    }
}

/// Converts a snake_case identifier to PascalCase.
fn to_pascal_case_ident(ident: &Ident) -> Ident {
    Ident::new(&ident.to_string().to_upper_camel_case(), ident.span())
}

/// Generate a match arm for the input enum to add received values to the corresponding input port.
fn expand_input_match_arm(info: &Field) -> TokenStream2 {
    fn input_match_arm_body(ty: &Type) -> TokenStream2 {
        match ty {
            Type::Path(_) => {
                quote::quote! {
                    port.add_value(value).unwrap();
                }
            }
            Type::Array(array) => {
                let elem_ty = &array.elem;
                let body = input_match_arm_body(elem_ty);
                quote::quote! {
                    let (index, value) = value;
                    if let Some(port) = port.get_mut(index)
                    {
                        #body
                    }
                }
            }
            _ => {
                quote::quote! {
                    compile_error!("unsupported input port type; expected array or Port");
                }
            }
        }
    }
    let field = &info.ident;
    let ty = &info.ty;
    let body = input_match_arm_body(ty);
    quote::quote! {
        {
            let port = &mut self.#field;
            {
                #body
            }
        }
    }
}

/// Generate a for loop for the output enum to publish values from the corresponding output port.
fn expand_output_for(info: &Field, variant: Ident) -> TokenStream2 {
    fn output_for_body(ty: &Type, variant: Ident, from_array: bool) -> TokenStream2 {
        match ty {
            Type::Path(_) => {
                if from_array {
                    quote::quote! {
                        for value in port.get_values() {
                            output_fn(Self::Mux::#variant((index, value.clone())));
                        }
                    }
                } else {
                    quote::quote! {
                        for value in port.get_values() {
                            output_fn(Self::Mux::#variant(value.clone()));
                        }
                    }
                }
            }
            Type::Array(array) => {
                let body = output_for_body(&array.elem, variant, true);
                quote::quote! {
                    for (index, port) in port.iter().enumerate() {
                        #body
                    }
                }
            }
            _ => {
                quote::quote! {
                    compile_error!("unsupported output port type; expected array or Port");
                }
            }
        }
    }
    let field = &info.ident;
    let ty = &info.ty;
    let body = output_for_body(ty, variant, false);
    quote::quote! {
        let port = &self.#field;
        {
            #body
        }
    }
}
