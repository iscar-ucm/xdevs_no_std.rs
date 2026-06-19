use crate::combine_err;
use proc_macro2::TokenStream as TokenStream2;
use syn::{Error, FieldsNamed, Ident, ItemStruct, Result};

pub fn expand(mut item: ItemStruct) -> Result<TokenStream2> {
    let mut acc: Option<Error> = None;

    // Generate initial variables for code generation
    let (impl_generics, ty_generics, where_clause) = item.generics.split_for_impl();
    let ty_generics_turbofish = ty_generics.as_turbofish();
    let item_ident = &item.ident;

    let mut item_fields = Vec::new();
    let mut item_tys = Vec::new();

    match &item.fields {
        syn::Fields::Named(fields) => {
            for field in &fields.named {
                let Some(field_ident) = &field.ident else {
                    combine_err(&mut acc, Error::new_spanned(field, "expected named field"));
                    continue;
                };

                item_fields.push(field_ident.clone());
                item_tys.push(field.ty.clone());
            }
        }
        _ => {
            combine_err(
                &mut acc,
                Error::new_spanned(
                    &item.fields,
                    "only named fields are supported for coupled components",
                ),
            );
        }
    }

    if let Some(err) = acc {
        return Err(err);
    }

    // Generate the components struct (with recursive unwrapping)
    let components_struct = {
        let mut item = item.clone();
        item.ident = Ident::new(&format!("{}Components", item.ident), item.ident.span());
        for field in item.fields.iter_mut() {
            let ty = &field.ty;
            field.ty = syn::parse_quote! {
                <#ty as ::xdevs::simulation::ErasedSimulable>::Simulator
            };
        }
        item
    };

    // Generate the components input wrapper struct (wrapping the outermost type)
    let components_input_struct = {
        let mut item = item.clone();
        item.attrs = Vec::new();
        item.ident = Ident::new(&format!("{}ComponentsInput", item.ident), item.ident.span());
        for field in item.fields.iter_mut() {
            let original_ty = field.ty.clone();
            field.ty = syn::parse_quote! {
                <#original_ty as ::xdevs::Component>::Input
            };
        }
        item
    };

    // Generate the components output wrapper struct (wrapping the outermost type)
    let components_output_struct = {
        let mut item = item.clone();
        item.attrs = Vec::new();
        item.ident = Ident::new(
            &format!("{}ComponentsOutput", item.ident),
            item.ident.span(),
        );
        for field in item.fields.iter_mut() {
            let original_ty = field.ty.clone();
            field.ty = syn::parse_quote! {
                <#original_ty as ::xdevs::Component>::Output
            };
        }
        item
    };

    // Construct the initialization fields iteratively to handle arrays.
    let mut init_fields = Vec::new();
    for (ident, ty) in item_fields.iter().zip(item_tys.iter()) {
        let init_expr = quote::quote! {
            <#ty as ::xdevs::simulation::ErasedSimulable>::to_simulator(#ident)
        };
        init_fields.push(quote::quote! { #ident: #init_expr });
    }

    // Modify the original struct fields
    let components_ident = &components_struct.ident;
    let components_fields_idents: Vec<Ident> = components_struct
        .fields
        .iter()
        .filter_map(|field| field.ident.clone())
        .collect();
    let components_input_ident = &components_input_struct.ident;
    let components_output_ident = &components_output_struct.ident;

    let new_fields: FieldsNamed = syn::parse_quote! {
        {
            components: #components_ident #ty_generics,
            components_input: #components_input_ident #ty_generics,
            components_output: #components_output_ident #ty_generics,
        }
    };
    item.fields = syn::Fields::Named(new_fields);

    // Generate the expanded code
    let expanded = quote::quote! {
        /// Wrapper struct holding mutable references to all inner components' inputs.
        #[derive(::xdevs::Bag)]
        #components_input_struct

        /// Wrapper struct holding references to all inner components' outputs.
        #[derive(::xdevs::Bag)]
        #components_output_struct

        /// Struct holding all inner components as fields.
        #components_struct

        impl #impl_generics ::xdevs::Component for #components_ident #ty_generics #where_clause {
            type Kind = ::xdevs::ComponentsKind;
            type Input = #components_input_ident #ty_generics;
            type Output = #components_output_ident #ty_generics;
        }

        unsafe impl #impl_generics ::xdevs::simulation::AbstractSimulator for #components_ident #ty_generics #where_clause {
            type Input = <Self as ::xdevs::Component>::Input;
            type Output = <Self as ::xdevs::Component>::Output;

            #[inline(always)]
            fn start(&mut self, t_start: f64) -> f64 {
                let mut t_next = f64::INFINITY;
                #(t_next = f64::min(t_next, ::xdevs::simulation::AbstractSimulator::start(&mut self.#components_fields_idents, t_start));)*
                t_next
            }

            #[inline(always)]
            fn stop(&mut self) {
                #(::xdevs::simulation::AbstractSimulator::stop(&mut self.#components_fields_idents);)*
            }

            #[inline(always)]
            fn lambda(&mut self, output: &mut Self::Output, t: f64) {
                #(::xdevs::simulation::AbstractSimulator::lambda(&mut self.#components_fields_idents, &mut output.#components_fields_idents, t);)*
            }

            #[inline(always)]
            fn delta(&mut self, input: &mut Self::Input, output: &mut Self::Output, t: f64) -> f64 {
                let mut t_next = f64::INFINITY;
                #(t_next = f64::min(t_next, ::xdevs::simulation::AbstractSimulator::delta(
                        &mut self.#components_fields_idents,
                        &mut input.#components_fields_idents,
                        &mut output.#components_fields_idents,
                        t));)*
                t_next
            }
        }

        /// Original model struct with added fields for inner components, their inputs and outputs, and simulation time.
        #item

        impl #impl_generics #item_ident #ty_generics #where_clause {
            #[inline]
            pub fn build(#(#item_fields: #item_tys),*) -> Self {
                Self {
                    components_input: <#components_input_ident #ty_generics as ::xdevs::port::Bag>::build(),
                    components_output: <#components_output_ident #ty_generics as ::xdevs::port::Bag>::build(),
                    components: #components_ident #ty_generics_turbofish {
                        #(#init_fields),*
                    },
                }
            }
        }

        impl #impl_generics ::xdevs::component::coupled::PartialCoupled for #item_ident #ty_generics #where_clause {
            type Components = #components_ident #ty_generics;

            fn get_components(&self) -> &::xdevs::component::coupled::Processors<Self> {
                &self.components
            }

            fn get_components_mut(&mut self) -> &mut ::xdevs::component::coupled::Processors<Self> {
                &mut self.components
            }
        }
    };
    Ok(expanded.into())
}
