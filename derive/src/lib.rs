#![feature(proc_macro_diagnostic)]

mod utils;
mod parse;

use proc_macro2::{Ident, Span};
use quote::quote;
use syn::{AttributeArgs, DeriveInput, NestedMeta, parse_macro_input};
use syn::spanned::Spanned;

#[proc_macro_derive(Parse)]
pub fn derive_parse(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = input.ident;
    let fn_read = parse::generate_fn_read_code(&name, &input.data);
    let fn_write = parse::generate_fn_write_code(&name, &input.data);

    let generics = input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let expanded = quote! {
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
    };

    proc_macro::TokenStream::from(expanded)
}

#[proc_macro_attribute]
pub fn request(args: proc_macro::TokenStream, input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let args = parse_macro_input!(args as AttributeArgs);
    let item: DeriveInput = syn::parse(input).unwrap();

    let name = &item.ident;
    let mut action = Ident::new("Empty", Span::call_site());
    let mut cache = Ident::new("Empty", Span::call_site());

    for arg in args {
        match arg {
            NestedMeta::Meta(x) => {
                let path = x.path();
                let first = &path.segments.first().unwrap().ident;
                if first == &Ident::new("Actions", Span::call_site()) {
                    let b = path.segments.last().unwrap();
                    action = Ident::new(&b.clone().ident.to_string(), Span::call_site());
                } else if first == &Ident::new("Caches", Span::call_site()) {
                    let b = path.segments.last().unwrap();
                    cache = Ident::new(&b.clone().ident.to_string(), Span::call_site());
                }
            }
            _ => {
                item.span()
                    .unstable()
                    .error("Must be meta")
                    .emit();
            }
        }
    }

    proc_macro::TokenStream::from(quote! {
        #item

        #[async_trait::async_trait]
        impl cachem::ProtocolRequest for #name {
            fn action(&self) -> u8 {
                Actions::#action.into()
            }

            fn cache(&self) -> u8 {
                Caches::#cache.into()
            }
        }
    })
}
