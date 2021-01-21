use std::collections::HashSet;

use cachem_utils::*;
use tokio::net::TcpStream;

#[derive(Default, ProtocolParse)]
struct Demo {
    val_u32: u32,
    val_u64: u64,
    val_u128: u128,
    val_f32: f32,
    val_f64: f64,
    val_string: String,
    val_bool: bool,
}

#[derive(Default, ProtocolParse)]
struct Demo2(Vec<Demo>);

#[derive(Default, ProtocolParse)]
struct Demo3(pub u32);

#[derive(Default, ProtocolParse)]
struct Demo4;

#[derive(ProtocolParse)]
struct Demo5(HashSet<u32>);

#[derive(ProtocolParse)]
struct Demo6 {
    not_a_vec: u32,
    my_vec: Vec<Demo>
}

#[tokio::main]
async fn main() {
    let demo = Demo {
        val_u32: 0u32,
        val_u64: 0u64,
        val_u128: 0u128,
        val_f32: 0f32,
        val_f64: 0f64,
        val_string: String::new(),
        val_bool: false,
    };

    let mut stream = TcpStream::connect("localhost:9999").await.unwrap();
    demo.write(&mut stream).await.unwrap();
}
