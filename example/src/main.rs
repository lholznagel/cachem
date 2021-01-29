mod sample_structs;

use cachem_utils::*;
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

#[derive(Debug, Parse)]
pub struct FetchSampleEntryById(pub u32);

#[derive(Default)]
pub struct SampleCache(RwLock<HashMap<u32, SampleEntry>>);

impl SampleCache {
    pub async fn fetch_by_id(&self, id: u32) -> Option<SampleEntry> {
        if let Some(x) = self.0.read().await.get(&id) {
            Some(x.clone())
        } else {
            None
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
    impl From<u8> for Actions {
        fn from(x: u8) -> Self {
            match x {
                0 => Actions::Fetch,
                _ => panic!("Invalid action")
            }
        }
    }

    #[derive(Debug)]
    enum Caches {
        Sample
    }
    impl From<u8> for Caches {
        fn from(x: u8) -> Self {
            match x {
                0 => Caches::Sample,
                _ => panic!("Invalid action")
            }
        }
    }

    cachem! {
        "0.0.0.0:9999",

        let sample_copy = sample_cache.clone();

        (Actions::Fetch, Caches::Sample) => (FetchId, FetchSampleEntryById, sample_copy),
    }
}
