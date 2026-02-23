use proc_macro2::TokenStream as TokenStream2;
use syn::{
    parse::{Parse, ParseStream},
    Error, Generics, Ident, ItemStruct, LitInt, Token,
};

// Import Field and filter_generics from parent module to reuse parsing logic
use super::{filter_generics, Field};

/// Arguments for the `#[rt_engine]` attribute macro.
///
/// Supported arguments:
/// - `in_size`: capacity of the input channel
/// - `out_size`: capacity of the output channel
/// - `max_out_subs`: number of subscribers for the output PubSubChannel
struct TopArgs {
    in_size: usize,
    out_size: usize,
    max_out_subs: usize,
}

impl Parse for TopArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut in_size = None;
        let mut out_size = None;
        let mut max_out_subs = None;

        while !input.is_empty() {
            let ident: Ident = input.parse()?;
            input.parse::<Token![=]>()?;
            let value: LitInt = input.parse()?;
            let value: usize = value.base10_parse()?;

            match ident.to_string().as_str() {
                "in_size" => in_size = Some(value),
                "out_size" => out_size = Some(value),
                "max_out_subs" => max_out_subs = Some(value),
                _ => {
                    return Err(Error::new(
                        ident.span(),
                        "unknown top argument; expected `in_size`, `out_size`, or `max_out_subs`",
                    ))
                }
            }

            // Optional trailing comma
            if !input.is_empty() {
                input.parse::<Token![,]>()?;
            }
        }

        Ok(TopArgs {
            in_size: in_size
                .ok_or_else(|| Error::new(input.span(), "missing mandatory argument: in_size"))?,
            out_size: out_size
                .ok_or_else(|| Error::new(input.span(), "missing mandatory argument: out_size"))?,
            max_out_subs: max_out_subs.ok_or_else(|| {
                Error::new(input.span(), "missing mandatory argument: max_out_subs")
            })?,
        })
    }
}

/// Enum with the inner type possibilities.
enum PortInnerType {
    Single(syn::Type),
    Array(Box<PortInnerType>),
}

/// Information about a port field extracted from the struct.
struct PortInfo {
    field_name: Ident,
    inner_type: PortInnerType,
}

/// Parsed representation of an `#[rt_engine]` annotated struct.
///
/// Holds all data extracted during parsing, ready for code generation via `quote()`.
pub struct Component {
    ident: Ident,
    generics: Generics,
    input_ports: Vec<PortInfo>,
    output_ports: Vec<PortInfo>,
    args: TopArgs,
    item: TokenStream2,
}

impl Component {
    /// Parse the attribute arguments and the annotated struct item.
    ///
    /// Extracts input/output port fields and validates the struct structure.
    pub fn parse(args: TokenStream2, item: TokenStream2) -> syn::Result<Self> {
        let args: TopArgs = syn::parse2(args)?;
        let component: ItemStruct = syn::parse2(item.clone())?;

        let ident = component.ident.clone();
        let generics = component.generics.clone();

        // Parse fields to extract input and output ports
        let mut inputs = Vec::new();
        let mut outputs = Vec::new();
        let mut last_attr = None;

        for field in &component.fields {
            let field_attrs = &field.attrs;

            if field_attrs.len() > 1 {
                return Err(Error::new_spanned(
                    &field,
                    "Each field may have at most one attribute",
                ));
            }

            if let Some(attr) = field_attrs.first() {
                last_attr = Some(attr);
            }
            if let Some(attr) = last_attr {
                if attr.path().is_ident("input") {
                    let field_ident = field.ident.clone().unwrap();
                    let field_ty = field.ty.clone();
                    inputs.push(Field {
                        ident: field_ident,
                        ty: field_ty,
                    });
                } else if attr.path().is_ident("output") {
                    let field_ident = field.ident.clone().unwrap();
                    let field_ty = field.ty.clone();
                    outputs.push(Field {
                        ident: field_ident,
                        ty: field_ty,
                    });
                }
                // Ignore other attributes (state, components, etc.)
            }
        }

        // Extract port info from the Field objects
        let input_ports: Vec<PortInfo> = inputs
            .iter()
            .filter_map(|field| extract_port_info(field.ident.clone(), &field.ty))
            .collect();

        let output_ports: Vec<PortInfo> = outputs
            .iter()
            .filter_map(|field| extract_port_info(field.ident.clone(), &field.ty))
            .collect();

        Ok(Component {
            ident,
            generics,
            input_ports,
            output_ports,
            args,
            item,
        })
    }

