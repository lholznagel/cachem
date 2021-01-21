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
    collection_ident(&type_path) == Ident::new("Vec", Span::call_site())
}

pub fn is_hash_set(
    type_path: &TypePath
) -> bool {
    collection_ident(&type_path) == Ident::new("HashSet", Span::call_site())
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
                                x
                                    .path
                                    .segments
                                    .first()
                                    .unwrap()
                                    .ident
                                    .clone()
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

fn collection_ident(
    type_path: &TypePath
) -> Ident {
    type_path
        .path
        .segments
        .first()
        .unwrap()
        .ident
        .clone()
}
