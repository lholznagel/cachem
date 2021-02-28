use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, Fields, Ident, Type};
use syn::spanned::Spanned;

pub fn generate_fn_read(struct_name: &Ident, data: &Data) -> TokenStream {
    match *data {
        Data::Struct(ref data) => {
            match data.fields {
                // Structs with fields
                Fields::Named(ref fields) => {
                    let struct_fields = fields
                        .named
                        .iter()
                        .map(|field| {

                        let field_name = &field.ident;
                        match &field.ty {
                            Type::Path(x) => {
                                if crate::utils::is_collection(&x) {
                                    let datatype = crate::utils::get_collection_datatype(&x);

                                    if crate::utils::is_vec(&x) {
                                        let vec = read_vec(&datatype);
                                        quote! {
                                            #field_name: {
                                                #vec
                                            }
                                        }
                                    } else {
                                        return crate::utils::error(
                                            x.span(),
                                            "Only the collection type Vec is supported.".into()
                                        );
                                    }
                                } else {
                                    let datatype = crate::utils::ident_from_type(&field.ty);
                                    quote! {
                                        #field_name: #datatype::read(buf).await?
                                    }
                                }
                            },
                            _ => {
                                return crate::utils::error(
                                    field.ty.span(),
                                    "The given type is not supported.".into()
                                );
                            }
                        }
                    });
                    quote! {
                        Ok(#struct_name {
                            #(#struct_fields),*
                        })
                    }
                },
                // Struct with only one entry
                Fields::Unnamed(ref fields) => {
                    let datatype = fields
                        .unnamed
                        .iter()
                        .map(|f| {

                        match &f.ty {
                            Type::Path(x) => {
                                if crate::utils::is_collection(&x) {
                                    let datatype = crate::utils::get_collection_datatype(&x);

                                    let code = if crate::utils::is_vec(&x) {
                                        read_vec(&datatype)
                                    } else if crate::utils::is_hashset(&x) {
                                        read_hashset(&datatype)
                                    } else {
                                        return crate::utils::error(
                                            x.span(),
                                            "Only Vec and Hashset is supported.".into()
                                        );
                                    };
                                    quote! {
                                        Ok(Self({ #code }))
                                    }
                                } else {
                                    let datatype = x.path.segments.first().unwrap().ident.clone();
                                    quote! {
                                        Ok(Self(#datatype::read(buf).await?))
                                    }
                                }
                            },
                            _ => {
                                return crate::utils::error(
                                    f.ty.span(),
                                    "The given type is not supported.".into()
                                );
                            }
                        }
                    });
                    quote! {
                        #(#datatype)*
                    }
                },
                // Structs without any fields
                Fields::Unit => quote! {
                    let _ = u8::read(buf).await?;

                    Ok(Self {})
                }
            }
        },
        _ => {
            return crate::utils::error(
                struct_name.span(),
                "Only structs are supported.".into()
            );
        }
    }
}

pub fn generate_fn_write(struct_name: &Ident, data: &Data) -> TokenStream {
    match *data {
        Data::Struct(ref data) => {
            match data.fields.clone() {
                Fields::Named(ref fields) => {
                    let recurse = fields
                        .named
                        .iter()
                        .map(|f| {

                        let field_name = &f.ident;
                        match &f.ty {
                            Type::Path(x) => {
                                if crate::utils::is_vec(x) {
                                    let code = write_collection(&field_name);
                                    quote! { #code }
                                } else {
                                    quote! {
                                        self.#field_name.write(buf).await?;
                                    }
                                }
                            },
                            _ => {
                                return crate::utils::error(
                                    f.ty.span(),
                                    "The given type is not supported.".into()
                                );
                            }
                        }
                    });
                    quote! {
                        #(#recurse)*
                    }
                },
                Fields::Unnamed(fields) => {
                    let datatype = fields
                        .unnamed
                        .iter()
                        .map(|f| {

                        match &f.ty {
                            Type::Path(x) => {
                                if crate::utils::is_collection(x) {
                                    quote! {
                                        u32::from(self.0.len() as u32).write(buf).await?;
                                        for entry in self.0.iter() {
                                            entry.write(buf).await?;
                                        }
                                    }
                                } else {
                                    quote! {
                                        self.0.write(buf).await?;
                                    }
                                }
                            },
                            _ => {
                                return crate::utils::error(
                                    f.ty.span(),
                                    "The given type is not supported.".into()
                                );
                            }
                        }
                    });
                    quote! {
                        #(#datatype)*
                    }
                },
                Fields::Unit => quote! {
                    0u8.write(buf).await?;
                }
            }
        },
        _ => {
            struct_name.span().unwrap().error(format!("Only structs are supported.")).emit();
            return TokenStream::new();
        }
    }
}

fn read_vec(datatype: &Ident) -> TokenStream {
    quote! {
        let entry_count = u32::read(buf).await?;
        let mut entries = Vec::new();

        for _ in 0..entry_count {
            let x = #datatype::read(buf).await?;
            entries.push(x);
        }
        entries
    }
}

fn read_hashset(datatype: &Ident) -> TokenStream {
    quote! {
        let entry_count = u32::read(buf).await?;
        let mut entries = std::collections::HashSet::new();

        for _ in 0..entry_count {
            let x = #datatype::read(buf).await?;
            entries.insert(x);
        }
        entries
    }
}

fn write_collection(field_name: &Option<Ident>) -> TokenStream {
    quote! {
        u32::from(self.#field_name.len() as u32).write(buf).await?;
        for entry in self.#field_name.iter() {
            entry.write(buf).await?
        }
    }
}