    /// Generate the top-level infrastructure code:
    /// - Input/Output enums for channel communication
    /// - Static input `Channel` and output `PubSubChannel`
    /// - `unsafe impl RtEngine` with `map_input` and `map_output`
    ///
    /// The original item (with inner attributes like `#[atomic]`, `#[coupled]`, etc.)
    /// is passed through unchanged so those macros can process the struct next.
    pub fn quote(&self) -> TokenStream2 {
        let model_ident = &self.ident;

        // Model generics (for the impl block)
        let (model_impl_generics, model_ty_generics, model_where_clause) =
            self.generics.split_for_impl();

        // Compute generics for input/output enums based on port *inner* types only.
        // This means Port<&'a T, N> propagates 'a and T to the enum,
        // but &'a Port<T, N> or [&'a Port<T, N>; M] do NOT propagate 'a.
        let input_inner_fields: Vec<Field> = self
            .input_ports
            .iter()
            .map(|info| Field {
                ident: info.field_name.clone(),
                ty: inner_leaf_type(&info.inner_type).clone(),
            })
            .collect();
        let output_inner_fields: Vec<Field> = self
            .output_ports
            .iter()
            .map(|info| Field {
                ident: info.field_name.clone(),
                ty: inner_leaf_type(&info.inner_type).clone(),
            })
            .collect();
        let input_enum_generics = filter_generics(&input_inner_fields, &self.generics);
        let output_enum_generics = filter_generics(&output_inner_fields, &self.generics);
        let (input_enum_impl_generics, input_enum_ty_generics, input_enum_where_clause) =
            input_enum_generics.split_for_impl();
        let (output_enum_impl_generics, output_enum_ty_generics, output_enum_where_clause) =
            output_enum_generics.split_for_impl();

        // Generate identifiers for the infrastructure types
        let input_enum_ident = quote::format_ident!("{}InputEnum", model_ident);
        let output_enum_ident = quote::format_ident!("{}OutputEnum", model_ident);

        let upper_name = to_screaming_snake_case(&model_ident.to_string());
        let in_channel_ident = quote::format_ident!("{}_IN_CHANNEL", upper_name);
        let out_channel_ident = quote::format_ident!("{}_OUT_CHANNEL", upper_name);

        let in_size = self.args.in_size;
        let out_size = self.args.out_size;
        let max_out_subs = self.args.max_out_subs;

        // Hidden module items
        let snake_name = to_snake_case(&model_ident.to_string());
        let private_mod_ident =
            quote::format_ident!("__xdevs_no_std_private_{}_rt_engine", snake_name);
        let mut private = TokenStream2::new();

        // Build generated code starting with the original item (pass-through)
        let mut generated = self.item.clone();

        // === Input infrastructure (enum, static channel) ===
        let input_enum_type;
        let map_input_body;
        let sender_type;
        let input_channel_type;
        let input_channel_call;

        if !self.input_ports.is_empty() {
            let input_variants: Vec<TokenStream2> = self
                .input_ports
                .iter()
                .map(|info| {
                    let variant = to_pascal_case_ident(&info.field_name);
                    let ty = port_payload_type_tokens(&info.inner_type);
                    quote::quote! { #variant(#ty) }
                })
                .collect();

            let match_arms: Vec<TokenStream2> = self
                .input_ports
                .iter()
                .map(|info| {
                    let variant = to_pascal_case_ident(&info.field_name);
                    let field = &info.field_name;
                    match &info.inner_type {
                        PortInnerType::Single(_) => quote::quote! {
                            Self::InputEnum::#variant(value) => input.#field.add_value(value).unwrap()
                        },
                        PortInnerType::Array(_) => quote::quote! {
                            Self::InputEnum::#variant((index, value)) => {
                                if let Some(port) = input.#field.get_mut(index) {
                                    port.add_value(value).unwrap();
                                } else {
                                    panic!("input index out of bounds");
                                }
                            }
                        },
                    }
                })
                .collect();

            private.extend(quote::quote! {
                /// Auto-generated static input channel.
                pub static #in_channel_ident: ::xdevs::export::Channel<
                    ::xdevs::export::Mutex,
                    #input_enum_ident #input_enum_ty_generics,
                    #in_size
                > = ::xdevs::export::Channel::new();

                /// Auto-generated input enum for top-level channel communication.
                pub enum #input_enum_ident #input_enum_impl_generics #input_enum_where_clause {
                    #(#input_variants),*
                }
            });

