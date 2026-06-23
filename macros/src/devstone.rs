extern crate proc_macro;

use quote::{format_ident, quote};
use syn::{
    parse::{Parse, ParseStream},
    Result, Token,
};

pub struct GenerateArgs {
    width: syn::LitInt,
    depth: syn::LitInt,
}

impl Parse for GenerateArgs {
    fn parse(input: ParseStream) -> Result<Self> {
        let width: syn::LitInt = input.parse()?;
        input.parse::<Token![,]>()?;
        let depth: syn::LitInt = input.parse()?;

        if !input.is_empty() {
            return Err(input.error("expected only two arguments: width, depth"));
        }

        Ok(Self { width, depth })
    }
}

pub(crate) fn expand_li(args: GenerateArgs) -> Result<proc_macro2::TokenStream> {
    let width_val: usize = args.width.base10_parse()?;
    let depth_val: usize = args.depth.base10_parse()?;

    if width_val < 1 {
        return Err(syn::Error::new(
            args.width.span(),
            "width must be at least 1",
        ));
    }
    if depth_val < 1 {
        return Err(syn::Error::new(
            args.depth.span(),
            "depth must be at least 1",
        ));
    }

    let width_minus_one = syn::LitInt::new(&(width_val - 1).to_string(), args.width.span());

    let mut token = proc_macro2::TokenStream::new();

    for val in 1..(depth_val + 1) {
        if val == 1 {
            if val != depth_val {
                token.extend(quote! {
                    let model_1 = ::alloc::boxed::Box::new(::xdevs::devstone::li::LIEnum::Leaf(::xdevs::devstone::common::LeafModel::new().to_simulator()));
                })
            } else {
                token.extend(quote! {
                    let model_1 = ::xdevs::devstone::li::LIEnum::Leaf(::xdevs::devstone::common::LeafModel::new().to_simulator());
                });
            }
        } else {
            let val_minus_one = val - 1;
            let model_name = format_ident!("model_{}", val);
            let prev_model = format_ident!("model_{}", val_minus_one);
            if val != depth_val {
                token.extend(quote! {
                    let #model_name = ::alloc::boxed::Box::new(::xdevs::devstone::li::LIEnum::Branch(::xdevs::devstone::li::LIModel::<#width_minus_one>::new(#prev_model).to_simulator()));
                });
            } else {
                token.extend(quote! {
                    let #model_name = ::xdevs::devstone::li::LIEnum::Branch(::xdevs::devstone::li::LIModel::<#width_minus_one>::new(#prev_model).to_simulator());
                });
            }
        }
    }

    let model_name = format_ident!("model_{}", depth_val);
    token.extend(quote! {let model_li = #model_name ;});
    Ok(token)
}

pub(crate) fn expand_hi(args: GenerateArgs) -> Result<proc_macro2::TokenStream> {
    let width_val: usize = args.width.base10_parse()?;
    let depth_val: usize = args.depth.base10_parse()?;

    if width_val < 1 {
        return Err(syn::Error::new(
            args.width.span(),
            "width must be at least 1",
        ));
    }
    if depth_val < 1 {
        return Err(syn::Error::new(
            args.depth.span(),
            "depth must be at least 1",
        ));
    }

    let width_minus_one = syn::LitInt::new(&(width_val - 1).to_string(), args.width.span());

    let mut token = proc_macro2::TokenStream::new();

    for val in 1..(depth_val + 1) {
        if val == 1 {
            if val != depth_val {
                token.extend(quote! {
                    let model_1 = ::alloc::boxed::Box::new(::xdevs::devstone::hi::HIEnum::Leaf(::xdevs::devstone::common::LeafModel::new().to_simulator()));
                })
            } else {
                token.extend(quote! {
                    let model_1 = ::xdevs::devstone::hi::HIEnum::Leaf(::xdevs::devstone::common::LeafModel::new().to_simulator());
                });
            }
        } else {
            let val_minus_one = val - 1;
            let model_name = format_ident!("model_{}", val);
            let prev_model = format_ident!("model_{}", val_minus_one);
            if val != depth_val {
                token.extend(quote! {
                    let #model_name = ::alloc::boxed::Box::new(::xdevs::devstone::hi::HIEnum::Branch(::xdevs::devstone::hi::HIModel::<#width_minus_one>::new(#prev_model).to_simulator()));
                });
            } else {
                token.extend(quote! {
                    let #model_name = ::xdevs::devstone::hi::HIEnum::Branch(::xdevs::devstone::hi::HIModel::<#width_minus_one>::new(#prev_model).to_simulator());
                });
            }
        }
    }

    let model_name = format_ident!("model_{}", depth_val);
    token.extend(quote! {let model_hi = #model_name ;});
    Ok(token)
}

pub(crate) fn expand_ho(args: GenerateArgs) -> Result<proc_macro2::TokenStream> {
    let width_val: usize = args.width.base10_parse()?;
    let depth_val: usize = args.depth.base10_parse()?;

    if width_val < 1 {
        return Err(syn::Error::new(
            args.width.span(),
            "width must be at least 1",
        ));
    }
    if depth_val < 1 {
        return Err(syn::Error::new(
            args.depth.span(),
            "depth must be at least 1",
        ));
    }

    let width_minus_one = syn::LitInt::new(&(width_val - 1).to_string(), args.width.span());

    let mut token = proc_macro2::TokenStream::new();

    for val in 1..(depth_val + 1) {
        if val == 1 {
            if val != depth_val {
                token.extend(quote! {
                    let model_1 = ::alloc::boxed::Box::new(::xdevs::devstone::ho::HOEnum::Leaf(::xdevs::devstone::ho::LeafModel::<#width_minus_one>::new().to_simulator()));
                })
            } else {
                token.extend(quote! {
                    let model_1 = ::xdevs::devstone::ho::HOEnum::Leaf(::xdevs::devstone::ho::LeafModel::<#width_minus_one>::new().to_simulator());
                });
            }
        } else {
            let val_minus_one = val - 1;
            let model_name = format_ident!("model_{}", val);
            let prev_model = format_ident!("model_{}", val_minus_one);
            if val != depth_val {
                token.extend(quote! {
                    let #model_name = ::alloc::boxed::Box::new(::xdevs::devstone::ho::HOEnum::Branch(::xdevs::devstone::ho::HOModel::<#width_minus_one>::new(#prev_model).to_simulator()));
                });
            } else {
                token.extend(quote! {
                    let #model_name = ::xdevs::devstone::ho::HOEnum::Branch(::xdevs::devstone::ho::HOModel::<#width_minus_one>::new(#prev_model).to_simulator());
                });
            }
        }
    }

    let model_name = format_ident!("model_{}", depth_val);
    token.extend(quote! {let model_ho = #model_name ;});
    Ok(token)
}
