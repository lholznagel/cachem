use proc_macro2::Span;
use syn::{GenericArgument, Ident, PathArguments, Type, TypePath};

pub fn is_collection(
    type_path: &TypePath
) -> bool {
    is_vec(&type_path) || is_hash_set(&type_path)
}

pub fn is_vec(
    type_path: &TypePath
) -> bool {
    get_ident(&type_path) == Ident::new("Vec", Span::call_site())
}

pub fn is_hash_set(
    type_path: &TypePath
) -> bool {
    get_ident(&type_path) == Ident::new("HashSet", Span::call_site())
}

pub fn ident_from_type(
    type_: &Type
) -> Ident {
    match type_ {
        Type::Path(x) => {
            x
                .path
                .segments
                .first()
                .unwrap()
                .ident
                .clone()
        },
        _ => panic!("Invalid datatype, {:?}", &type_)
    }
}

pub fn get_collection_datatype(
    type_path: &TypePath
) -> Ident {
    match &type_path
        .path
        .segments
        .first()
        .unwrap()
        .arguments {
            PathArguments::AngleBracketed(ref path_arg) => {
                match path_arg
                    .args
                    .first()
                    .unwrap() {

                    GenericArgument::Type(x) => {
                        match x {
                            Type::Path(x) => {
                                get_ident(x)
                            },
                            _ => panic!("Invalid type")
                        }
                    },
                    _ => panic!("Invalid generic argument")
                }
            },
            _ => panic!("Invalid path argument")
        }
}

fn get_ident(
    type_path: &TypePath
) -> Ident {
    if let Some(x) = type_path.path.get_ident() {
        x.clone()
    }  else {
        type_path
            .path
            .segments
            .first()
            .unwrap()
            .ident
            .clone()
    }
}
