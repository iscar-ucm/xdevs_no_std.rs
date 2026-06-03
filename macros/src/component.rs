pub mod atomic;
mod backend;
pub mod coupled;
mod port;
mod rt_engine;
mod state;

use crate::{
    combine_err,
    component::{backend::Backend, coupled::components::Components, state::State},
};

use self::port::Ports;
use backend::RtEngine;
use proc_macro2::Span;
use proc_macro2::TokenStream as TokenStream2;
use std::collections::HashSet;
use syn::{
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    visit::{self, Visit},
    Attribute, Error, ExprPath, Field, Fields, GenericParam, Generics, Ident, ItemStruct, Lifetime,
    Meta, Result, Token, Type, TypeGenerics, TypePath, Visibility,
};

/// Different types of fields supported by `#[atomic]` and `#[coupled]` components.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FieldKind {
    Input,
    Output,
    State,
    Components,
}

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

/// Named struct field extracted from a component declaration.
pub struct ComponentField {
    pub ident: Ident,
    pub _vis: Visibility,
    pub ty: Type,
    pub attr: Attribute,
}

impl ComponentField {
    pub fn from_field(field: &Field, prev_attr: &mut Option<Attribute>) -> Result<Self> {
        let mut acc: Option<Error> = None;
        let attrs = &field.attrs;

        let ident = match &field.ident {
            Some(id) => id.clone(),
            None => {
                let err = Error::new_spanned(field, "Only named struct fields are supported");
                combine_err(&mut acc, err);
                syn::parse_quote!(_) // Placeholder to allow continued parsing
            }
        };

        if attrs.len() > 1 {
            combine_err(
                &mut acc,
                Error::new_spanned(&attrs[1], "Only one attribute is allowed on this field"),
            );
        }

        let attr: Attribute = match attrs.is_empty() {
            true => {
                if let Some(attr) = prev_attr {
                    attr.clone()
                } else {
                    combine_err(&mut acc, Error::new_spanned(
                        field,
                        "Field requires an attribute (#[input], #[output], #[state], or #[components])",
                    ));
                    syn::parse_quote!(#[invalid]) // Placeholder to allow continued parsing
                }
            }
            false => {
                let attr = attrs.first().unwrap().clone();
                *prev_attr = Some(attr.clone());
                attr
            }
        };

        if let Some(err) = acc {
            return Err(err);
        }
        Ok(Self {
            ident,
            _vis: field.vis.clone(),
            ty: field.ty.clone(),
            attr,
        })
    }

    pub fn kind(&self) -> Option<FieldKind> {
        if self.attr.path().is_ident("input") {
            Some(FieldKind::Input)
        } else if self.attr.path().is_ident("output") {
            Some(FieldKind::Output)
        } else if self.attr.path().is_ident("state") {
            Some(FieldKind::State)
        } else if self.attr.path().is_ident("components") {
            Some(FieldKind::Components)
        } else {
            None
        }
    }
}

/// Shared metadata used by atomic and coupled component macro expansions.
pub struct Component {
    pub ident: Ident,
    pub _vis: Visibility,
    pub generics: Generics,
    pub input: Ports,
    pub output: Ports,
    pub state: State,
    pub components: Components,
    pub rt_engine: Option<RtEngine>,
}

impl Component {
    pub fn new(args: &ComponentArgs, item: &ItemStruct) -> Result<Self> {
        let mut acc: Option<Error> = None;
        let mut input: Vec<ComponentField> = Vec::new();
        let mut output: Vec<ComponentField> = Vec::new();
        let mut state: Vec<ComponentField> = Vec::new();
        let mut components: Vec<ComponentField> = Vec::new();

        match &item.fields {
            Fields::Named(fields) => {
                let mut prev_attr: Option<Attribute> = None;
                for field in &fields.named {
                    match ComponentField::from_field(field, &mut prev_attr) {
                        Ok(field) => match field.kind() {
                            Some(FieldKind::Input) => input.push(field),
                            Some(FieldKind::Output) => output.push(field),
                            Some(FieldKind::State) => state.push(field),
                            Some(FieldKind::Components) => components.push(field),
                            None => {
                                combine_err(
                                    &mut acc,
                                    Error::new_spanned(
                                        &field.attr,
                                        "Invalid attribute for this field",
                                    ),
                                );
                            }
                        },
                        Err(err) => {
                            combine_err(&mut acc, err);
                        }
                    }
                }
            }
            _ => {
                return Err(Error::new_spanned(
                    item,
                    "Only named struct fields are supported",
                ))
            }
        }

        let ident = &item.ident;
        let generics = &item.generics;

        let input_generics = filter_generics(&input, &generics);
        let output_generics = filter_generics(&output, &generics);
        let state_generics = filter_generics(&state, &generics);
        let components_generics = filter_generics(&components, &generics);

        let input_ident = Ident::new(&format!("{ident}Input"), ident.span());
        let output_ident = Ident::new(&format!("{ident}Output"), ident.span());
        let state_ident = Ident::new(&format!("{ident}State"), ident.span());
        let components_ident = Ident::new(&format!("{ident}Components"), ident.span());

        let input = Ports::new(input, input_ident, input_generics);
        let output = Ports::new(output, output_ident, output_generics);
        let state = State::new(state, state_ident, state_generics);
        let components = Components::new(components, components_ident, components_generics);

        // Rt-engine argument
        let rt_engine = args.rt_engine.clone();
        if let Some(rt_engine) = &rt_engine {
            // Check compatibility of the component with the selected rt-engine backend.
            if let Err(err) = rt_engine.check_compatibility(args, &input, &output) {
                combine_err(&mut acc, err);
            }
        }

        if let Some(err) = acc {
            return Err(err);
        }

        Ok(Self {
            ident: ident.clone(),
            _vis: item.vis.clone(),
            generics: generics.clone(),
            input,
            output,
            state,
            components,
            rt_engine,
        })
    }
}

/// Internal visitor used to collect generic parameters referenced by fields.
struct GenericsCollector<'a> {
    declared_types: &'a HashSet<String>,
    declared_consts: &'a HashSet<String>,
    declared_lifetimes: &'a HashSet<String>,

