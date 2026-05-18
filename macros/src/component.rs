pub mod atomic;
mod backend;
pub mod coupled;
mod port;
mod rt_engine;
mod state;

use self::port::Ports;
use backend::RtEngine;
use proc_macro2::TokenStream as TokenStream2;
use std::collections::HashSet;
use syn::{
    parenthesized,
    parse::{ParseStream, Parser},
    token::Paren,
    visit::{self, Visit},
    Error, Field, GenericParam, Generics, Ident, ItemStruct, Lifetime, Result, Type, TypeGenerics,
    TypePath,
};

/// Named struct field extracted from a component declaration.
pub struct ComponentField {
    pub ident: Ident,
    pub ty: Type,
}

/// Shared metadata used by atomic and coupled component macro expansions.
pub struct CommonComponent {
    pub ident: Ident,
    pub generics: Generics,
    pub input: Ports,
    pub output: Ports,
    pub rt_engine: Option<RtEngine>,
}

impl CommonComponent {
    fn parse_rt_engine(
        args: TokenStream2,
        unknown_arg_error: &'static str,
    ) -> Result<Option<RtEngine>> {
        let mut rt_engine = None;
        Parser::parse2(
            |input: ParseStream| -> Result<()> {
                while !input.is_empty() {
                    let ident: Ident = input.parse()?;
                    if ident == "rt_engine" {
                        // Accept both `rt_engine` and `rt_engine(...)`.
                        if input.peek(Paren) {
                            let content;
                            parenthesized!(content in input);
                            rt_engine = Some(content.parse::<RtEngine>()?);
                        } else {
                            rt_engine = Some(RtEngine::default());
                        }
                    } else {
                        return Err(Error::new(ident.span(), unknown_arg_error));
                    }
                }
                Ok(())
            },
            args,
        )?;

        Ok(rt_engine)
    }

    pub fn new(
        ident: Ident,
        generics: Generics,
        inputs: Vec<ComponentField>,
        outputs: Vec<ComponentField>,
        args: TokenStream2,
        unknown_arg_error: &'static str,
    ) -> Result<Self> {
        let rt_engine = Self::parse_rt_engine(args, unknown_arg_error)?;
        let input_generics = filter_generics(&inputs, &generics);
        let output_generics = filter_generics(&outputs, &generics);

        let input_ident = Ident::new(&format!("{ident}Input"), ident.span());
        let output_ident = Ident::new(&format!("{ident}Output"), ident.span());

        Ok(Self {
            ident,
            generics,
            input: Ports::new(inputs, input_ident, input_generics),
            output: Ports::new(outputs, output_ident, output_generics),
            rt_engine,
        })
    }
}

#[derive(Default)]
/// Parsed field groups collected from a user component struct.
pub struct ParsedComponentFields {
    pub input: Vec<ComponentField>,
    pub output: Vec<ComponentField>,
    pub state: Vec<ComponentField>,
    pub components: Vec<ComponentField>,
}

impl ParsedComponentFields {
    /// Check for duplicate field names across inputs, outputs, and a third field list (state or components).
    fn check_duplicate_fields(
        input: &[ComponentField],
        output: &[ComponentField],
        third: &[ComponentField],
    ) -> Result<()> {
        let output_names: HashSet<String> = output.iter().map(|f| f.ident.to_string()).collect();
        let third_names: HashSet<String> = third.iter().map(|f| f.ident.to_string()).collect();

        for input in input {
            let name = input.ident.to_string();
            if output_names.contains(&name) || third_names.contains(&name) {
                return Err(Error::new_spanned(
                    &input.ident,
                    format!("Duplicate field name '{}': already defined as input", name),
                ));
            }
        }
        for output in output {
            let name = output.ident.to_string();
            if third_names.contains(&name) {
                return Err(Error::new_spanned(
                    &output.ident,
                    format!("Duplicate field name '{}': already defined as output", name),
                ));
            }
        }
        Ok(())
    }

    /// Parse and classify all recognized component fields by attribute.
    ///
    /// Supported attributes are `#[input]`, `#[output]`, `#[state]`,
    /// and `#[components]`.
    pub fn parse(component: &ItemStruct) -> Result<Self> {
        fn to_component_field(field: &Field) -> Result<ComponentField> {
            let ident = field.ident.clone().ok_or_else(|| {
                Error::new_spanned(field, "Only named struct fields are supported")
            })?;
            Ok(ComponentField {
                ident,
                ty: field.ty.clone(),
            })
        }

        let mut parsed = Self::default();
        let mut last_attr = None;

        for field in &component.fields {
            let field_attrs = &field.attrs;

            if field_attrs.len() > 1 {
                return Err(Error::new_spanned(
                    field,
                    "Each field may have at most one attribute",
                ));
            }

            if let Some(attr) = field_attrs.first() {
                last_attr = Some(attr);
            } else if last_attr.is_none() {
                return Err(Error::new_spanned(
                    field,
                    "Field requires an attribute (#[input], #[output], #[state], or #[components])",
                ));
            }
            if let Some(attr) = last_attr {
                let parsed_field = to_component_field(field)?;
                if attr.path().is_ident("input") {
                    parsed.input.push(parsed_field);
                } else if attr.path().is_ident("output") {
                    parsed.output.push(parsed_field);
                } else if attr.path().is_ident("state") {
                    parsed.state.push(parsed_field);
                } else if attr.path().is_ident("components") {
                    parsed.components.push(parsed_field);
                } else {
                    return Err(Error::new_spanned(attr, "Unknown attribute"));
                }
            }
        }

        // Validate duplicate names as part of common field parsing.
        // Only run checks for non-empty groups so each macro kind can keep
        // its own semantic constraints (e.g. state vs components) separately.
        if !parsed.state.is_empty() {
            Self::check_duplicate_fields(&parsed.input, &parsed.output, &parsed.state)?;
        }
        if !parsed.components.is_empty() {
            Self::check_duplicate_fields(&parsed.input, &parsed.output, &parsed.components)?;
        }

        Ok(parsed)
    }
}

/// Internal visitor used to collect generic parameters referenced by fields.
struct GenericsCollector<'a> {
    type_idents: HashSet<&'a Ident>,
    lifetimes: HashSet<&'a Lifetime>,
}

impl<'a, 'ast: 'a> Visit<'ast> for GenericsCollector<'a> {
    fn visit_type_path(&mut self, node: &'ast TypePath) {
        if let Some(ident) = node.path.get_ident() {
            self.type_idents.insert(ident);
        }
        visit::visit_type_path(self, node);
    }

    fn visit_lifetime(&mut self, lt: &'ast Lifetime) {
        self.lifetimes.insert(lt);
        visit::visit_lifetime(self, lt);
    }
}

pub fn filter_generics(fields: &[ComponentField], all: &Generics) -> Generics {
    let mut collector = GenericsCollector {
        type_idents: HashSet::new(),
        lifetimes: HashSet::new(),
    };
    for field in fields {
        collector.visit_type(&field.ty);
    }

    // Filter params
    let mut new_generics = all.clone();
    new_generics.params = all
        .params
        .iter()
        .filter_map(|param| match param {
            GenericParam::Type(tp) if collector.type_idents.contains(&tp.ident) => {
                Some(param.clone())
            }
            GenericParam::Lifetime(lp) if collector.lifetimes.contains(&lp.lifetime) => {
                Some(param.clone())
            }
            GenericParam::Const(cp) if collector.type_idents.contains(&cp.ident) => {
                Some(param.clone())
            }
            _ => None,
        })
        .collect();
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
