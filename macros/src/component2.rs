pub mod atomic;
mod backend;
pub mod coupled;
pub mod coupled2;
mod port;
mod rt_engine;
mod state;

use proc_macro2::TokenStream as TokenStream2;
use std::collections::HashSet;
use syn::visit::{self, Visit};
use syn::{Error, GenericParam, Generics, Ident, Lifetime, TypeGenerics};

use self::port::Ports;
use self::rt_engine::RtEngine;

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

/// Check for duplicate field names across inputs, outputs, and a third field list (state or components).
pub fn check_duplicate_fields(
    inputs: &[ComponentField],
    outputs: &[ComponentField],
    third: &[ComponentField],
) -> syn::Result<()> {
    let output_names: HashSet<String> = outputs.iter().map(|f| f.ident.to_string()).collect();
    let third_names: HashSet<String> = third.iter().map(|f| f.ident.to_string()).collect();

    for input in inputs {
        let name = input.ident.to_string();
        if output_names.contains(&name) {
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
