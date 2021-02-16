use cachem::*;

#[derive(Default, Parse)]
struct Demo {
    val_u32: u32,
    val_u64: u64,
    val_u128: u128,
    val_f32: f32,
    val_f64: f64,
    val_string: String,
    val_bool: bool,
}

#[derive(Default, Parse)]
struct Demo2(pub Vec<Demo>);

#[derive(Default, Parse)]
struct Demo3(pub u32);

#[derive(Default, Parse)]
struct Demo4;

#[derive(Parse)]
struct Demo6 {
    not_a_vec: u32,
    my_vec: Vec<u16>,
}

#[request(Actions::A, Caches::B)]
#[derive(Parse)]
struct Demo7(pub u32);

#[derive(Parse)]
struct Demo8(pub Demo7);

enum Actions {
    A
}

impl Into<u8> for Actions {
    fn into(self) -> u8 {
        0u8
    }
}

enum Caches {
    B
}

impl Into<u8> for Caches {
    fn into(self) -> u8 {
        0u8
    }
}
