//! Contains all traits that are used across the database

use crate::Command;
use crate::Parse;

use async_trait::*;
use tokio::fs::OpenOptions;
use std::collections::HashMap;
use tokio::io::{AsyncWriteExt, BufStream};
use tokio::net::TcpStream;

/// This trait implements default functions for caches
#[async_trait]
pub trait Cache: Send + Sync {
    /// Name of the cache
    ///
    /// # Returns
    ///
    /// Name of the cache
    ///
    fn name(&self) -> String;

    /// TODO
    async fn handle(&self, cmd: Command, buf: &mut BufStream<TcpStream>);

    /// TODO
    async fn cnc_listener(&self);
}

/// Trait for getting data from the cache.
///
/// # Generics
///
/// * `Id`  - Datatype for the id
/// * `Res` - Datatype of the result, must implement [Parse]
///
/// # Usage
///
/// ```rust
/// use async_trait::async_trait;
/// use cachem::Get2;
/// use std::collections::HashMap;
/// use tokio::sync::RwLock;
///
/// struct Cache {
///     cache: RwLock<HashMap<u32, u32>>,
/// }
///
/// #[async_trait]
/// impl Get2<u32, u32> for Cache {
///     async fn get(
///         &self,
///         id: u32
///     ) -> Option<u32> {
///         self
///             .cache
///             .read()
///             .await
///             .get(&id)
///             .cloned()
///     }
/// }
/// ```
///
/// It is also possible to add multiple [`Get2`] implementations, the only
/// requirement is that at least one generic parameter must be different.
///
/// ```rust
/// use async_trait::async_trait;
/// use cachem::Get2;
/// use std::collections::HashMap;
/// use tokio::sync::RwLock;
///
/// struct Cache {
///     cache: RwLock<HashMap<u32, u32>>
/// }
///
/// #[async_trait]
/// impl Get2<u32, u32> for Cache {
///     async fn get(
///         &self,
///         id: u32
///     ) -> Option<u32> {
///         self
///             .cache
///             .read()
///             .await
///             .get(&id)
///             .cloned()
///     }
/// }
///
/// #[async_trait]
/// impl Get2<u32, f32> for Cache {
///     async fn get(
///         &self,
///         id: u32
///     ) -> Option<f32> {
///         self
///             .cache
///             .read()
///             .await
///             .get(&id)
///             .cloned()
///             .map(|x| { x as f32 / 100f32 })
///     }
/// }
/// ```
///
#[async_trait]
pub trait Get2<Id, Res>
    where
        Id:  Parse + Send + 'static,
        Res: Parse + Send + 'static {

    /// Gets a single item from the cache.
    ///
    /// # Params
    ///
    /// * `ìd` - Id of the entry to get
    ///
    /// # Returns
    ///
    /// If the item does not exist `None`, if the item exist a single item
    /// of the generic datatype `Res`.
    ///
    async fn get(
        &self,
        id: Id,
    ) -> Option<Res>;

    /// Gets multiple items from the cache.
    /// The output will always have the same length as the given number of ids.
    ///
    /// # Params
    ///
    /// * `ids` - List of ids to get from the cache
    ///
    /// # Returns
    ///
    /// Items that don´t exist will be omitted. All other items will be in the
    /// result list.
    ///
    async fn mget(
        &self,
        ids: Vec<Id>,
    ) -> Vec<Option<Res>> {
        let mut result = Vec::new();
        for id in ids {
            result.push(self.get(id).await);
        }
        result
    }
}

/// PId -> Primary Id
/// SId -> Secondary Id
#[async_trait]
pub trait Index<PId, SId>
    where 
        PId: Send + 'static,
        SId: Send + 'static {

    /// Converts the given secondary Id to a primary id
    async fn get(&self, id: SId) -> Option<PId>;

    /// TODO
    async fn index_set(&self, pid: PId, sid: SId);
}

/// Deprecated
#[async_trait]
pub trait Get {
    /// Deprecated
    type Id:    Send;
    /// Deprecated
    type Res:   Parse + Send;
    /// Deprecated
    type Param: Clone + Parse + Send + Sync;

    /// Deprecated
    async fn get(
        &self,
        id: Self::Id,
        params: Option<Self::Param>
    ) -> Option<Self::Res>;