    used_types: HashSet<String>,
    used_consts: HashSet<String>,
    used_lifetimes: HashSet<String>,
}

impl<'a> GenericsCollector<'a> {
    fn new(
        declared_types: &'a HashSet<String>,
        declared_consts: &'a HashSet<String>,
        declared_lifetimes: &'a HashSet<String>,
    ) -> Self {
        Self {
            declared_types,
            declared_consts,
            declared_lifetimes,
            used_types: HashSet::new(),
            used_consts: HashSet::new(),
            used_lifetimes: HashSet::new(),
        }
    }

    fn maybe_type(&mut self, ident: &Ident) {
        let name = ident.to_string();
        if self.declared_types.contains(&name) {
            self.used_types.insert(name);
        }
    }

    fn maybe_const(&mut self, ident: &Ident) {
        let name = ident.to_string();
        if self.declared_consts.contains(&name) {
            self.used_consts.insert(name);
        }
    }

    fn maybe_lifetime(&mut self, lifetime: &Lifetime) {
        let name = lifetime.ident.to_string();
        if self.declared_lifetimes.contains(&name) {
            self.used_lifetimes.insert(name);
        }
    }
}

impl<'ast, 'g> Visit<'ast> for GenericsCollector<'g> {
    fn visit_type_path(&mut self, node: &'ast TypePath) {
        if node.qself.is_none() {
            if let Some(ident) = node.path.get_ident() {
                self.maybe_type(ident);
            } else if let Some(first) = node.path.segments.first() {
                self.maybe_type(&first.ident);
            }
        }

        visit::visit_type_path(self, node);
    }

    fn visit_expr_path(&mut self, node: &'ast ExprPath) {
        if node.qself.is_none() {
            if let Some(ident) = node.path.get_ident() {
                self.maybe_const(ident);
            }
        }

        visit::visit_expr_path(self, node);
    }

    fn visit_lifetime(&mut self, lifetime: &'ast Lifetime) {
        self.maybe_lifetime(lifetime);
    }
}

pub fn filter_generics(fields: &[ComponentField], all: &Generics) -> Generics {
    let declared_types = all
        .type_params()
        .map(|tp| tp.ident.to_string())
        .collect::<HashSet<_>>();

    let declared_consts = all
        .const_params()
        .map(|cp| cp.ident.to_string())
        .collect::<HashSet<_>>();

    let declared_lifetimes = all
        .lifetimes()
        .map(|lp| lp.lifetime.ident.to_string())
        .collect::<HashSet<_>>();

    let mut collector =
        GenericsCollector::new(&declared_types, &declared_consts, &declared_lifetimes);

    for field in fields {
        collector.visit_type(&field.ty);
    }

    // Filter params
    let mut new_generics = all.clone();

    new_generics.params = all
        .params
        .iter()
        .filter(|param| match param {
            GenericParam::Type(tp) => collector.used_types.contains(&tp.ident.to_string()),
            GenericParam::Lifetime(lp) => collector
                .used_lifetimes
                .contains(&lp.lifetime.ident.to_string()),
            GenericParam::Const(cp) => collector.used_consts.contains(&cp.ident.to_string()),
        })
        .cloned()
        .collect();

    if new_generics.params.is_empty() {
        new_generics.lt_token = None;
        new_generics.gt_token = None;
    }

    new_generics
}

fn impl_component(
    ident: &Ident,
    input_ident: &Ident,
    output_ident: &Ident,
    generics: &Generics,
    input_generics: &TypeGenerics,
    output_generics: &TypeGenerics,
) -> TokenStream2 {
    let (impl_generics, ty_generics, _) = generics.split_for_impl();
    quote::quote! {
        unsafe impl #impl_generics xdevs::traits::Component for #ident #ty_generics{
            type Input = #input_ident #input_generics;
            type Output = #output_ident #output_generics;
            #[inline]
            fn get_t_last(&self) -> f64 {
                self.t_last
            }
            #[inline]
            fn set_t_last(&mut self, t_last: f64) {
                self.t_last = t_last;
            }
            #[inline]
            fn get_t_next(&self) -> f64 {
                self.t_next
            }
            #[inline]
            fn set_t_next(&mut self, t_next: f64) {
                self.t_next = t_next;
            }
        }
    }
}
