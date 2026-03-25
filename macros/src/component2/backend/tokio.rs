pub struct RtEngineBackend {}

impl RtEngineBackend {
    pub fn new() -> Self {
        Self {}
    }
    pub fn parse_max_out_subs(&mut self, _: usize) -> Result<(), syn::Error> {
        return Err(syn::Error::new(
            proc_macro2::Span::call_site(),
            "max_out_subs is not supported in the std backend",
        ));
    }
}
