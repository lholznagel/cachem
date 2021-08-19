use crate::{CachemError, Parse};
use super::{Command, ConnectionPool};

use std::convert::AsMut;
use std::collections::HashMap;
use std::hash::Hash;
use std::ops::{Deref, DerefMut};
use tokio::io::{AsyncWriteExt, BufStream};
use tokio::net::TcpStream;

/// Wrapper for an [`tokio::net::TcpStream`] in a [`tokio::io::BufStream`].
/// This is returned when a connection from the [`crate::ConnectionPool`] is requested.
/// Internally the library should use the underlying buffer for reading and
/// writing, but externals only should see the wrapper struct.
pub struct Connection(BufStream<TcpStream>);

impl Connection {
    /// Takes the given [`tokio::net::TcpStream`] and wraps it in a
    /// [`tokio::io::BufStream`] and stores it in the struct.
    pub fn new(stream: TcpStream) -> Self {
        Self(BufStream::new(stream))
    }

    /// Checkes if the connection is still healthy
    ///
    /// # Returns
    ///
    /// * `false` -> Connection is broken and should not be used
    /// * `true`  -> Connection is healthy and can be used
    pub async fn is_healthy(&mut self) -> bool {
        matches!(self.ping().await, Ok(true))
    }

    /// Sends a PING command to the server
    ///
    /// # Returns
    ///
    /// * `true` -> The server replied with PONG
    /// * `false` -> The server did not reply
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use cachem::*;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// // creates a new pool with one connection
    /// let pool = ConnectionPool::new("127.0.0.1:1337".into(), 1usize).await?;
    /// // get a connection
    /// let mut conn = pool.acquire().await?;
    /// conn.ping().await?;
    ///
    /// # Ok(())
    /// # }
    /// ```
    pub async fn ping(&mut self) -> Result<bool, CachemError> {
        self.0.get_mut().write_u8(Command::Ping.into()).await?;
        self.0.flush().await?;

        if u8::read(&mut self.0).await.is_ok() {
            Ok(true)
        } else {
            log::error!("Connection not healthy");
            Ok(false)
        }
    }

    pub async fn save(&mut self) -> Result<(), CachemError> {
        unimplemented!()
    }

    /// Sends a GET command to the server
    ///
    /// # Params
    ///
    /// * `cache` -> Target cache for the command
    /// * `idx`   -> Id of the entry to get
    ///
    /// # Returns
    ///
    /// * `Some(R)` -> The requested id exists in that cache
    /// * `None`    -> The requested id does not exist
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use cachem::*;
    /// enum CacheName { A }
    /// impl Into<u8> for CacheName {
    ///     fn into(self) -> u8 { 0u8 }
    /// }
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// // creates a new pool with one connection
    /// let pool = ConnectionPool::new("127.0.0.1:1337".into(), 1usize).await?;
    /// // get a connection
    /// let mut conn = pool.acquire().await?;
    /// conn.get::<_, _, u32>(CacheName::A, 0u8).await;
    ///
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get<C, I, R>(&mut self, cache: C, idx: I) -> Result<Option<R>, CachemError>
    where
        C: Into<u8>,
        I: Parse,
        R: Parse + Send + Sync {

        self.0.get_mut().write_u8(Command::Get.into()).await?;
        self.0.get_mut().write_u8(cache.into()).await?;
        idx.write(&mut self.0.get_mut()).await?;
        self.0.flush().await?;

        Ok(Option::<R>::read(&mut self.0).await?)
    }

    /// Sends a MGET command to the server
    ///
    /// # Params
    ///
    /// * `cache` -> Target cache for the command
    /// * `ids`   -> List of ids to get
    ///
    /// # Returns
    ///
    /// * `Vec<R>` -> List of the requested ids, if an id did not exist, that
    ///               entry will be ignored
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use cachem::*;
    /// enum CacheName { A }
    /// impl Into<u8> for CacheName {
    ///     fn into(self) -> u8 { 0u8 }
    /// }
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// // creates a new pool with one connection
    /// let pool = ConnectionPool::new("127.0.0.1:1337".into(), 1usize).await?;
    /// // get a connection
    /// let mut conn = pool.acquire().await?;
    /// conn.mget::<_, _, u32>(CacheName::A, vec![0u32, 1u32, 2u32]).await;
    ///
    /// # Ok(())
    /// # }
    /// ```
    pub async fn mget<C, I, R>(&mut self, cache: C, ids: Vec<I>) -> Result<Vec<Option<R>>, CachemError>
    where
        C: Into<u8>,
        I: Parse + Send + Sync,
        R: Parse + Send + Sync {

        self.0.get_mut().write_u8(Command::MGet.into()).await?;
        self.0.get_mut().write_u8(cache.into()).await?;
        ids.write(&mut self.0.get_mut()).await?;
        self.0.flush().await?;

        Ok(Vec::<Option<R>>::read(&mut self.0).await?)
    }

