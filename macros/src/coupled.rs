mod backend;
mod rt_engine;

use crate::{
    combine_err,
    coupled::{backend::RtEngine, rt_engine::expand_rt_engine},
};
use proc_macro2::{Span, TokenStream as TokenStream2};
use syn::{
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    Error, FieldsNamed, Ident, ItemStruct, Meta, Result, Token,
};

/// Arguments for both the `#[atomic]` and `#[coupled]` attribute macros.
#[derive(Debug)]
pub struct ComponentArgs {
    pub rt_engine: Option<RtEngine>,
    pub rt_engine_span: Option<Span>,
}

impl Parse for ComponentArgs {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut acc: Option<Error> = None;
        let mut args = ComponentArgs {
            rt_engine: None,
            rt_engine_span: None,
        };
        let mut rt_engine_seen = false;

        // Parse a comma-separated list of meta items (args)
        let metas = Punctuated::<Meta, Token![,]>::parse_terminated(input)?;

        for meta in metas {
            // Check if the argument matches what we are looking for
            if meta.path().is_ident("rt_engine") {
                if rt_engine_seen {
                    combine_err(
                        &mut acc,
                        Error::new_spanned(&meta, "duplicate argument: rt_engine"),
                    );
                } else {
                    rt_engine_seen = true;
                    args.rt_engine_span = Some(syn::spanned::Spanned::span(&meta));
                    match meta {
                        // Handles the case with no parenthesis: `#[component(rt_engine)]`
                        Meta::Path(_) => {
                            args.rt_engine = Some(RtEngine::default());
                        }
                        // Handles the parenthesized case: `#[component(rt_engine(...))]`
                        Meta::List(list) => match syn::parse2(list.tokens) {
                            Ok(rt_engine) => args.rt_engine = Some(rt_engine),
                            Err(err) => combine_err(&mut acc, err),
                        },
                        // Reject unsupported format `#[component(rt_engine = value)]`
                        Meta::NameValue(nv) => {
                            combine_err(
                                &mut acc,
                                Error::new_spanned(
                                    nv,
                                    "expected `rt_engine` or `rt_engine(...)`, found `rt_engine = ...`",
                                ),
                            );
                        }
                    }
                }
            } else {
                combine_err(
                    &mut acc,
                    Error::new_spanned(meta, "Unknown component argument"),
                );
            }
        }

        if let Some(err) = acc {
            return Err(err);
        }

        Ok(args)
    }
}

