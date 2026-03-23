pub(crate) fn parse_max_out_subs(
    ident: &syn::Ident,
    max_out_subs: &Option<usize>,
    value: usize,
) -> Result<usize, syn::Error> {
    if let Some(_) = max_out_subs {
        Err(syn::Error::new(
            ident.span(),
            "duplicate argument: max_out_subs",
        ))
    } else {
        Ok(value)
    }
}