    /// Sends a KEYS command to the server
    ///
    /// # Params
    ///
    /// * `cache` -> Target cache for the command
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use cachem::*;
    /// enum CacheName { A }
    /// impl Into<u8> for CacheName {
    ///     fn into(self) -> u8 { 0u8 }
    /// }
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// // creates a new pool with one connection
    /// let pool = ConnectionPool::new("127.0.0.1:1337".into(), 1usize).await?;
    /// // get a connection
    /// let mut conn = pool.acquire().await?;
    /// conn.keys::<_, u32>(CacheName::A).await;
    ///
    /// # Ok(())
    /// # }
    /// ```
    pub async fn keys<C, R>(&mut self, cache: C) -> Result<Vec<R>, CachemError>
    where
        C: Into<u8>,
        R: Parse + Send + Sync {

        self.0.get_mut().write_u8(Command::Keys.into()).await?;
        self.0.get_mut().write_u8(cache.into()).await?;
        self.0.flush().await?;

        Ok(Vec::<R>::read(&mut self.0).await?)
    }

    /// Sends a EXISTS command to the server
    ///
    /// # Params
    ///
    /// * `cache` -> Target cache for the command
    /// * `idx`   -> Id of the entry to check
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use cachem::*;
    /// enum CacheName { A }
    /// impl Into<u8> for CacheName {
    ///     fn into(self) -> u8 { 0u8 }
    /// }
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// // creates a new pool with one connection
    /// let pool = ConnectionPool::new("127.0.0.1:1337".into(), 1usize).await?;
    /// // get a connection
    /// let mut conn = pool.acquire().await?;
    /// conn.exists(CacheName::A, 0u32).await;
    ///
    /// # Ok(())
    /// # }
    /// ```
    pub async fn exists<C, I>(&mut self, cache: C, idx: I) -> Result<bool, CachemError>
    where
        C: Into<u8>,
        I: Parse {

        self.0.get_mut().write_u8(Command::Exists.into()).await?;
        self.0.get_mut().write_u8(cache.into()).await?;
        idx.write(&mut self.0.get_mut()).await?;
        self.0.flush().await?;

        Ok(bool::read(&mut self.0).await?)
    }

    /// Sends a MEXISTS command to the server
    ///
    /// # Params
    ///
    /// * `cache` -> Target cache for the command
    /// * `ids`   -> Ids of the entries to check
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use cachem::*;
    /// enum CacheName { A }
    /// impl Into<u8> for CacheName {
    ///     fn into(self) -> u8 { 0u8 }
    /// }
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// // creates a new pool with one connection
    /// let pool = ConnectionPool::new("127.0.0.1:1337".into(), 1usize).await?;
    /// // get a connection
    /// let mut conn = pool.acquire().await?;
    /// conn.mexists(CacheName::A, vec![0u32, 1u32]).await;
    ///
    /// # Ok(())
    /// # }
    /// ```
    pub async fn mexists<C, I>(&mut self, cache: C, ids: Vec<I>) -> Result<Vec<bool>, CachemError>
    where
        C: Into<u8>,
        I: Parse + Send + Sync {

        self.0.get_mut().write_u8(Command::MExists.into()).await?;
        self.0.get_mut().write_u8(cache.into()).await?;
        ids.write(&mut self.0.get_mut()).await?;
        self.0.flush().await?;

        Ok(Vec::<bool>::read(&mut self.0).await?)
    }

    /// Sends a SET command to the server
    ///
    /// # Params
    ///
    /// * `cache` -> Target cache for the command
    /// * `id`    -> Id of the new entry
    /// * `data`  -> Date for the entry
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use cachem::*;
    /// enum CacheName { A }
    /// impl Into<u8> for CacheName {
    ///     fn into(self) -> u8 { 0u8 }
    /// }
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// // creates a new pool with one connection
    /// let pool = ConnectionPool::new("127.0.0.1:1337".into(), 1usize).await?;
    /// // get a connection
    /// let mut conn = pool.acquire().await?;
    /// conn.set(CacheName::A, 0u32, 1u32).await;
    ///
    /// # Ok(())
    /// # }
    /// ```
    pub async fn set<C, I, D>(&mut self, cache: C, idx: I, data: D) -> Result<(), CachemError>
    where
        C: Into<u8>,
        I: Parse,
        D: Parse {

        self.0.get_mut().write_u8(Command::Set.into()).await?;
        self.0.get_mut().write_u8(cache.into()).await?;
        idx.write(&mut self.0.get_mut()).await?;
        data.write(&mut self.0.get_mut()).await?;
        self.0.flush().await?;

        u8::read(&mut self.0).await?;
        Ok(())
    }

