use cachem_example::*;

use async_trait::*;
use cachem::Parse;
use cachem::v2::{Command, Get, Key, Server, Set, Cache};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::{io::{AsyncWriteExt, BufStream}, sync::watch::Receiver};
use tokio::net::TcpStream;
use tokio::sync::RwLock;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (cnc_rec, mut server) = Server::new("0.0.0.0:55555".into());

    server.add(CacheName::A, Arc::new(Box::new(ACache::new(cnc_rec))));

    server.listen_cnc();
    server.listen_tcp().await;

    Ok(())
}

#[derive(Clone, Copy, Debug, Parse)]
pub struct ACacheVal {
    field_a: u32,
    field_b: u64,
}

pub struct ACache {
    cache: RwLock<HashMap<u32, ACacheVal>>,
    cnc:   Receiver<Command>,
}

impl ACache {
    pub fn new(cnc: Receiver<Command>) -> Self {
        let mut map = HashMap::new();
        map.insert(0, ACacheVal { field_a: 1, field_b: 2 });
        map.insert(1, ACacheVal { field_a: 2, field_b: 3 });
        map.insert(2, ACacheVal { field_a: 3, field_b: 4 });

        Self {
            cache: RwLock::new(map),
            cnc
        }
    }
}

#[async_trait]
impl Cache for ACache {
    fn name(&self) -> String {
        "ACache".into()
    }

    async fn handle(&self, cmd: Command, buf_socket: &mut BufStream<TcpStream>) {
        match cmd {
            Command::Get => {
                let val = u32::read(buf_socket).await.unwrap();
                print!("id = {:?} -> ", val);
                let res = self.get(val).await;
                println!("res = {:?}", res);

                res.write(buf_socket).await.unwrap();
            },
            Command::Keys => {
                let res = self.keys().await;
                println!("res = {:?}", res);
                for x in res {
                    x.write(buf_socket).await.unwrap();
                }
            },
            Command::Set => {
                let idx = u32::read(buf_socket).await.unwrap();
                let val = ACacheVal::read(buf_socket).await.unwrap();

                self.set(idx, val).await;
                println!("res = OK");
            },
            _ => panic!("Unknown Command")
        };
    }

    async fn cnc_listener(&self) {
        let mut cnc_copy = self.cnc.clone();
        loop {
            cnc_copy.changed().await.unwrap();
            let _val = *cnc_copy.borrow();

            let a = self.get(1).await;
            dbg!(a);
        }
    }
}

#[async_trait]
impl Get for ACache {
    type Idx = u32;
    type Res = ACacheVal;

    async fn get(&self, idx: Self::Idx) -> Option<Self::Res> {
        self
            .cache
            .read()
            .await
            .get(&idx)
            .cloned()
    }
}

#[async_trait]
impl Key for ACache {
    type Idx = u32;

    async fn keys(&self) -> Vec<Self::Idx> {
        self.cache.read().await.keys().map(|x| *x).collect()
    }
}

#[async_trait]
impl Set for ACache {
    type Idx = u32;
    type Val = ACacheVal;

    async fn set(&self, idx: Self::Idx, val: Self::Val) {
        self.cache.write().await.insert(idx, val);
    }
}

