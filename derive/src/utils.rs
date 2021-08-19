//! This file contains some general purpose functions

use proc_macro2::{Span, TokenStream};
use syn::{Ident, Type, TypePath};

/// Generates a new [proc_macro2::TokenStream] error
///
/// # Params
///
/// * `span` - Span of the error
/// * `msg`  - Message of the error to display
///
/// # Returns
///
/// [proc_macro2::TokenStream] error
///
pub fn error(span: Span, msg: String) -> TokenStream {
    span
        .unwrap()
        .error(msg)
        .emit();
    TokenStream::new()
}

/// Extracts the datatype from a [syn::Type]
///
/// # Params
///
/// * `typ` - Type where the datatype should be extracted from
///
/// # Returns
///
/// Datatype of the given [syn::Type]
///
pub fn ident_from_type(
    typ: &Type
) -> Ident {
    match typ {
        Type::Path(x) => {
            x
                .path
                .segments
                .first()
                .unwrap()
                .ident
                .clone()
        },
        _ => panic!("Invalid datatype, {:?}", &typ)
    }
}

/// Extracts the datatype from the given [syn::TypePath]
///
/// # Params
///
/// * `type_path` - Path where the datatype should be extracted from
///
/// # Returns
///
/// Datatype from the [sync::TypePath]
///
pub fn get_datatype_enum(
    type_path: &TypePath
) -> Ident {
    type_path
        .path
        .segments
        .first()
        .unwrap()
        .ident
        .clone()
}