    /// Sends a MSET command to the server
    ///
    /// # Params
    ///
    /// * `cache` -> Target cache for the command
    /// * `data`  -> Map of entries to insert
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use cachem::*;
    /// # use std::collections::HashMap;
    /// enum CacheName { A }
    /// impl Into<u8> for CacheName {
    ///     fn into(self) -> u8 { 0u8 }
    /// }
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// // creates a new pool with one connection
    /// let pool = ConnectionPool::new("127.0.0.1:1337".into(), 1usize).await?;
    /// // get a connection
    /// let mut conn = pool.acquire().await?;
    ///
    /// let mut data = HashMap::new();
    /// data.insert(0u32, 1u32);
    /// data.insert(1u32, 2u32);
    /// conn.mset(CacheName::A, data).await;
    ///
    /// # Ok(())
    /// # }
    /// ```
    pub async fn mset<C, I, D>(&mut self, cache: C, data: HashMap<I, D>) -> Result<(), CachemError>
    where
        C: Into<u8>,
        I: Parse + Eq + Hash + Send + Sync,
        D: Parse + Send + Sync {

        self.0.get_mut().write_u8(Command::MSet.into()).await?;
        self.0.get_mut().write_u8(cache.into()).await?;
        data.write(&mut self.0.get_mut()).await?;
        self.0.flush().await?;

        u8::read(&mut self.0).await?;
        Ok(())
    }

    /// Sends a DEL command to the server
    ///
    /// # Params
    ///
    /// * `cache` -> Target cache for the command
    /// * `idx`   -> Id to delete
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use cachem::*;
    /// enum CacheName { A }
    /// impl Into<u8> for CacheName {
    ///     fn into(self) -> u8 { 0u8 }
    /// }
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// // creates a new pool with one connection
    /// let pool = ConnectionPool::new("127.0.0.1:1337".into(), 1usize).await?;
    /// // get a connection
    /// let mut conn = pool.acquire().await?;
    /// conn.del(CacheName::A, 0u32).await;
    ///
    /// # Ok(())
    /// # }
    /// ```
    pub async fn del<C, I>(&mut self, cache: C, idx: I) -> Result<(), CachemError>
    where
        C: Into<u8>,
        I: Parse {

        self.0.get_mut().write_u8(Command::Del.into()).await?;
        self.0.get_mut().write_u8(cache.into()).await?;
        idx.write(&mut self.0.get_mut()).await?;
        self.0.flush().await?;

        u8::read(&mut self.0).await?;
        Ok(())
    }

    /// Sends a MDEL command to the server
    ///
    /// # Params
    ///
    /// * `cache` -> Target cache for the command
    /// * `ids`   -> Ids to delete
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use cachem::*;
    /// enum CacheName { A }
    /// impl Into<u8> for CacheName {
    ///     fn into(self) -> u8 { 0u8 }
    /// }
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// // creates a new pool with one connection
    /// let pool = ConnectionPool::new("127.0.0.1:1337".into(), 1usize).await?;
    /// // get a connection
    /// let mut conn = pool.acquire().await?;
    /// conn.mdel(CacheName::A, vec![0u32, 1u32]).await;
    ///
    /// # Ok(())
    /// # }
    /// ```
    pub async fn mdel<C, I>(&mut self, cache: C, ids: Vec<I>) -> Result<(), CachemError>
    where
        C: Into<u8>,
        I: Parse + Send + Sync {

        self.0.get_mut().write_u8(Command::MDel.into()).await?;
        self.0.get_mut().write_u8(cache.into()).await?;
        ids.write(&mut self.0.get_mut()).await?;
        self.0.flush().await?;

        u8::read(&mut self.0).await?;
        Ok(())
    }
}

impl AsMut<BufStream<TcpStream>> for Connection {
    fn as_mut(&mut self) -> &mut BufStream<TcpStream> {
        &mut self.0
    }
}

/// This guard wrapps a connection from the pool.
///
/// When the guard is dropped, the connection is returned to the connectiton
/// pool and can be used for further usage
pub struct ConnectionGuard {
    pool:       ConnectionPool,
    connection: Option<Connection>,
}

impl ConnectionGuard {
    pub fn new(pool: ConnectionPool, con: Connection) -> Self {
        Self {
            pool,
            connection: Some(con),
        }
    }
}

impl Drop for ConnectionGuard {
    fn drop(&mut self) {
        self.pool.release(self.connection.take().unwrap());
    }
}

impl Deref for ConnectionGuard {
    type Target = Connection;

    fn deref(&self) -> &Self::Target {
        self.connection.as_ref().unwrap()
    }
}

impl DerefMut for ConnectionGuard {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.connection.as_mut().unwrap()
    }
}