pub fn expand(args: ComponentArgs, mut item: ItemStruct) -> Result<TokenStream2> {
    let mut acc: Option<Error> = None;

    //TODO Remove this rt_engine generation and create its own #[derive] macro for it
    let rt_engine_impl = match expand_rt_engine(&args, &item) {
        Ok(tokens) => tokens,
        Err(err) => {
            combine_err(&mut acc, err);
            TokenStream2::new()
        }
    };

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

    // Generate the components struct
    let components_struct = {
        let mut item = item.clone();
        item.ident = Ident::new(&format!("{}Components", item.ident), item.ident.span());
        for field in item.fields.iter_mut() {
            let original_ty = field.ty.clone();
            field.ty = syn::parse_quote! {
                ::xdevs::simulator::Simulator<#original_ty>
            };
        }
        item
    };

    // Generate the components input wrapper struct
    let components_input_struct = {
        let mut item = item.clone();
        item.attrs = Vec::new();
        item.ident = Ident::new(&format!("{}ComponentsInput", item.ident), item.ident.span());
        for field in item.fields.iter_mut() {
            let original_ty = field.ty.clone();
            field.ty = syn::parse_quote! {
                <#original_ty as ::xdevs::traits::Component>::Input
            };
        }
        item
    };

    // Generate the components output wrapper struct
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
                <#original_ty as ::xdevs::traits::Component>::Output
            };
        }
        item
    };

    // Modify the original struct fields
    let components_ident = &components_struct.ident;
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
        #rt_engine_impl

        /// Struct holding all inner components as fields.
        #components_struct

        /// Wrapper struct holding mutable references to all inner components' inputs.
        #[derive(::xdevs::Bag)]
        #components_input_struct

        /// Wrapper struct holding references to all inner components' outputs.
        #[derive(::xdevs::Bag)]
        #components_output_struct

        /// Original model struct with added fields for inner components, their inputs and outputs, and simulation time.
        #item

        impl #impl_generics #item_ident #ty_generics #where_clause {
            #[inline]
            pub fn build(#(#item_fields: #item_tys),*) -> Self {
                Self {
                    components_input: <#components_input_ident #ty_generics as ::xdevs::traits::Bag>::build(),
                    components_output: <#components_output_ident #ty_generics as ::xdevs::traits::Bag>::build(),
                    components: #components_ident #ty_generics_turbofish {#(#item_fields: ::xdevs::simulator::Simulator::new(#item_fields)),*},
                }
            }
        }

        unsafe impl #impl_generics ::xdevs::traits::PartialCoupled for #item_ident #ty_generics #where_clause {
            type ComponentsInput = #components_input_ident #ty_generics;
            type ComponentsOutput = #components_output_ident #ty_generics;
        }
        unsafe impl #impl_generics ::xdevs::traits::AbstractSimulator for #item_ident #ty_generics #where_clause {
            #[inline]
            fn start(simulator: &mut ::xdevs::simulator::Simulator<Self>, t_start: f64) -> f64 {
                // set t_last to t_start
                simulator.set_t_last(t_start);
                // get minimum t_next from all components
                let mut t_next = f64::INFINITY;
                #(t_next = f64::min(t_next, ::xdevs::traits::AbstractSimulator::start(&mut simulator.components.#item_fields, t_start));)*
                // set t_next to minimum t_next
                simulator.set_t_next(t_next);

                t_next
            }

            #[inline]
            fn stop(simulator: &mut ::xdevs::simulator::Simulator<Self>, t_stop: f64) {
                // stop all components
                #(::xdevs::traits::AbstractSimulator::stop(&mut simulator.components.#item_fields, t_stop);)*
                // set t_last to t_stop and t_next to infinity
                simulator.set_t_last(t_stop);
                simulator.set_t_next(f64::INFINITY);
            }

            #[inline]
            fn lambda(simulator: &mut ::xdevs::simulator::Simulator<Self>, output: &mut Self::Output, t: f64) {
                if t >= simulator.get_t_next() {
                    let model = &mut **simulator;
                    // propagate lambda to all components
                    #(::xdevs::traits::AbstractSimulator::lambda(&mut model.components.#item_fields, &mut model.components_output.#item_fields, t);)*
                    // propagate EOCs via Coupled trait
                    <Self as ::xdevs::Coupled>::eoc(&simulator.components_output, output);
                }
            }

            #[inline]
            fn delta(simulator: &mut ::xdevs::simulator::Simulator<Self>, input: &mut Self::Input, output: &mut Self::Output, t: f64) -> f64 {
                let model = &mut **simulator;

                // propagate EICs and ICs via Coupled trait
                <Self as ::xdevs::Coupled>::eic(input, &mut model.components_input);
                <Self as ::xdevs::Coupled>::ic(&model.components_output, &mut model.components_input);

                // get minimum t_next from all components after executing their delta
                let mut t_next = f64::INFINITY;
                #(t_next = f64::min(t_next, ::xdevs::traits::AbstractSimulator::delta(
                    &mut model.components.#item_fields,
                    &mut model.components_input.#item_fields,
                    &mut model.components_output.#item_fields,
                     t));)*

                // set t_last to t and t_next to minimum t_next
                simulator.set_t_last(t);
                simulator.set_t_next(t_next);

                // clear input and output events
                <Self::Input as ::xdevs::traits::Bag>::clear(input);
                <Self::Output as ::xdevs::traits::Bag>::clear(output);

                t_next
            }
        }
    };
    Ok(expanded.into())
}
