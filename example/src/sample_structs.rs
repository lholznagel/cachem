use cachem::*;
use std::collections::HashSet;

#[derive(Default, Parse)]
struct Example1;

#[derive(Default, Parse)]
struct Example2(pub u32);

#[derive(Parse)]
struct Example3(pub Example2);

#[derive(Default, Parse)]
struct Example4(pub Vec<Example2>);

#[derive(Default, Parse)]
struct Example5(pub HashSet<u32>);

#[derive(Parse)]
struct Example6(pub Example8);

#[derive(Parse)]
struct Example7(pub Vec<Example8>);

#[derive(Default, Parse)]
struct Example8 {
    val_u32: u32,
    val_u64: u64,
    val_u128: u128,
    val_f32: f32,
    val_f64: f64,
    val_string: String,
    val_bool: bool,
}

#[derive(Parse)]
struct Example9 {
    my_vec: Vec<u16>,
}

#[derive(Parse)]
struct Example10 {
    val1: u32,
    val2: Vec<u32>,
    val3: Example2,
}

#[request(Actions::A)]
#[derive(Parse)]
struct Example11(pub u32);

#[derive(Action)]
enum Actions {
    A,
    B,
    Invalid,
}
