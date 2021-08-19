use proc_macro2::TokenStream;
use syn::{Data, DeriveInput, Field};

/// Codegen for implementing the [cachem::Get2] trait.
///
/// # Params
///
/// * `input` - Information about the struct the trait should be implmented on
///
/// # Error
///
/// Returns an error when the struct has no primary key.
///
/// # Returns
///
/// New [proc_macro2::TokenStream] containung the implementation of the trait.
///
pub fn code_gen(input: DeriveInput) -> Result<TokenStream, TokenStream> {
    let struct_fields = struct_fields(&input)?;

    if !struct_fields.iter().any(has_primary_attr) {
        return Err(crate::utils::error(
                    input.ident.span(),
                    "Struct has no primary key field".into()
                )
            )
    }

    todo!()
}

/// Extracts all fields from the struct.
///
/// # Params
///
/// * `input` - Information about the struct the fields should be extract from
///
/// # Error
///
/// Returns an error of the struct type is not supported
///
/// # Returns
///
/// Vec of [syn::Field]
///
fn struct_fields(input: &DeriveInput) -> Result<Vec<Field>, TokenStream> {
    match &input.data {
        Data::Struct(x) => {
            let fields = x.fields
                .iter()
                .cloned()
                .collect::<Vec<_>>();
            Ok(fields)
        },
        _ => Err(crate::utils::error(
                    input.ident.span(),
                    "The given type is not supported".into()
                )
            )
    }
}

/// Checks that the struct has a field marked as primary key
///
/// # Params
///
/// Single syn::Field
///
/// # Returns
///
/// `true`  - when the field is marked as primary
/// `false` - when the field is not marked as primary
///
fn has_primary_attr(field: &Field) -> bool {
    todo!();
    !field.attrs.is_empty()
}
