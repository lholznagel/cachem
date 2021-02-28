use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{Data, Ident};

pub fn generate_fn_from(struct_name: &Ident, data: &Data) -> TokenStream {
    let mut entries = Vec::new();
    if let Data::Enum(x) = data {
        for (i, v) in x.variants.iter().enumerate() {
            let i = i as u16;
            let ident = &v.ident;

            if ident != &Ident::new("Invalid", Span::call_site()) {
                entries.push(quote! {
                    #i => Self::#ident,
                })
            }
        }
    } else {
        return crate::utils::error(
            struct_name.span(),
            "Only for enums".into()
        );
    }

    quote! {
        match x {
            #(#entries)*
            _ => {
                log::error!("Unknown action {}", x);
                Self::Invalid
            }
        }
    }
}

pub fn generate_fn_into(struct_name: &Ident, data: &Data) -> TokenStream {
    let mut entries = Vec::new();
    if let Data::Enum(x) = data {
        for (i, v) in x.variants.iter().enumerate() {
            let i = i as u16;
            let ident = &v.ident;

            if ident != &Ident::new("Invalid", Span::call_site()) {
                entries.push(quote! {
                    Self::#ident => #i,
                })
            }
        }
    } else {
        return crate::utils::error(
            struct_name.span(),
            "Only for enums".into()
        );
    }

    quote! {
        match self {
            #(#entries)*
            Self::Invalid => u16::MAX,
        }
    }
}
