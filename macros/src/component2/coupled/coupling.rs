use proc_macro2::{TokenStream as TokenStream2, TokenTree as TokenTree2};
use quote::quote;
use std::collections::HashSet;
use std::hash::{Hash, Hasher};
use syn::{
    parse::{Parse, ParseStream},
    token::Brace,
    Error, Ident, Token,
};

use super::Field;

#[derive(Clone)]
pub struct Coupling {
    pub first_source_ident: Ident,
    pub source_1: TokenStream2,
    pub source_2: TokenStream2,
    pub source_iter_chain: Option<TokenStream2>,
    pub source_is_zipped: bool,
    pub first_dest_ident: Ident,
    pub destination_1: TokenStream2,
    pub destination_2: TokenStream2,
    pub dest_iter_chain: Option<TokenStream2>,
}

impl PartialEq for Coupling {
    fn eq(&self, other: &Self) -> bool {
        self.first_source_ident == other.first_source_ident
            && self.source_1.to_string() == other.source_1.to_string()
            && self.source_2.to_string() == other.source_2.to_string()
            && self.source_iter_chain.as_ref().map(|t| t.to_string()) == other.source_iter_chain.as_ref().map(|t| t.to_string())
            && self.source_is_zipped == other.source_is_zipped
            && self.first_dest_ident == other.first_dest_ident
            && self.destination_1.to_string() == other.destination_1.to_string()
            && self.destination_2.to_string() == other.destination_2.to_string()
            && self.dest_iter_chain.as_ref().map(|t| t.to_string()) == other.dest_iter_chain.as_ref().map(|t| t.to_string())
    }
}

impl Eq for Coupling {}

impl Hash for Coupling {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.first_source_ident.hash(state);
        self.source_1.to_string().hash(state);
        self.source_2.to_string().hash(state);
        self.source_iter_chain.as_ref().map(|t| t.to_string()).hash(state);
        self.source_is_zipped.hash(state);
        self.first_dest_ident.hash(state);
        self.destination_1.to_string().hash(state);
        self.destination_2.to_string().hash(state);
        self.dest_iter_chain.as_ref().map(|t| t.to_string()).hash(state);
    }
}

impl Coupling {
    pub fn is_eoc(&self, outputs: &[Field]) -> bool {
        outputs.iter().any(|f| f.ident == self.first_dest_ident)
    }

