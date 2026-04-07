pub mod atomic;
mod backend;
pub mod coupled;
pub mod coupled2;
mod port;
mod rt_engine;
mod state;

use self::port::Ports;
use backend::RtEngine;
use proc_macro2::TokenStream as TokenStream2;
use std::collections::HashSet;
use syn::visit::{self, Visit};
use syn::{
    braced, parse::ParseStream, Error, GenericParam, Generics, Ident, ItemStruct, Lifetime, Token,
    TypeGenerics,
};

pub struct ComponentField {
    ident: syn::Ident,
    ty: syn::Type,
}

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
    ) -> syn::Result<Option<RtEngine>> {
        let mut rt_engine = None;
        syn::parse::Parser::parse2(
            |input: ParseStream| -> syn::Result<()> {
                while !input.is_empty() {
                    let ident: Ident = input.parse()?;
                    if ident == "rt_engine" {
                        // Accept both `rt_engine` and `rt_engine = { ... }`.
                        if input.peek(Token![=]) {
                            input.parse::<Token![=]>()?;
                            let content;
                            braced!(content in input);
                            rt_engine = Some(content.parse::<RtEngine>()?);
                        } else {
                            rt_engine = Some(RtEngine::default());
                        }
                    } else {
                        return Err(Error::new(ident.span(), unknown_arg_error));
                    }
                    if !input.is_empty() {
                        input.parse::<Token![,]>()?;
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
    ) -> syn::Result<Self> {
        let rt_engine = Self::parse_rt_engine(args, unknown_arg_error)?;
        let input_generics = filter_generics(&inputs, &generics);
        let output_generics = filter_generics(&outputs, &generics);

        let input_ident = syn::Ident::new(&format!("{ident}Input"), ident.span());
        let output_ident = syn::Ident::new(&format!("{ident}Output"), ident.span());

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
pub struct ParsedComponentFields {
    pub inputs: Vec<ComponentField>,
    pub outputs: Vec<ComponentField>,
    pub state: Vec<ComponentField>,
    pub components: Vec<ComponentField>,
}

impl ParsedComponentFields {
    /// Check for duplicate field names across inputs, outputs, and a third field list (state or components).
    fn check_duplicate_fields(
        inputs: &[ComponentField],
        outputs: &[ComponentField],
        third: &[ComponentField],
    ) -> syn::Result<()> {
        let output_names: HashSet<String> = outputs.iter().map(|f| f.ident.to_string()).collect();
        let third_names: HashSet<String> = third.iter().map(|f| f.ident.to_string()).collect();

        for input in inputs {
            let name = input.ident.to_string();
            if output_names.contains(&name) || third_names.contains(&name) {
                return Err(Error::new_spanned(
                    &input.ident,
                    format!("Duplicate field name '{}': already defined as input", name),
                ));
            }
            if third_names.contains(&name) {
                return Err(Error::new_spanned(
                    &input.ident,
                    format!("Duplicate field name '{}': already defined as input", name),
                ));
            }
        }
        for output in outputs {
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
    /// `#[components]`, and `#[component]`.
    pub fn parse(component: &ItemStruct) -> syn::Result<Self> {
        fn to_component_field(field: &syn::Field) -> syn::Result<ComponentField> {
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
            }
            if let Some(attr) = last_attr {
                let parsed_field = to_component_field(field)?;
                if attr.path().is_ident("input") {
                    parsed.inputs.push(parsed_field);
                } else if attr.path().is_ident("output") {
                    parsed.outputs.push(parsed_field);
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
            Self::check_duplicate_fields(&parsed.inputs, &parsed.outputs, &parsed.state)?;
        }
        if !parsed.components.is_empty() {
            Self::check_duplicate_fields(&parsed.inputs, &parsed.outputs, &parsed.components)?;
        }

        Ok(parsed)
    }
}

struct GenericsCollector<'a> {
    type_idents: HashSet<&'a Ident>,
    lifetimes: HashSet<&'a Lifetime>,
}

impl<'a, 'ast: 'a> Visit<'ast> for GenericsCollector<'a> {
    fn visit_type_path(&mut self, node: &'ast syn::TypePath) {
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
    ident: &syn::Ident,
    input_ident: &syn::Ident,
    output_ident: &syn::Ident,
    generics: &Generics,
    input_generics: &TypeGenerics,
    output_generics: &TypeGenerics,
) -> TokenStream2 {
    let (impl_generics, ty_generics, _) = generics.split_for_impl();
    quote::quote! {
        unsafe impl #impl_generics xdevs::traits::Component for #ident #ty_generics{
            type Input = #input_ident #input_generics;
            type Output = #output_ident #output_generics;
            type InputRef<'__xdevs_ports> = &'__xdevs_ports mut #input_ident #input_generics where Self: '__xdevs_ports;
            type OutputRef<'__xdevs_ports> = &'__xdevs_ports #output_ident #output_generics where Self: '__xdevs_ports;
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
            #[inline]
            fn get_input(&self) -> &Self::Input {
                &self.input
            }
            #[inline]
            fn get_input_mut(&mut self) -> &mut Self::Input {
                &mut self.input
            }
            #[inline]
            fn get_output(&self) -> &Self::Output {
                &self.output
            }
            #[inline]
            fn get_output_mut(&mut self) -> &mut Self::Output {
                &mut self.output
            }
            #[inline]
            fn get_ports(&mut self) -> (Self::InputRef<'_>, Self::OutputRef<'_>) {
                (&mut self.input, &self.output)
            }
            #[inline]
            fn get_out_ports(&self) -> Self::OutputRef<'_> {
                &self.output
            }
        }
    }
}