    /// Deprecated
    async fn mget(
        &self,
        ids: Vec<Self::Id>,
        params: Option<Self::Param>
    ) -> Vec<Option<Self::Res>> {
        let mut result = Vec::with_capacity(ids.len());
        for id in ids {
            result.push(self.get(id, params.clone()).await);
        }
        result
    }

    /// Deprecated
    async fn exists(&self, id: Self::Id) -> bool {
        self.get(id, None).await.is_some()
    }

    /// Deprecated
    async fn mexists(&self, ids: Vec<Self::Id>) -> Vec<bool> {
        self
            .mget(ids, None)
            .await
            .iter()
            .map(|x| x.is_some())
            .collect::<Vec<_>>()
    }
}

/// Trait for working with keys
#[async_trait]
pub trait Key {
    /// Type of the id
    type Id: Send;

    /// Gets a list of all keys in the cachem
    ///
    /// # Returns
    ///
    /// Vector of ids
    async fn keys(&self) -> Vec<Self::Id>;

    /// Counts the number of keys in the cachem
    ///
    /// # Returns
    ///
    /// Number of keys
    async fn count(&self) -> u64 {
        self
            .keys()
            .await
            .len() as u64
    }
}

/// Trait for setting values in the cache
#[async_trait]
pub trait Set {
    /// Type of the id
    type Id: Send;
    /// Type of the value
    type Val: Parse + Send;

    /// Sets a value
    ///
    /// # Params
    ///
    /// * `id`  - Id of the new entry
    /// * `val` - Value that should be set
    ///
    async fn set(&self, id: Self::Id, val: Self::Val);

    /// Sets mutliple values
    ///
    /// # Params
    ///
    /// * `entries` - HashMap of entries to set.
    ///               The key of the map is the id of the entry
    ///               The value of the map is the value to set
    ///
    async fn mset(&self, entries: HashMap<Self::Id, Self::Val>) {
        for (id, key) in entries {
            self.set(id, key).await;
        }
    }
}

#[async_trait]
pub trait Set2<Id, Val>
    where
        Id:  Parse + Send + 'static,
        Val: Parse + Send + 'static {

    /// Sets a value
    ///
    /// # Params
    ///
    /// * `id`  - Id of the new entry
    /// * `val` - Value that should be set
    ///
    async fn set(&self, id: Id, val: Val);

    /// Sets mutliple values
    ///
    /// # Params
    ///
    /// * `entries` - HashMap of entries to set.
    ///               The key of the map is the id of the entry
    ///               The value of the map is the value to set
    ///
    async fn mset(&self, entries: HashMap<Id, Val>) {
        for (id, key) in entries {
            self.set(id, key).await;
        }
    }
}

/// Trait for deleting entries from the cache
#[async_trait]
pub trait Del {
    /// Defines the type of the id
    type Id: Send;

    /// Deletes a single entry by id from the cache
    ///
    /// # Params
    ///
    /// * `id` - Id of the entry to delete
    ///
    async fn del(&self, id: Self::Id);

    /// Deletes the given vector of ids from the cache
    ///
    /// # Params
    ///
    /// * `ids` - Vector of ids to delete
    ///
    async fn mdel(&self, ids: Vec<Self::Id>) {
        for id in ids {
            self.del(id).await;
        }
    }
}

/// Trait for reading and writing a struct to a file
#[async_trait]
pub trait Save {
    /// Defines the struct type
    type Typ: Default + Parse + Send + Sync;

    /// Sets the filename to read and write from
    ///
    /// # Returns
    ///
    /// Filename to use for reading and writing the struct
    ///
    fn file(&self) -> &str;

    /// Reads a file and parses it into the given type [Save::Typ].
    ///
    /// # Returns
    ///
    /// Parsed file defined by [Save::Typ]
    ///
    async fn read(&self) -> Self::Typ;

    /// Writes the given datatype [Save::Typ] to a file
    ///
    /// # Params
    ///
    /// * `data` - Struct from the type [Save::Typ]
    ///
    async fn write(&self, data: Self::Typ);

    /// Default implementation for writing the current struct to a file
    ///
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

    /// Default implementation for loading the file and parsing it into the
    /// struct that is defined by [Save::Typ].
    ///
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

