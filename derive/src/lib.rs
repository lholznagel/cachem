#![feature(proc_macro_diagnostic)]

//! This crate provides useful derives for different traits of the database.

/// Implementation of the [cachem::Get2] trait
mod get;
/// Implementation of the [cachem::Parse] trait
mod parse;
/// General purpose functions
mod utils;

use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, parse_macro_input};

/// Function for deriving the [cachem::Cachem] trait
///
/// # Params
///
/// * `input` - [proc_macro::TokenStream] of the struct the trait should be
///             implemented on
///
/// # Returns
///
/// [proc_macro::TokenStream] that implements the trait cachem::Cachem
///
#[proc_macro_derive(Get, attributes(cachem))]
pub fn derive_cachem(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let code_gen = get::code_gen(input);
    let code_gen = if let Ok(x) = code_gen {
        x
    } else if let Err(x) = code_gen {
        x
    } else {
        quote! {}
    };

    TokenStream::from(code_gen)
}

/// Function for deriving the [cachem::Parse] trait
///
/// # Params
///
/// * `input` - [proc_macro::TokenStream] of the struct the trait should be
///             implemented on
///
/// # Returns
///
/// [proc_macro::TokenStream] that implements the trait [cachem::Parse]
///
#[proc_macro_derive(Parse)]
pub fn derive_parse(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let code_gen = parse::code_gen(input);
    TokenStream::from(code_gen)
}

