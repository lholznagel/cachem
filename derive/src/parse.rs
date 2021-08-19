use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields, Ident, Type};
use syn::spanned::Spanned;

/// Code generator for implementing the [cachem::Parse] trait
///
/// # Params
///
/// * `input` - Input of the struct where the trait should be implemented only
///
/// # Returns
///
/// [proc_macro2::TokenStream] with the implementation of the [cachem::Parse]
/// trait.
///
pub fn code_gen(input: DeriveInput) -> TokenStream {
    let name = input.ident;
    let fn_read  = crate::parse::generate_fn_read(&name, &input.data);
    let fn_write = crate::parse::generate_fn_write(&name, &input.data);

    let generics = input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    quote! {
        #[async_trait::async_trait]
        impl #impl_generics cachem::Parse for #name #ty_generics #where_clause {
            async fn read<B>(
                buf: &mut B
            ) -> Result<Self, cachem::CachemError>
            where
                B: tokio::io::AsyncBufRead + tokio::io::AsyncRead + Send + Unpin {

                #fn_read
            }

            async fn write<B>(
                &self,
                buf: &mut B
            ) -> Result<(), cachem::CachemError>
            where
                B: tokio::io::AsyncWrite + Send + Unpin {

                #fn_write
                Ok(())
            }
        }
    }
}

/// Generates the code for reading bytes to struct
///
/// # Params
///
/// * `struct_name` - Name of the struct
/// * `data`        - Information about the struct
///
/// # Returns
///
/// [proc_macro2::TokenStream] with the implementation of the read function.
///
fn generate_fn_read(struct_name: &Ident, data: &Data) -> TokenStream {
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
                            Type::Path(_) => {
                                let datatype = crate::utils::ident_from_type(&field.ty);
                                quote! {
                                    #field_name: #datatype::read(buf).await?
                                }
                            }
                            _ => {
                                crate::utils::error(
                                    field.ty.span(),
                                    "The given type is not supported.".into()
                                )
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
                                let datatype = x.path.segments.first().unwrap().ident.clone();
                                quote! {
                                    Ok(Self(#datatype::read(buf).await?))
                                }
                            },
                            _ => {
                                crate::utils::error(
                                    f.ty.span(),
                                    "The given type is not supported.".into()
                                )
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
        Data::Enum(ref data) => {
            let fields = data.variants.iter().enumerate().map(|(i, v)| {
                let i = i as u8;
                let field_name = &v.ident;
                match v.fields {
                    Fields::Unnamed(ref fields) => {
                        let datatype = fields
                            .unnamed
                            .iter()
                            .map(|f| {

                            match &f.ty {
                                Type::Path(x) => {
                                    let datatype = crate::utils::get_datatype_enum(x);
                                    quote! {
                                        Self::#field_name(#datatype::read(buf).await?)
                                    }
                                },
                                _ => {
                                    crate::utils::error(
                                        f.ty.span(),
                                        "The given type is not supported.".into()
                                    )
                                }
                            }
                        });
                        quote! {
                            #i => #(#datatype)*
                        }
                    },
                    Fields::Unit => {
                        quote! {
                            #i => {
                                cachem::EmptyMsg::read(buf).await?;
                                Self::#field_name
                            }
                        }
                    }
                    _ => {
                        crate::utils::error(
                            v.fields.span(),
                            "Only unnamed and unit fields are supported.".into()
                        )
                    }
                }
            });

            quote! {
                let index = u8::read(buf).await?;
                let ret = match index {
                    #(#fields),*,
                    _ => panic!("Invalid enum field")
                };
                Ok(ret)
            }
        },
        _ => {
            crate::utils::error(
                struct_name.span(),
                "Only structs are supported.".into()
            )
        }
    }
}

/// Generates the code for writing the struct to bytes
///
/// # Params
///
/// * `struct_name` - Name of the struct
/// * `data`        - Information about the struct
///
/// # Returns
///
/// [proc_macro2::TokenStream] with the implementation of the write function.
///
fn generate_fn_write(struct_name: &Ident, data: &Data) -> TokenStream {
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
                            Type::Path(_) => {
                                quote! {
                                    self.#field_name.write(buf).await?;
                                }
                            },
                            _ => {
                                crate::utils::error(
                                    f.ty.span(),
                                    "The given type is not supported.".into()
                                )
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
                            Type::Path(_) => {
                                quote! {
                                    self.0.write(buf).await?;
                                }
                            },
                            _ => {
                                crate::utils::error(
                                    f.ty.span(),
                                    "The given type is not supported.".into()
                                )
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
        Data::Enum(ref data) => {
            let fields = data.variants.iter().enumerate().map(|(i, v)| {
                let i = i as u8;
                let field_name = &v.ident;
                match v.fields {
                    Fields::Unnamed(ref fields) => {
                        let datatype = fields
                            .unnamed
                            .iter()
                            .map(|f| {

                            match &f.ty {
                                Type::Path(_) => {
                                    quote! {
                                        Self::#field_name(x) => {
                                            #i.write(buf).await?;
                                            x.write(buf).await?;
                                        }
                                    }
                                },
                                _ => {
                                    crate::utils::error(
                                        f.ty.span(),
                                        "The given type is not supported.".into()
                                    )
                                }
                            }
                        });
                        quote! {
                            #(#datatype)*
                        }
                    },
                    Fields::Unit => {
                        quote! {
                            Self::#field_name => {
                                #i.write(buf).await?;
                                cachem::EmptyMsg::default().write(buf).await?;
                            }
                        }
                    }
                    _ => {
                        crate::utils::error(
                            v.fields.span(),
                            "Only unnamed fields are supported.".into()
                        )
                    }
                }
            });

            quote! {
                match self {
                    #(#fields),*
                };
            }
        },
        _ => {
            struct_name.span().unwrap().error("Only structs and enums are supported.").emit();
            TokenStream::new()
        }
    }
}

