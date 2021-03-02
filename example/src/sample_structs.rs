use cachem::*;
use std::collections::HashSet;

#[derive(Default, Parse)]
struct Example1;

#[derive(Debug, Default, Parse, PartialEq, Eq)]
struct Example2(pub u32);

#[derive(Debug, Default, Parse, PartialEq, Eq)]
struct Example3(pub Example2);

#[derive(Debug, Default, Parse, PartialEq, Eq)]
struct Example4(pub Vec<u32>);

#[derive(Debug, Default, Parse, PartialEq, Eq)]
struct Example5(pub Vec<Example2>);

#[derive(Debug, Default, Parse, PartialEq, Eq)]
struct Example6(pub HashSet<u32>);

#[derive(Debug, Default, Parse, PartialEq)]
struct Example8 {
    val_u32: u32,
    val_u64: u64,
    val_u128: u128,
    val_f32: f32,
    val_f64: f64,
    val_string: String,
    val_bool: bool,
}

#[derive(Debug, Default, Parse, PartialEq, Eq)]
struct Example9 {
    my_vec: Vec<u16>,
}

#[derive(Debug, Default, Parse, PartialEq, Eq)]
struct Example10 {
    val1: u32,
    val2: Vec<u32>,
    val3: Example2,
}

#[derive(Debug, Parse, PartialEq, Eq)]
enum Example11 {
    Ok(Example2),
    Vec(Vec<u32>),
    Empty,
}

#[request(Actions::A)]
#[derive(Default, Parse, PartialEq, Eq)]
struct Example12(pub u32);

#[derive(Action)]
enum Actions {
    A,
    B,
    Invalid,
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::io::Cursor;
    use tokio::io::{AsyncWriteExt, BufStream};

    #[tokio::test]
    async fn test_example_1_w() {
        let mut buf = BufStream::new(Cursor::new(Vec::new()));
        let e = Example1::default();
        e.write(&mut buf).await.unwrap();
        buf.flush().await.unwrap();

        let is = buf.into_inner().into_inner();
        let expected = vec![0];
        assert_eq!(is, expected);
    }

    #[tokio::test]
    async fn test_example_2_r() {
        let mut buf = BufStream::new(Cursor::new(vec![1, 1, 0, 0]));
        let is = Example2::read(&mut buf).await.unwrap();

        let expected = Example2(16842752u32);
        assert_eq!(is, expected);
    }

    #[tokio::test]
    async fn test_example_2_w() {
        let mut buf = BufStream::new(Cursor::new(Vec::new()));
        let e = Example2(16842752u32);
        e.write(&mut buf).await.unwrap();
        buf.flush().await.unwrap();

        let is = buf.into_inner().into_inner();
        let expected = vec![1, 1, 0, 0];
        assert_eq!(is, expected);
    }

    #[tokio::test]
    async fn test_example_11_r() {
        let mut buf = BufStream::new(Cursor::new(vec![0, 0, 0, 4, 210]));
        let is = Example11::read(&mut buf).await.unwrap();

        let expected = Example11::Ok(Example2(1234u32));
        assert_eq!(is, expected);
    }

    #[tokio::test]
    async fn test_example_11_w() {
        let mut buf = BufStream::new(Cursor::new(Vec::new()));
        let e = Example11::Ok(Example2(1234u32));
        e.write(&mut buf).await.unwrap();
        buf.flush().await.unwrap();

        let is = buf.into_inner().into_inner();
        let expected = vec![0, 0, 0, 4, 210];
        assert_eq!(is, expected);
    }
}
