use proc_macro::TokenStream;

mod component;
mod component2;

#[proc_macro]
pub fn component(input: TokenStream) -> TokenStream {
    let component: component::Component = syn::parse_macro_input!(input);
    component.quote().into()
}

// #[proc_macro]
// pub fn atomic(input: TokenStream) -> TokenStream {
//     let atomic_meta: atomic::AtomicMeta = syn::parse_macro_input!(input);
//     atomic_meta.quote().into()
// }

// #[proc_macro]
// pub fn coupled(input: TokenStream) -> TokenStream {
//     let coupled_meta: coupled::CoupledMeta = syn::parse_macro_input!(input);
//     coupled_meta.quote().into()
// }

#[proc_macro_attribute]
pub fn atomic(_args: TokenStream, item: TokenStream) -> TokenStream {
    let atomic_component = component2::atomic::Component::parse(item.into());
    match atomic_component {
        Ok(component) => component.quote().into(),
        Err(err) => err.to_compile_error().into(),
    }
}

#[proc_macro_attribute]
pub fn coupled(args: TokenStream, item: TokenStream) -> TokenStream {
    let coupled_component = component2::coupled::Component::parse(args.into(), item.into());
    match coupled_component {
        Ok(component) => component.quote().into(),
        Err(err) => err.to_compile_error().into(),
    }
}

#[proc_macro_attribute]
pub fn coupled2(args: TokenStream, item: TokenStream) -> TokenStream {
    let coupled_component = component2::coupled2::Component::parse(args.into(), item.into());
    match coupled_component {
        Ok(component) => component.quote().into(),
        Err(err) => err.to_compile_error().into(),
    }
}

#[proc_macro_attribute]
pub fn rt_engine(args: TokenStream, item: TokenStream) -> TokenStream {
    let component = component2::rt_engine::Component::parse(args.into(), item.into());
    match component {
        Ok(component) => component.quote().into(),
        Err(err) => err.to_compile_error().into(),
    }
}
