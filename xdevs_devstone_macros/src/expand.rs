use crate::GenerateArgs;
use quote::{format_ident, quote};
use syn::Result;

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
            // token.extend(quote! {
            //     let model_1 = Coup::CoupD(CoupAtom::new());
            // });
            if val != depth_val {
                token.extend(quote! {
                    let model_1 = ::std::boxed::Box::new(Coup::CoupD(CoupAtom::new()));
                })
            } else {
                token.extend(quote! {
                    let model_1 = Coup::CoupD(CoupAtom::new());
                });
            }
        } else {
            let val_minus_one = val - 1;
            let model_name = format_ident!("model_{}", val);
            let prev_model = format_ident!("model_{}", val_minus_one);
            if val != depth_val {
                token.extend(quote! {
                    let #model_name = ::std::boxed::Box::new(Coup::RestoCoup(ModCoupLI::<#width_minus_one>::new(#prev_model)));
                });
            } else {
                token.extend(quote! {
                    let #model_name = Coup::RestoCoup(ModCoupLI::<#width_minus_one>::new(#prev_model));
                });
            }

            // let model_name = format_ident!("model_{}", val);
            // let model_ref = if val != depth_val {
            //     format_ident!("model_{}_ref", val)
            // } else {
            //     format_ident!("model_{}", val)
            // };
            // let prev_model_ref = format_ident!("model_{}_ref", val_minus_one);
            // token.extend(quote! {
            //     let #model_ref = ::std::boxed::Box::new(Coup::RestoCoup(ModCoupLI::<#width_minus_one>::new(#prev_model_ref)));
            // });

            // if val != depth_val {
            //     token.extend(quote! {
            //         let #model_ref = ::std::boxed::Box::new(#model_name);
            //     })
            // }
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
            token.extend(quote! {
                let model_1 = HI::CoupD(CoupAtom::new());
            });
            if val != depth_val {
                token.extend(quote! {
                    let model_1_ref = ::std::boxed::Box::new(model_1);
                })
            }
        } else {
            let val_minus_one = val - 1;
            let model_name = format_ident!("model_{}", val);
            let model_ref = format_ident!("model_{}_ref", val);
            let prev_model_ref = format_ident!("model_{}_ref", val_minus_one);
            token.extend(quote! {
                let #model_name = HI::RestoCoup(CoupHI::<#width_minus_one>::new(#prev_model_ref));
            });

            if val != depth_val {
                token.extend(quote! {
                    let #model_ref = ::std::boxed::Box::new(#model_name);
                })
            }
        }
    }

    let model_name = format_ident!("model_{}", depth_val);
    token.extend(quote! {let model_li = #model_name ;});
    Ok(token)
}
