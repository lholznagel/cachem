use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{Data, Fields, Ident, Type};
use syn::spanned::Spanned;

pub fn generate_fn_read_code(struct_name: &Ident, data: &Data) -> TokenStream {
    match *data {
        Data::Struct(ref data) => {
            match data.fields {
                Fields::Named(ref fields) => {
                    let struct_fields = fields
                        .named
                        .iter()
                        .map(|f| {

                        let field_name = &f.ident;
                        match &f.ty {
                            Type::Path(x) => {
                                if crate::utils::is_collection(&x) {
                                    let datatype = crate::utils::get_collection_datatype(&x);

                                    if crate::utils::is_vec(&x) {
                                        quote! {
                                            #field_name: {
                                                let entry_count = u32::read(buf).await?;
                                                let mut entries = Vec::new();
                        
                                                for _ in 0..entry_count {
                                                    let x = #datatype::read(buf).await?;
                                                    entries.push(x);
                                                }
                                                entries
                                            }
                                        }
                                    } else {
                                        x.span().unwrap().error(format!("Currently only the datatype Vec and simple types are supported.")).emit();
                                        return TokenStream::new();
                                    }
                                } else {
                                    let datatype = crate::utils::ident_from_type(&f.ty);
                                    quote! {
                                        #field_name: #datatype::read(buf).await?,
                                    }
                                }
                            },
                            _ => {
                                f.ty.span().unwrap().error(format!("The given type is not supported.")).emit();
                                return TokenStream::new();
                            }
                        }
                    });

                    quote! {
                        Ok(#struct_name {
                            #(#struct_fields)*
                        })
                    }
                },
                Fields::Unnamed(ref fields) => {
                    let datatype = fields
                        .unnamed
                        .iter()
                        .map(|f| {

                        match &f.ty {
                            Type::Path(x) => {
                                if crate::utils::is_collection(&x) {
                                    let datatype = crate::utils::get_collection_datatype(&x);

                                    if crate::utils::is_vec(&x) {
                                        quote! {
                                            let entry_count = u32::read(buf).await?;
                                            let mut entries = Vec::new();

                                            for _ in 0..entry_count {
                                                let x = #datatype::read(buf).await?.into();
                                                entries.push(x);
                                            }

                                            Ok(Self(entries))
                                        }
                                    } else if crate::utils::is_hash_set(&x) {
                                        quote! {
                                            let entry_count = u32::read(buf).await?;
                                            let mut entries = std::collections::HashSet::new();

                                            for _ in 0..entry_count {
                                                let x = #datatype::read(buf).await?.into();
                                                entries.insert(x);
                                            }

                                            Ok(Self(entries))
                                        }
                                    } else {
                                        x.span().unwrap().error(format!("Currently only Vec and HashSet are supported.")).emit();
                                        return TokenStream::new();
                                    }
                                } else {
                                    let datatype = x.path.segments.first().unwrap().ident.clone();
                                    quote! {
                                        Ok(Self(#datatype::read(buf).await?))
                                    }
                                }
                            },
                            _ => {
                                f.ty.span().unwrap().error(format!("The given type is not supported.")).emit();
                                return TokenStream::new();
                            }
                        }
                    });
                    quote! {
                        #(#datatype)*
                    }
                },
                Fields::Unit => quote! {
                    Ok(Self {})
                }
            }
        },
        _ => {
            struct_name.span().unwrap().error(format!("Only sttructs are supported.")).emit();
            return TokenStream::new();
        }
    }
}

pub fn generate_fn_write_code(struct_name: &Ident, data: &Data) -> TokenStream {
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
                                if x.path.segments.first().unwrap().ident == Ident::new("Vec", Span::call_site()) {
                                    quote! {
                                        u32::from(self.#field_name.len() as u32).write(buf).await?;
                                        for entry in self.#field_name.iter() {
                                            entry.write(buf).await?
                                        }
                                    }
                                } else {
                                    quote! {
                                        self.#field_name.write(buf).await?;
                                    }
                                }
                            },
                            _ => {
                                f.ty.span().unwrap().error(format!("The given type is not supported.")).emit();
                                return TokenStream::new();
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
                                if x.path.segments.first().unwrap().ident == Ident::new("Vec", Span::call_site()) {
                                    quote! {
                                        u32::from(self.0.len() as u32).write(buf).await?;
                                        for entry in self.0.iter() {
                                            entry.write(buf).await?
                                        }
            
                                        self.write(buf).await?;
                                    }
                                } else {
                                    quote! {
                                        self.write(buf).await?;
                                    }
                                }
                            },
                            _ => {
                                f.ty.span().unwrap().error(format!("The given type is not supported.")).emit();
                                return TokenStream::new();
                            }
                        }
                    });
                    quote! {
                        #(#datatype)*
                    }
                },
                Fields::Unit => quote! { }
            }
        },
        _ => {
            struct_name.span().unwrap().error(format!("Only sttructs are supported.")).emit();
            return TokenStream::new();
        }
    }
}
