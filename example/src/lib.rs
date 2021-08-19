use async_trait::*;
use cachem::ConnectionGuard;
use cachem::{Command, Get2, Key, Set, Cache};
use cachem::{Index, Parse};
use std::collections::HashMap;
use tokio::net::TcpStream;
use tokio::sync::RwLock;
use tokio::{io::BufStream, sync::watch::Receiver};

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum CacheName {
    A,
    B,
    CommandAndControl,
    Invalid,
}

impl From<u8> for CacheName {
    fn from(x: u8) -> Self {
        match x {
            0   => Self::A,
            1   => Self::B,
            255 => Self::CommandAndControl,
            _   => Self::Invalid,
        }
    }
}

impl Into<u8> for CacheName {
    fn into(self) -> u8 {
        match self {
            Self::A => 0,
            Self::B => 1,
            Self::CommandAndControl => 255,
            Self::Invalid => 255
        }
    }
}

pub struct ExampleDbCli;

#[derive(Clone, Copy, Debug, Parse)]
pub struct ACacheVal {
    field_a: u32,
    field_b: u64,
}

pub struct ACache {
    cache: RwLock<HashMap<u32, ACacheVal>>,
    cnc:   Receiver<Command>,

    index: RwLock<HashMap<u64, u32>>,
}

impl ACache {
    pub fn new(cnc: Receiver<Command>) -> Self {
        let mut map = HashMap::new();
        map.insert(0, ACacheVal { field_a: 1, field_b: 2 });
        map.insert(1, ACacheVal { field_a: 2, field_b: 3 });
        map.insert(2, ACacheVal { field_a: 3, field_b: 4 });

        Self {
            cache: RwLock::new(map),
            cnc,

            index: RwLock::new(HashMap::new())
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
                let res = <ACache as Get2<u32, ACacheVal>>::get(&self, val).await;
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

            let a = <ACache as Get2<u32, ACacheVal>>::get(&self, 1).await;
            dbg!(a);
        }
    }
}

#[async_trait]
impl Get2<u32, ACacheVal> for ACache {
    async fn get(&self, idx: u32) -> Option<ACacheVal> {
        self
            .cache
            .read()
            .await
            .get(&idx)
            .cloned()
    }
}

#[async_trait]
impl Get2<u64, ACacheVal> for ACache {
    async fn get(&self, idx: u64) -> Option<ACacheVal> {
        self
            .cache
            .read()
            .await
            .get(&(idx as u32))
            .cloned()
    }
}

#[async_trait]
impl Index<u32, u64> for ACache {

    async fn get(&self, sid: u64) -> Option<u32> {
        self
            .index
            .read()
            .await
            .get(&sid)
            .cloned()
    }

    async fn index_set(&self, pid: u32, sid: u64) {
        self
            .index
            .write()
            .await
            .insert(sid, pid);
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

impl ExampleDbCli {
    pub async fn get_by_u32(&self, con: &mut ConnectionGuard, id: u32) -> Option<ACacheVal> {
        let _ = con
            .get::<_, u32, ACacheVal>(CacheName::A, id)
            .await;

        None
    }

    pub async fn get_by_u64(&self, con: &mut ConnectionGuard, id: u64) -> Option<ACacheVal> {
        let _ = con
            .get::<_, u64, ACacheVal>(CacheName::A, id)
            .await;

        None
    }
}

