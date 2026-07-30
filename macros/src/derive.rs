use heck::ToSnakeCase;
use heck::ToUpperCamelCase;
use proc_macro2::TokenStream as TokenStream2;
use syn::{Data, DeriveInput, Error, Fields, Ident, Index, Result};

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

pub fn derive_asport(input: DeriveInput) -> Result<TokenStream2> {
    let ident = input.ident;
    let snake_case_ident = Ident::new(&ident.to_string().to_snake_case(), ident.span());
    let private_mod_ident = Ident::new(
        &format!("_xdevs_no_std_{}_asport", snake_case_ident),
        ident.span(),
    );
    let generics = input.generics;

    let fields = match input.data {
        Data::Struct(data) => data.fields,
        _ => {
            return Err(Error::new_spanned(
                ident,
                "AsPort can only be derived for structs",
            ))
        }
    };

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    match fields {
        Fields::Unnamed(_) => Err(Error::new_spanned(
            ident,
            "AsPort cannot be derived for tuple structs",
        )),
        Fields::Unit => Ok(quote::quote! {
            unsafe impl #impl_generics ::xdevs::port::AsPort for #ident #ty_generics #where_clause {
                type Value = ();
                fn inject_event(&mut self, _event: Self::Value) -> ::core::result::Result<(), Self::Value> {
                    Ok(())
                }
                fn eject_events(&self, _ejector: impl FnMut(Self::Value)) {}
            }
        }),

        Fields::Named(fields) => {
            let variants: Vec<TokenStream2> = fields
                .named
                .iter()
                .map(|info| {
                    let variant = to_pascal_case_ident(
                        info.ident.as_ref().expect("named field must have ident"),
                    );
                    let ty = &info.ty;
                    quote::quote! { #variant(<#ty as ::xdevs::port::AsPort>::Value) }
                })
                .collect();

            let match_arms: Vec<TokenStream2> = fields
                .named
                .iter()
                .map(|info| {
                    let variant = to_pascal_case_ident(
                        info.ident.as_ref().expect("named field must have ident"),
                    );
                    let field = info.ident.as_ref().expect("named field must have ident");
                    quote::quote! {
                        Self::Value::#variant(value) => self.#field.inject_event(value).map_err(Self::Value::#variant)
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
                    let field = info.ident.as_ref().expect("named field must have ident");
                    quote::quote! {
                        self.#field.eject_events(|v| ejector(Self::Value::#variant(v)));
                    }
                })
                .collect();

            Ok(quote::quote! {
                unsafe impl #impl_generics ::xdevs::port::AsPort for #ident #ty_generics #where_clause {
                    type Value = #private_mod_ident::PortMux #ty_generics;

                    fn inject_event(&mut self, event: Self::Value) -> ::core::result::Result<(), Self::Value> {
                        match event {
                            #(#match_arms),*
                        }
                    }

                    fn eject_events(&self, mut ejector: impl FnMut(Self::Value)) {
                        #(#propagations)*
                    }
                }

                mod #private_mod_ident {
                    use super::*;

                    #[derive(Clone)]
                    pub enum PortMux #impl_generics #where_clause {
                        #(#variants),*
                    }
                }
            })
        }
    }
}

fn to_pascal_case_ident(ident: &Ident) -> Ident {
    Ident::new(&ident.to_string().to_upper_camel_case(), ident.span())
}
