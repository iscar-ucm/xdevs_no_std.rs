use proc_macro2::TokenStream as TokenStream2;
use syn::{
    parse::{Parse, ParseStream},
    Error, Ident, ItemStruct, LitInt, Token,
};

// Import Field and filter_generics from parent module to reuse parsing logic
use super::{filter_generics, Field};

/// Arguments for the `#[top]` attribute macro.
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

/// Parse the struct and generate the top-level infrastructure:
/// - Input/Output enums for channel communication
/// - Static input `Channel` and output `PubSubChannel`
/// - An `AsyncInput` handler struct
/// - A `propagate_output` function
///
/// The original item (with inner attributes like `#[atomic]`, `#[coupled]`, etc.)
/// is passed through unchanged so those macros can process the struct next.
pub fn parse_and_generate(args: TokenStream2, item: TokenStream2) -> syn::Result<TokenStream2> {
    let args: TopArgs = syn::parse2(args)?;
    let component: ItemStruct = syn::parse2(item.clone())?;

    let model_ident = &component.ident;

    // Parse fields to extract input and output ports using the same pattern as coupled2.rs
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

    // Get generics and filter them for input and output structs
    let generics = component.generics.clone();
    let input_generics = filter_generics(&inputs, &generics);
    let output_generics = filter_generics(&outputs, &generics);
    let (_, input_ty_generics, _) = input_generics.split_for_impl();
    let (_, output_ty_generics, _) = output_generics.split_for_impl();

    // Generate identifiers for the infrastructure types
    let input_enum_ident = quote::format_ident!("{}InputEnum", model_ident);
    let output_enum_ident = quote::format_ident!("{}OutputEnum", model_ident);
    let input_ident = quote::format_ident!("{}Input", model_ident);
    let output_ident = quote::format_ident!("{}Output", model_ident);

    let upper_name = to_screaming_snake_case(&model_ident.to_string());
    let in_channel_ident = quote::format_ident!("{}_IN_CHANNEL", upper_name);
    let out_channel_ident = quote::format_ident!("{}_OUT_CHANNEL", upper_name);

    let handler_ident = quote::format_ident!("{}InputHandler", model_ident);

    let in_size = args.in_size;
    let out_size = args.out_size;
    let max_out_subs = args.max_out_subs;

    // Hidden module items
    let snake_name = to_snake_case(&model_ident.to_string());
    let private_mod_ident = quote::format_ident!("__xdevs_no_std_private_{}_rt_engine", snake_name);
    let mut private = TokenStream2::new();

    // Build generated code starting with the original item (pass-through)
    let mut generated = item;

    // Generate input infrastructure (enum, channel, handler)
    if !input_ports.is_empty() {
        let input_variants: Vec<TokenStream2> = input_ports
            .iter()
            .map(|info| {
                let variant = to_pascal_case_ident(&info.field_name);
                let ty = port_payload_type_tokens(&info.inner_type);
                quote::quote! { #variant(#ty) }
            })
            .collect();

        let match_arms: Vec<TokenStream2> = input_ports
            .iter()
            .map(|info| {
                let variant = to_pascal_case_ident(&info.field_name);
                let field = &info.field_name;
                match &info.inner_type {
                    PortInnerType::Single(_) => quote::quote! {
                        #input_enum_ident::#variant(value) => input.#field.add_value(value).unwrap()
                    },
                    PortInnerType::Array(_) => quote::quote! {
                        #input_enum_ident::#variant((index, value)) => {
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

        generated.extend(quote::quote! {
            /// Auto-generated input enum for top-level channel communication.
            pub enum #input_enum_ident {
                #(#input_variants),*
            }
        });

        private.extend(quote::quote! {
            /// Auto-generated static input channel.
            pub static #in_channel_ident: ::xdevs::export::Channel<
                ::xdevs::export::Mutex,
                #input_enum_ident,
                #in_size
            > = ::xdevs::export::Channel::new();

            /// Auto-generated input handler implementing `AsyncInput`.
            pub struct #handler_ident {
                last_rt: Option<::xdevs::Instant>,
            }

            impl #handler_ident {
                /// Creates a new input handler.
                pub fn new() -> Self {
                    Self { last_rt: None }
                }
            }

            impl ::xdevs::traits::AsyncInput for #handler_ident {
                type Input = #input_ident #input_ty_generics;

                async fn handle(
                    &mut self,
                    config: &::xdevs::simulator::Config,
                    t_from: f64,
                    t_until: f64,
                    input: &mut Self::Input,
                ) -> f64 {
                    let last_rt = self.last_rt.unwrap_or_else(::xdevs::Instant::now);
                    let time_duration = (t_until - t_from) * config.time_scale;
                    let time_duration = (time_duration * 1_000_000_000.0) as u64;
                    let next_rt = last_rt + ::xdevs::Duration::from_nanos(time_duration);

                    let future = async {
                        // Wait for at least one input event
                        let rcv = #in_channel_ident.receive().await;
                        match rcv {
                            #(#match_arms),*
                        };
                        // Drain all additional inputs that arrived at the same time
                        while let Ok(rcv) = #in_channel_ident.try_receive() {
                            match rcv {
                                #(#match_arms),*
                            };
                        }
                    };

                    if let Err(_) = ::xdevs::export::with_deadline(next_rt.into(), future).await {
                        // Deadline reached (timeout), check for jitter
                        if let Some(max_jitter) = config.max_jitter {
                            let jitter = ::xdevs::Instant::now().duration_since(next_rt);
                            let max_jitter_ticks = ::xdevs::Duration::from_micros(max_jitter.as_micros() as u64);
                            if jitter > max_jitter_ticks {
                                panic!("Jitter too high");
                            }
                        }
                        self.last_rt = Some(next_rt);
                        return t_until;
                    } else {
                        let now = ::xdevs::Instant::now();
                        self.last_rt = Some(now);
                        let elapsed_rt = now.duration_since(last_rt).as_micros() as f64 / 1_000_000.0;
                        let elapsed_sim = elapsed_rt / config.time_scale;
                        return t_from + elapsed_sim;
                    }
                }
            }
        });
    }

    // Generate output infrastructure (enum, channel, propagate function)
    if !output_ports.is_empty() {
        let output_variants: Vec<TokenStream2> = output_ports
            .iter()
            .map(|info| {
                let variant = to_pascal_case_ident(&info.field_name);
                let ty = port_payload_type_tokens(&info.inner_type);
                quote::quote! { #variant(#ty) }
            })
            .collect();

        let propagations: Vec<TokenStream2> = output_ports
            .iter()
            .map(|info| {
                let variant = to_pascal_case_ident(&info.field_name);
                let field = &info.field_name;
                match &info.inner_type {
                    PortInnerType::Single(_) => quote::quote! {
                        if let Some(&value) = output.#field.get_values().last() {
                            #private_mod_ident::#out_channel_ident.immediate_publisher()
                                .publish_immediate(#output_enum_ident::#variant(value));
                        }
                    },
                    PortInnerType::Array(_) => quote::quote! {
                        for (index, port) in output.#field.iter().enumerate() {
                            if let Some(&value) = port.get_values().last() {
                                #private_mod_ident::#out_channel_ident.immediate_publisher()
                                    .publish_immediate(#output_enum_ident::#variant((index, value)));
                            }
                        }
                    },
                }
            })
            .collect();

        generated.extend(quote::quote! {
            /// Auto-generated output enum for top-level channel communication.
            #[derive(Clone, PartialEq)]
            pub enum #output_enum_ident {
                #(#output_variants),*
            }
        });

        private.extend(quote::quote! {
            /// Auto-generated static output PubSub channel.
            pub static #out_channel_ident: ::xdevs::export::PubSubChannel<
                ::xdevs::export::Mutex,
                #output_enum_ident,
                #out_size,
                #max_out_subs,
                1
            > = ::xdevs::export::PubSubChannel::new();

            /// Auto-generated function to propagate model outputs to the output channel.
            pub fn propagate_output(output: &#output_ident #output_ty_generics) {
                #(#propagations)*
            }
        });
    }

    // Generate the rt_engine within the private module
    generated.extend(quote::quote! {
        /// Auto-generated real-time engine for the top-level component.
        impl #model_ident {
            pub fn into_rt_engine(self) -> ::xdevs::export::RtEngine<
            'static,
            Self,
            #input_enum_ident,
            #output_enum_ident,
            #private_mod_ident::#handler_ident,
            impl FnMut(&<Self as ::xdevs::traits::Component>::Output #output_ty_generics),
            #in_size, #out_size, #max_out_subs> {
                ::xdevs::export::RtEngine::new(
                    self,
                    &#private_mod_ident::#in_channel_ident,
                    #private_mod_ident::#handler_ident::new(),
                    &#private_mod_ident::#out_channel_ident,
                    #private_mod_ident::propagate_output)
            }
        }
    });

    // Combine all generated code into the final output
    generated.extend(quote::quote! {
        /// Hidden module containing auto-generated infrastructure for the top-level component.
        mod #private_mod_ident {
            use super::*;
            #private
        }
    });

    Ok(generated)
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