            input_enum_type = quote::quote! { #private_mod_ident::#input_enum_ident #input_enum_ty_generics };
            map_input_body = quote::quote! {
                match ::xdevs::traits::RtEngineInputChannel::recv(in_channel).await {
                    #(#match_arms),*
                }
            };
            sender_type = quote::quote! { <Self::InputChannel as ::xdevs::traits::RtEngineInputChannel>::Sender };
            input_channel_type = quote::quote! { ::xdevs::export::InputChannel<'static,
                    #input_enum_type,
                    #in_size
                > };
            input_channel_call = quote::quote! {::xdevs::export::InputChannel::new(&#private_mod_ident::#in_channel_ident) };
        } else {
            input_enum_type = quote::quote! { () };
            map_input_body = quote::quote! {};
            sender_type = quote::quote! { () };
            input_channel_type = quote::quote! { () };
            input_channel_call = quote::quote! { () };
        }

        // === Output infrastructure (enum, static channel) ===
        let output_enum_type;
        let map_output_body;
        let subscriber_type;
        let output_channel_type;
        let output_channel_call;

        if !self.output_ports.is_empty() {
            let output_variants: Vec<TokenStream2> = self
                .output_ports
                .iter()
                .map(|info| {
                    let variant = to_pascal_case_ident(&info.field_name);
                    let ty = port_payload_type_tokens(&info.inner_type);
                    quote::quote! { #variant(#ty) }
                })
                .collect();

            let propagations: Vec<TokenStream2> = self
                .output_ports
                .iter()
                .map(|info| {
                    let variant = to_pascal_case_ident(&info.field_name);
                    let field = &info.field_name;
                    match &info.inner_type {
                        PortInnerType::Single(_) => quote::quote! {
                            if let Some(&value) = output.#field.get_values().last() {
                                ::xdevs::traits::RtEngineOutputChannel::publish(
                                    out_channel, 
                                    Self::OutputEnum::#variant(value));
                            }
                        },
                        PortInnerType::Array(_) => quote::quote! {
                            for (index, port) in output.#field.iter().enumerate() {
                                if let Some(&value) = port.get_values().last() {
                                    ::xdevs::traits::RtEngineOutputChannel::publish(
                                        out_channel, 
                                        Self::OutputEnum::#variant((index, value)));
                                }
                            }
                        },
                    }
                })
                .collect();

            private.extend(quote::quote! {
                /// Auto-generated static output PubSub channel.
                pub static #out_channel_ident: ::xdevs::export::PubSubChannel<
                    ::xdevs::export::Mutex,
                    #output_enum_ident #output_enum_ty_generics,
                    #out_size,
                    #max_out_subs,
                    1
                > = ::xdevs::export::PubSubChannel::new();

                /// Auto-generated output enum for top-level channel communication.
                #[derive(Clone, PartialEq)]
                pub enum #output_enum_ident #output_enum_impl_generics #output_enum_where_clause {
                    #(#output_variants),*
                }
            });

            output_enum_type = quote::quote! { #private_mod_ident::#output_enum_ident #output_enum_ty_generics };
            map_output_body = quote::quote! {
                #(#propagations)*
            };
            subscriber_type = quote::quote! { <Self::OutputChannel as ::xdevs::traits::RtEngineOutputChannel>::Subscriber };
            output_channel_type = quote::quote! { ::xdevs::export::OutputChannel<'static,
                    #output_enum_type,
                    #out_size,
                    #max_out_subs                
                > };
            output_channel_call = quote::quote! {::xdevs::export::OutputChannel::new(&#private_mod_ident::#out_channel_ident) };
        } else {
            output_enum_type = quote::quote! { () };
            map_output_body = quote::quote! {};
            subscriber_type = quote::quote! { () };
            output_channel_type = quote::quote! { () };
            output_channel_call = quote::quote! { () };
        }

        // === RtEngine trait implementation ===
        generated.extend(quote::quote! {
            /// Auto-generated `RtEngine` implementation for the top-level component.
            unsafe impl #model_impl_generics ::xdevs::traits::RtEngineWrapper for #model_ident #model_ty_generics #model_where_clause {
                type InputEnum = #input_enum_type;
                type OutputEnum = #output_enum_type;
                type Sender = #sender_type;
                type Subscriber = #subscriber_type;
                type InputChannel = #input_channel_type;
                type OutputChannel = #output_channel_type;
                
                async fn map_input(
                    in_channel: &Self::InputChannel,
                    input: &mut Self::Input,
                ) {
                    #map_input_body
                }

                fn map_output(output: &Self::Output, out_channel: &Self::OutputChannel) {
                    #map_output_body
                }
            }

            impl #model_impl_generics #model_ident #model_ty_generics #model_where_clause {
                /// Constructor for RtEngine.
                pub fn into_rt_engine(mut self) -> ::xdevs::rt_engine::RtEngine<Self> {
                    ::xdevs::rt_engine::RtEngine::new(
                        self,
                        #input_channel_call,
                        #output_channel_call,
                    )
                }
            }
        });

        // Combine all generated code into the final output
        generated.extend(quote::quote! {
            /// Hidden module containing auto-generated infrastructure for the top-level component.
            mod #private_mod_ident {
                #private
            }
        });

        generated
    }
}

/// Returns the leaf `syn::Type` from a `PortInnerType`, recursing through `Array` wrappers.
/// This extracts only the type inside `Port<T, N>`, ignoring outer `&'a` or `[...; N]` wrappers.
fn inner_leaf_type(inner: &PortInnerType) -> &syn::Type {
    match inner {
        PortInnerType::Single(ty) => ty,
        PortInnerType::Array(nested) => inner_leaf_type(nested),
    }
}

/// Extracts port information from a field type.
/// Handles both `Port<T, N>` and `[Port<T, N>; M]` types.
fn extract_port_info(field_name: Ident, ty: &syn::Type) -> Option<PortInfo> {
    match ty {
        syn::Type::Path(type_path) => {
            let last_segment = type_path.path.segments.last()?;
            if last_segment.ident != "Port" {
                return None;
            }
            match &last_segment.arguments {
                syn::PathArguments::AngleBracketed(args) => {
                    let inner_type = args.args.first().and_then(|arg| {
                        if let syn::GenericArgument::Type(t) = arg {
                            Some(t.clone())
                        } else {
                            None
                        }
                    })?;
                    Some(PortInfo {
                        field_name,
                        inner_type: PortInnerType::Single(inner_type),
                    })
                }
                _ => None,
            }
        }
        syn::Type::Array(type_array) => {
            let inner_info = extract_port_info(field_name, &type_array.elem)?;
            Some(PortInfo {
                field_name: inner_info.field_name,
                inner_type: PortInnerType::Array(Box::new(inner_info.inner_type)),
            })
        }
        _ => None,
    }
}

fn port_payload_type_tokens(inner_type: &PortInnerType) -> TokenStream2 {
    match inner_type {
        PortInnerType::Single(inner) => quote::quote! { #inner },
        PortInnerType::Array(inner) => {
            let payload = port_payload_type_tokens(inner);
            quote::quote! { (usize, #payload) }
        }
    }
}

/// Converts a snake_case identifier to PascalCase.
fn to_pascal_case_ident(ident: &Ident) -> Ident {
    let s = ident.to_string();
    let pascal: String = s
        .split('_')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                Some(c) => c.to_uppercase().collect::<String>() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect();
    Ident::new(&pascal, ident.span())
}

/// Converts PascalCase to SCREAMING_SNAKE_CASE.
fn to_screaming_snake_case(s: &str) -> String {
    let mut result = String::new();
    for (i, ch) in s.chars().enumerate() {
        if ch.is_uppercase() && i > 0 {
            result.push('_');
        }
        result.push(ch.to_ascii_uppercase());
    }
    result
}

/// Converts PascalCase to snake_case.
fn to_snake_case(s: &str) -> String {
    let mut result = String::new();
    for (i, ch) in s.chars().enumerate() {
        if ch.is_uppercase() && i > 0 {
            result.push('_');
        }
        result.push(ch.to_ascii_lowercase());
    }
    result
}
