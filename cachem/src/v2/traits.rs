use super::Command;

use crate::Parse;

use async_trait::*;
use tokio::fs::OpenOptions;
use std::collections::HashMap;
use tokio::io::{AsyncWriteExt, BufStream};
use tokio::net::TcpStream;

#[async_trait]
pub trait Cache: Send + Sync {
    fn name(&self) -> String;

    async fn handle(&self, cmd: Command, buf: &mut BufStream<TcpStream>);

    async fn cnc_listener(&self);
}

#[async_trait]
pub trait Get {
    type Idx:   Send;
    type Res:   Parse + Send;
    type Param: Clone + Parse + Send + Sync;

    async fn get(
        &self,
        idx: Self::Idx,
        params: Option<Self::Param>
    ) -> Option<Self::Res>;

    async fn mget(
        &self,
        ids: Vec<Self::Idx>,
        params: Option<Self::Param>
    ) -> Vec<Option<Self::Res>> {
        let mut result = Vec::with_capacity(ids.len());
        for idx in ids {
            result.push(self.get(idx, params.clone()).await);
        }
        result
    }

    async fn exists(&self, idx: Self::Idx) -> bool {
        self.get(idx, None).await.is_some()
    }

    async fn mexists(&self, ids: Vec<Self::Idx>) -> Vec<bool> {
        self
            .mget(ids, None)
            .await
            .iter()
            .map(|x| x.is_some())
            .collect::<Vec<_>>()
    }
}

#[async_trait]
pub trait Key {
    type Idx: Send;

    async fn keys(&self) -> Vec<Self::Idx>;

    async fn count(&self) -> u64 {
        self
            .keys()
            .await
            .len() as u64
    }
}

#[async_trait]
pub trait Set {
    type Idx: Send;
    type Val: Parse + Send;

    async fn set(&self, idx: Self::Idx, val: Self::Val);

    async fn mset(&self, entries: HashMap<Self::Idx, Self::Val>) {
        for (idx, key) in entries {
            self.set(idx, key).await;
        }
    }
}

#[async_trait]
pub trait Del {
    type Idx: Send;

    async fn del(&self, idx: Self::Idx);

    async fn mdel(&self, ids: Vec<Self::Idx>) {
        for id in ids {
            self.del(id).await;
        }
    }
}

#[async_trait]
pub trait Save {
    type Typ: Default + Parse + Send + Sync;

    fn file(&self) -> &str;

    async fn read(&self) -> Self::Typ;

    async fn write(&self, data: Self::Typ);

    async fn save(&self) {
        let file = OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .open(self.file())
            .await;
        if let Ok(file) = file {
            let cache = self.read().await;
            let mut buf = BufStream::new(file);
            let _ = cache.write(&mut buf).await;
            let _ = buf.flush().await;
        }
    }

    async fn load(&self) {
        let file = OpenOptions::new()
            .read(true)
            .open(self.file())
            .await;
        if let Ok(file) = file {
            let mut buf = BufStream::new(file);
            let data = Self::Typ::read(&mut buf).await.unwrap_or_default();
            self.write(data).await;
        }
    }
}

