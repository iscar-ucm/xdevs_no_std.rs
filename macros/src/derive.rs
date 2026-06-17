use heck::ToSnakeCase;
use heck::ToUpperCamelCase;
use proc_macro2::TokenStream as TokenStream2;
use syn::{Data, DeriveInput, Error, Field, Fields, Ident, Index, Result, Type};

use crate::combine_err;

pub fn derive_bag(input: DeriveInput) -> Result<TokenStream2> {
    let ident = input.ident;
    let generics = input.generics;

    let fields = match input.data {
        Data::Struct(data) => data.fields,
        _ => {
            return Err(Error::new_spanned(
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
                let index = Index::from(i);
                quote::quote!(self.#index)
            })
            .collect(),
        Fields::Unit => Vec::new(),
    };

    let build_body = match &fields {
        Fields::Named(fields) => {
            let build_fields = fields.named.iter().map(|field| {
                let field_ident = field.ident.as_ref().expect("named field must have ident");
                let field_ty = &field.ty;
                quote::quote!(#field_ident: <#field_ty as ::xdevs::port::Bag>::build())
            });
            quote::quote! {
                Self {
                    #(#build_fields),*
                }
            }
        }
        Fields::Unnamed(fields) => {
            let build_elems = fields.unnamed.iter().map(|field| {
                let field_ty = &field.ty;
                quote::quote!(<#field_ty as ::xdevs::port::Bag>::build())
            });
            quote::quote! {
                Self(
                    #(#build_elems),*
                )
            }
        }
        Fields::Unit => quote::quote! { Self },
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
        unsafe impl #impl_generics ::xdevs::port::Bag for #ident #ty_generics #where_clause {
            #[inline]
            fn build() -> Self {
                #build_body
            }

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

pub fn derive_bagmux(input: DeriveInput) -> Result<TokenStream2> {
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
            return Err(Error::new_spanned(
                ident,
                "Bag can only be derived for structs",
            ))
        }
    };

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    match fields {
        Fields::Unnamed(_) => Err(Error::new_spanned(
            ident,
            "BagMux cannot be derived for tuple structs",
        )),
        Fields::Unit => {
            Ok(quote::quote! {
            unsafe impl #impl_generics ::xdevs::port::BagMux for #ident #ty_generics #where_clause {
                type Mux = ();
                fn inject_event(&mut self, _event: Self::Mux) {
                    // No ports to receive input
                }
                fn eject_events(&self, _ejector: impl FnMut(Self::Mux)) {
                    // No ports to produce output
                }
            }})
        }

        Fields::Named(fields) => {
            // Define the accumulator
            let mut acc: Option<Error> = None;

            // Input match arms and output propagations
            let variants: Vec<TokenStream2> = fields
                .named
                .iter()
                .map(|info| {
                    let variant = to_pascal_case_ident(
                        info.ident.as_ref().expect("named field must have ident"),
                    );
                    let ty = &info.ty;
                    quote::quote! { #variant(<#ty as ::xdevs::port::AsPort>::Item) }
                })
                .collect();

            let match_arms: Vec<TokenStream2> = fields
                .named
                .iter()
                .map(|info| {
                    let variant = to_pascal_case_ident(
                        info.ident.as_ref().expect("named field must have ident"),
                    );

                    match expand_input_match_arm(info, &variant) {
                        Ok(arm_body) => quote::quote! {
                            Self::Mux::#variant(value) => #arm_body
                        },
                        Err(err) => {
                            combine_err(&mut acc, err);
                            // Emit a dummy arm to satisfy exact behavior and match exhaustiveness
                            quote::quote! {
                                Self::Mux::#variant(_) => Ok(())
                            }
                        }
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

                    match expand_output_for(info, &variant) {
                        Ok(for_body) => quote::quote! {
                            #for_body
                        },
                        Err(err) => {
                            combine_err(&mut acc, err);
                            // Output nothing for the failed field, errors handle it later
                            quote::quote! {}
                        }
                    }
                })
                .collect();

            if let Some(err) = acc {
                return Err(err);
            }

            let inject_event_body = quote::quote! {
                fn inject_event(&mut self, event: Self::Mux) -> Result<(), Self::Mux> {
                    match event {
                        #(#match_arms),*
                    }
                }
            };

            let eject_events_body = quote::quote! {
                fn eject_events(&self, mut ejector: impl FnMut(Self::Mux)) {
                    #(#propagations)*
                }
            };

            Ok(quote::quote! {
                unsafe impl #impl_generics ::xdevs::port::BagMux for #ident #ty_generics #where_clause {
                    type Mux = #private_mod_ident::PortMux #ty_generics;
                    #inject_event_body
                    #eject_events_body
                }

                mod #private_mod_ident {
                    use super::*;
                    /// Auto-generated enum for top-level channel communication.
                    #[derive(Clone)]
                    pub enum PortMux #impl_generics #where_clause {
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
fn expand_input_match_arm(info: &Field, variant: &Ident) -> Result<TokenStream2> {
    fn input_match_arm_body(
        ty: &Type,
        variant: &Ident,
        comes_from_array: bool,
    ) -> Result<TokenStream2> {
        match ty {
            Type::Path(_) => {
                let mut token = quote::quote! {
                    let result = port.add_value(value);
                };
                token.extend(if comes_from_array {
                    quote::quote! {
                        if let Err(value) = result {
                            Err(Self::Mux::#variant((index, value)))

                        } else {
                            Ok(())
                        }
                    }
                } else {
                    quote::quote! {
                        if let Err(value) = result {
                            Err(Self::Mux::#variant(value))
                        } else {
                            Ok(())
                        }
                    }
                });
                Ok(token)
            }
            Type::Array(array) => {
                let elem_ty = &array.elem;
                let body = input_match_arm_body(elem_ty, variant, true)?;
                Ok(quote::quote! {
                    let (index, value) = value;
                    if let Some(port) = port.get_mut(index)
                    {
                        #body
                    }
                    else
                    {
                        Ok(()) // Ignore out-of-bounds index, as it cannot be added to any port
                    }
                })
            }
            _ => Err(Error::new_spanned(
                ty,
                "unsupported input port type; expected array or Port",
            )),
        }
    }
    let field = &info.ident;
    let ty = &info.ty;
    let body = input_match_arm_body(ty, variant, false)?;
    Ok(quote::quote! {
        {
            let port = &mut self.#field;
            {
                #body
            }
        }
    })
}

/// Generate a for loop for the output enum to publish values from the corresponding output port.
fn expand_output_for(info: &Field, variant: &Ident) -> Result<TokenStream2> {
    fn output_for_body(ty: &Type, variant: &Ident, from_array: bool) -> Result<TokenStream2> {
        match ty {
            Type::Path(_) => {
                if from_array {
                    Ok(quote::quote! {
                        for value in port.get_values() {
                            ejector(Self::Mux::#variant((index, value.clone())));
                        }
                    })
                } else {
                    Ok(quote::quote! {
                        for value in port.get_values() {
                            ejector(Self::Mux::#variant(value.clone()));
                        }
                    })
                }
            }
            Type::Array(array) => {
                let body = output_for_body(&array.elem, variant, true)?;
                Ok(quote::quote! {
                    for (index, port) in port.iter().enumerate() {
                        #body
                    }
                })
            }
            _ => Err(Error::new_spanned(
                ty,
                "unsupported output port type; expected array or Port",
            )),
        }
    }
    let field = &info.ident;
    let ty = &info.ty;
    let body = output_for_body(ty, variant, false)?;
    Ok(quote::quote! {
        let port = &self.#field;
        {
            #body
        }
    })
}