    /// Build the base path for source access
    fn build_source_base(&self, inputs: &[Field], components: &[Field]) -> TokenStream2 {
        let first_source = &self.first_source_ident;
        let source_1 = &self.source_1;
        let source_2 = &self.source_2;

        let is_source_input = inputs.iter().any(|f| f.ident == *first_source);
        let is_source_component = components.iter().any(|f| f.ident == *first_source);
        let source_is_component_array = is_source_component && source_1.is_empty() && source_2.is_empty();

        if is_source_input {
            quote!(self.input.#first_source #source_1 #source_2)
        } else if source_is_component_array {
            // Iterator directly on component array, user specifies in map
            quote!(self.components.#first_source)
        } else if is_source_component {
            // Iterator on port, inject .output before port access
            quote!(self.components.#first_source #source_1 .output #source_2)
        } else {
            quote!(self.input.#first_source #source_1 #source_2)
        }
    }

    /// Build the base path for destination access
    fn build_dest_base(&self, outputs: &[Field], components: &[Field]) -> TokenStream2 {
        let first_dest = &self.first_dest_ident;
        let destination_1 = &self.destination_1;
        let destination_2 = &self.destination_2;

        let is_dest_output = outputs.iter().any(|f| f.ident == *first_dest);
        let is_dest_component = components.iter().any(|f| f.ident == *first_dest);
        let dest_is_component_array = is_dest_component && destination_1.is_empty() && destination_2.is_empty();

        if is_dest_output {
            quote!(self.output.#first_dest #destination_1 #destination_2)
        } else if dest_is_component_array {
            // Iterator directly on component array, user specifies in map
            quote!(self.components.#first_dest)
        } else if is_dest_component {
            // Iterator on port, inject .output before port access
            quote!(self.components.#first_dest #destination_1 .input #destination_2)
        } else {
            quote!(self.output.#first_dest #destination_1 #destination_2)
        }
    }

    pub fn quote(&self, inputs: &[Field], outputs: &[Field], components: &[Field]) -> TokenStream2 {
        let first_source = &self.first_source_ident;
        let first_dest = &self.first_dest_ident;

        let is_source_input = inputs.iter().any(|f| f.ident == *first_source);
        let is_dest_output = outputs.iter().any(|f| f.ident == *first_dest);

        if is_source_input && is_dest_output {
            let error = Error::new(
                self.span(),
                "invalid coupling: cannot connect input directly to output without going through a component",
            );
            return error.to_compile_error();
        }

        let source_has_iter = self.source_iter_chain.is_some();
        let dest_has_iter = self.dest_iter_chain.is_some();

        match (source_has_iter, dest_has_iter, self.source_is_zipped) {
            // zip(source_iter) -> dest_iter: 1-to-1 zipped coupling
            (true, true, true) => {
                let source_iter = self.source_iter_chain.as_ref().unwrap();
                let dest_iter = self.dest_iter_chain.as_ref().unwrap();
                let origin_base = self.build_source_base(inputs, components);
                let destination_base = self.build_dest_base(outputs, components);

                quote! {
                    {
                        for (src, dst) in (#origin_base #source_iter).zip(#destination_base #dest_iter) {
                            dst.add_values(&src.get_values()).expect("unable to propagate messages; destination port is full");
                        }
                    }
                }
            }
            // source_iter -> dest_iter (no zip): all-to-all coupling
            (true, true, false) => {
                let source_iter = self.source_iter_chain.as_ref().unwrap();
                let dest_iter = self.dest_iter_chain.as_ref().unwrap();
                let origin_base = self.build_source_base(inputs, components);
                let destination_base = self.build_dest_base(outputs, components);

                quote! {
                    {
                        for src in #origin_base #source_iter {
                            for dst in #destination_base #dest_iter {
                                dst.add_values(&src.get_values()).expect("unable to propagate messages; destination port is full");
                            }
                        }
                    }
                }
            }
            // source_port -> dest_iter: one to many
            (false, true, _) => {
                let dest_iter = self.dest_iter_chain.as_ref().unwrap();
                let origin = self.build_source_base(inputs, components);
                let destination_base = self.build_dest_base(outputs, components);

                quote! {
                    {
                        let values = #origin.get_values();
                        for dst in #destination_base #dest_iter {
                            dst.add_values(&values).expect("unable to propagate messages; destination port is full");
                        }
                    }
                }
            }
            // source_iter -> dest_port: many to one
            (true, false, _) => {
                let source_iter = self.source_iter_chain.as_ref().unwrap();
                let origin_base = self.build_source_base(inputs, components);
                let destination = self.build_dest_base(outputs, components);

                quote! {
                    {
                        for src in #origin_base #source_iter {
                            #destination.add_values(&src.get_values()).expect("unable to propagate messages; destination port is full");
                        }
                    }
                }
            }
            // source_port -> dest_port: simple single port-to-port coupling
            (false, false, _) => {
                let origin = self.build_source_base(inputs, components);
                let destination = self.build_dest_base(outputs, components);

                quote! {
                    #destination.add_values(&#origin.get_values()).expect("unable to propagate messages; destination port is full");
                }
            }
        }
    }

    pub fn span(&self) -> proc_macro2::Span {
        let start = self.first_source_ident.span();
        let end = self.first_dest_ident.span();

        start.join(end).unwrap_or_else(|| start)
    }
}

impl Parse for Coupling {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // Helper function to check if an identifier is an iterator method
        fn is_iter_method(ident: &Ident) -> bool {
            let name = ident.to_string();
            name == "iter" || name == "iter_mut"
        }

        // Helper function to parse source/destination with the following logic:
        // - First token is the initial identifier
        // - Accumulate tokens into "part_1" until it finds a `.identifier` not followed by `()`, a port
        // - That identifier becomes the first element of "part_2", and the rest follows
        // - If it encounters `.iter()` or `.iter_mut()`, capture from there as iter_chain
        fn parse_part(
            input: ParseStream,
            end_condition: impl Fn(&ParseStream) -> bool,
        ) -> syn::Result<(Ident, TokenStream2, TokenStream2, Option<TokenStream2>)> {
            let mut part_1 = TokenStream2::new();
            let mut part_2 = TokenStream2::new();
            let mut iter_chain = TokenStream2::new();
            let mut found_split = false;
            let mut found_iter = false;
            let mut split_ident: Option<Ident> = None;

            // First token should be an identifier
            let initial_ident: Ident = input.parse()?;

            while !input.is_empty() && !end_condition(&input) {
                if input.peek(Token![.]) {
                    // Consume the dot
                    input.parse::<Token![.]>()?;

                    if input.peek(syn::Ident) {
                        let ident: Ident = input.parse()?;

                        // Check if this identifier is followed by `()`
                        let has_parens = input.peek(syn::token::Paren);

                        // Check if this is an iter method, if so, start capturing iter_chain
                        if is_iter_method(&ident) && has_parens && !found_iter {
                            found_iter = true;
                            iter_chain.extend(quote!(. #ident));
                        } else if found_iter {
                            // Already in iter chain, keep adding
                            iter_chain.extend(quote!(. #ident));
                        } else if !has_parens && !found_split {
                            // This ident starts part_2
                            found_split = true;
                            split_ident = Some(ident);
                        } else if found_split {
                            part_2.extend(quote!(. #ident));
                        } else {
                            part_1.extend(quote!(. #ident));
                        }
                    } else {
                        if found_iter {
                            iter_chain.extend(quote!(.));
                        } else if found_split {
                            part_2.extend(quote!(.));
                        } else {
                            part_1.extend(quote!(.));
                        }
                    }
                } else {
                    // Any other token (including parentheses)
                    let tt: TokenTree2 = input.parse()?;
                    if found_iter {
                        iter_chain.extend(core::iter::once(tt));
                    } else if found_split {
                        part_2.extend(core::iter::once(tt));
                    } else {
                        part_1.extend(core::iter::once(tt));
                    }
                }
            }

            let final_iter_chain = if found_iter {
                Some(iter_chain)
            } else {
                None
            };

            if !found_split {
                Ok((initial_ident, part_1, TokenStream2::new(), final_iter_chain))
            } else {
                let split_ident = split_ident.unwrap();
                let mut full_part_2 = quote!(.#split_ident);
                full_part_2.extend(part_2);
                Ok((initial_ident, part_1, full_part_2, final_iter_chain))
            }
        }

        // Check if source is wrapped in zip(...)
        let source_is_zipped = input.peek(syn::Ident) && input.fork().parse::<Ident>().map(|i| i == "zip").unwrap_or(false);
        
        let (first_source_ident, source_1, source_2, source_iter_chain) = if source_is_zipped {
            // Consume "zip"
            input.parse::<Ident>()?;
            // Parse the parenthesized content
            let content;
            syn::parenthesized!(content in input);
            parse_part(&content, |inp| inp.is_empty())?
        } else {
            parse_part(input, |inp| inp.peek(Token![->]))?
        };
        
        input.parse::<Token![->]>()?; // consume the '->'

        // Parse destination
        let (first_dest_ident, destination_1, destination_2, dest_iter_chain) =
            parse_part(input, |inp| inp.peek(Token![,]))?;

        Ok(Self {
            first_source_ident,
            source_1,
            source_2,
            source_iter_chain,
            source_is_zipped,
            first_dest_ident,
            destination_1,
            destination_2,
            dest_iter_chain,
        })
    }
}

pub struct Couplings {
    pub _brace: Brace,
    pub couplings: Vec<Coupling>,
}

impl Couplings {
    pub fn quote(
        &self,
        inputs: &[Field],
        outputs: &[Field],
        components: &[Field],
    ) -> (Vec<TokenStream2>, Vec<TokenStream2>) {
        let mut eoc = Vec::new();
        let mut xic = Vec::new();

        for coupling in &self.couplings {
            if coupling.is_eoc(outputs) {
                eoc.push(coupling.quote(inputs, outputs, components));
            } else {
                xic.push(coupling.quote(inputs, outputs, components));
            }
        }
        (eoc, xic)
    }
}

impl Parse for Couplings {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        let brace = syn::braced!(content in input);
        let mut couplings = Vec::new();
        let mut cache = HashSet::new();

        while !content.is_empty() {
            let coupling = content.parse::<Coupling>()?;
            if cache.contains(&coupling) {
                return Err(Error::new(coupling.span(), "duplicate coupling"));
            }
            cache.insert(coupling.clone());

            couplings.push(coupling);
            if !content.is_empty() {
                content.parse::<Token![,]>()?; // comma between meta arguments
            }
        }
        Ok(Self {
            _brace: brace,
            couplings,
        })
    }
}
