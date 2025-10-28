pub mod atomic;
pub mod coupled;
mod port;
mod state;

pub struct Field {
    ident: syn::Ident,
    ty: syn::Type,
}
