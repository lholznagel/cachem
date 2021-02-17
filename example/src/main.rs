mod sample_structs;

use async_trait::async_trait;
use cachem::*;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone, Debug, Parse)]
pub struct SampleEntry {
    pub id:    u32,
    pub val_1: u32,
    pub val_2: bool,
    pub val_3: u64,
}

#[derive(Default, Parse)]
pub struct EmptyEntry;

#[derive(Debug, Parse)]
pub struct FetchSampleEntryById(pub u32);

#[derive(Default)]
pub struct SampleCache(RwLock<HashMap<u32, SampleEntry>>);

#[async_trait]
impl Fetch<FetchSampleEntryById> for SampleCache {
    type Error    = EmptyEntry;
    type Response = SampleEntry;

    async fn fetch(&self, input: FetchSampleEntryById) -> Result<Self::Response, Self::Error> {
        if let Some(x) = self.0.read().await.get(&input.0) {
            Ok(x.clone())
        } else {
            Err(EmptyEntry::default())
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let sample_cache = Arc::new(SampleCache::default());

    #[derive(Debug)]
    enum Actions {
        Fetch
    }
    impl From<u16> for Actions {
        fn from(x: u16) -> Self {
            match x {
                0 => Actions::Fetch,
                _ => panic!("Invalid action")
            }
        }
    }

    cachem! {
        "0.0.0.0:9999",

        let sample_copy = sample_cache.clone();

        - Actions::Fetch => (sample_copy, fetch, FetchSampleEntryById),
    }
}
