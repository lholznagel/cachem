#![feature(proc_macro_diagnostic)]

mod utils;
mod parse;

use quote::quote;
use syn::{DeriveInput, parse_macro_input};

#[proc_macro_derive(Parse)]
pub fn derive_heap_size(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = input.ident;
    let fn_read = parse::generate_fn_read_code(&name, &input.data);
    let fn_write = parse::generate_fn_write_code(&name, &input.data);

    let generics = input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let expanded = quote! {
        #[async_trait::async_trait]
        impl #impl_generics cachem_utils::Parse for #name #ty_generics #where_clause {
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
