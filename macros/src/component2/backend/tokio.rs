pub(crate) fn parse_max_out_subs(
    _: &syn::Ident,
    _: &Option<usize>,
    _: usize,
) -> Result<usize, syn::Error> {
    return Err(syn::Error::new(
        proc_macro2::Span::call_site(),
        "max_out_subs is not supported in the std backend",
    ));
}
