mod utils;

use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{Data, DeriveInput, Fields, Ident, Type, parse_macro_input};

#[proc_macro_derive(ProtocolParse)]
pub fn derive_heap_size(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = input.ident;
    let fn_read = generate_fn_read_code(&name, &input.data);
    let fn_write = generate_fn_write_code(&input.data);

    let generics = input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let expanded = quote! {
        #[async_trait::async_trait]
        impl #impl_generics cachem_utils::ProtocolParse for #name #ty_generics #where_clause {
            async fn read<B>(
                buf: &mut B
            ) -> Result<Self, cachem_utils::CachemError>
            where
                B: tokio::io::AsyncBufRead + tokio::io::AsyncRead + Send + Unpin {

                #fn_read
            }

            async fn write<B>(
                &self,
                buf: &mut B
            ) -> Result<(), cachem_utils::CachemError>
            where
                B: tokio::io::AsyncWrite + Send + Unpin {

                #fn_write
                Ok(())
            }
        }
    };

    proc_macro::TokenStream::from(expanded)
}

fn generate_fn_read_code(struct_name: &Ident, data: &Data) -> TokenStream {
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
                                if utils::is_collection(&x) {
                                    let datatype = utils::get_collection_datatype(&x);

                                    if utils::is_vec(&x) {
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
                                        panic!("Not a Vec")
                                    }
                                } else {
                                    let datatype = utils::ident_from_type(&f.ty);
                                    quote! {
                                        #field_name: #datatype::read(buf).await?,
                                    }
                                }
                            },
                            _ => panic!("[Read] Invalid datatype, {:?}", &f.ty)
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
                                if utils::is_collection(&x) {
                                    let datatype = utils::get_collection_datatype(&x);

                                    if utils::is_vec(&x) {
                                        quote! {
                                            let entry_count = u32::read(buf).await?;
                                            let mut entries = Vec::new();
                    
                                            for _ in 0..entry_count {
                                                let x = #datatype::read(buf).await?.into();
                                                entries.push(x);
                                            }
                    
                                            Ok(Self(entries))
                                        }
                                    } else if utils::is_hash_set(&x) {
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
                                        panic!("Not a Vec or HashMap")
                                    }
                                } else {
                                    let datatype = x.path.segments.first().unwrap().ident.clone();
                                    quote! {
                                        Ok(Self(#datatype::read(buf).await?))
                                    }
                                }
                            },
                            _ => panic!("[Read] Invalid datatype, {:?}", &f.ty)
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
        _ => panic!("[Read] Invalid data, {:?}", data)
    }
}

fn generate_fn_write_code(data: &Data) -> TokenStream {
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
                            _ => panic!("[Read] Invalid datatype, {:?}", &f.ty)
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
                            _ => panic!("[Read] Invalid datatype, {:?}", &f.ty)
                        }
                    });
                    quote! {
                        #(#datatype)*
                    }
                },
                Fields::Unit => quote! { }
            }
        },
        _ => panic!("[Write] Invalid data, {:?}", data)
    }
}
