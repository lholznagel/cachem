use std::ops::DerefMut;
use std::{collections::VecDeque, ops::Deref};
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicUsize, Ordering};
use tokio::io::BufStream;
use tokio::net::TcpStream;
use tokio::time::{Duration, sleep};

use crate::{CachemError, ConnectionPoolError};

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

    /// Gets the [`tokio::io::BufStream`] as a mutable reference
    pub(crate) fn get_mut(&mut self) -> &mut BufStream<TcpStream> {
        &mut self.0
    }
}

/// Manages connections to the database.
///
/// # Acquire and release a connection
/// To request a new connection use [`ConnectionPool::acquire()`]. After all
/// operations on that connection are done, return it using
/// [`ConnectionPool::release()`]
///
/// ## Example:
/// ```no_run
/// # use cachem::*;
/// # #[tokio::main]
/// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
/// // creates a new pool with one connection
/// let pool = ConnectionPool::new("127.0.0.1:1337".into(), 1usize).await?;
/// // get a connection
/// let mut conn = pool.acquire().await?;
///
/// // ... do something with the connection ...
///
/// // the connection is dropped and returned to the pool
/// # Ok(())
/// # }
/// ```
///
/// # Scaling the pool
/// The pool is also able to scale the number of connections up and down.
/// For more information take a look at [`ConnectionPool::scale_up`] and
/// [`ConnectionPool::scale_down`].
///
/// ## Example:
/// ```no_run
/// # use cachem::*;
/// # #[tokio::main]
/// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
/// // creates a new pool with one connection
/// let pool = ConnectionPool::new("127.0.0.1:1337".into(), 2usize).await?;
/// // gets the number of connections that can be acquired
/// // this should be 2
/// let count = pool.available_connections();
/// println!("Available connections: {}", count);
///
/// // scale down, the given number is the number of connections that are dropped
/// pool.scale_down(1).await?;
///
/// // this should now return 1
/// let count = pool.available_connections();
/// println!("Available connections: {}", count);
/// # Ok(())
/// # }
/// ```
#[derive(Clone)]
pub struct ConnectionPool {
    /// Counts the number of acquired connections.
    /// This is mainly for reducing the number of locks on the connections vec
    available: Arc<AtomicUsize>,
    /// Max number of connections the pool can contain
    pool_size: Arc<AtomicUsize>,
    /// All connections that are currently available
    connections: Arc<Mutex<VecDeque<Connection>>>,
    url: &'static str
}

impl ConnectionPool {
    const ACQUIRE_TIMEOUT: u64 = 5u64;

    /// Creates a new pool. The given number is the number of connections the
    /// pool will hold. The returned pool is already filled with connections
    /// and can be used.
    pub async fn new(url: &'static str, count: usize) -> Result<Self, CachemError> {
        let pool = Self {
            available: Arc::new(AtomicUsize::new(count)),
            pool_size: Arc::new(AtomicUsize::new(count)),
            connections: Arc::new(Mutex::new(VecDeque::new())),
            url
        };
        pool.connect(count).await?;
        Ok(pool)
    }

    /// Returns the currently available connections in the pool
    pub fn available_connections(&self) -> usize {
        self.available.load(Ordering::SeqCst)
    }

    /// This function will try to acquire a connection within 5 seconds.
    /// If no connection could be acquired in this time an error returned.
    pub async fn acquire(&self) -> Result<ConnectionGuard, CachemError> {
        let sleep = sleep(Duration::from_secs(Self::ACQUIRE_TIMEOUT));
        tokio::pin!(sleep);

        tokio::select! {
            _ = &mut sleep => {
                return Err(CachemError::ConnectionPoolError(ConnectionPoolError::TimeoutGettingConnection));
            }
            c = self.try_acquire() => {
                return c;
            }
        }
    }

    /// Tries to acquire a connection from the pool, if non is available an
    /// error is returned
    pub async fn try_acquire(&self) -> Result<ConnectionGuard, CachemError> {
        // Before locking the connections mutex, check if there are connections
        // available, if not return an error
        if self.available.load(Ordering::SeqCst) == 0 {
            return Err(CachemError::ConnectionPoolError(ConnectionPoolError::NoConnectionAvailable));
        }

        if let Some(conn) = self.connections.lock().unwrap().pop_front() {
            self.available.fetch_sub(1, Ordering::SeqCst);
            Ok(ConnectionGuard {
                pool: self.clone(),
                connection: Some(conn),
            })
        } else {
            Err(CachemError::ConnectionPoolError(ConnectionPoolError::NoConnectionAvailable))
        }
    }

    /// Adds the given amount of connections to the pool
    pub async fn scale_up(&self, count: usize) -> Result<(), CachemError> {
        self.connect(count).await?;
        // increase pool count and make the new connections available
        self.pool_size.fetch_add(count, Ordering::SeqCst);
        self.available.fetch_add(count, Ordering::SeqCst);
        Ok(())
    }

    /// Removes the given amount of connections from the pool.
    /// Fails if there are not enough connections to scale down or not enough
    /// connections are available
    pub async fn scale_down(&self, count: usize) -> Result<(), CachemError> {
        if self.pool_size.load(Ordering::SeqCst) < count {
            Err(CachemError::ConnectionPoolError(ConnectionPoolError::NotEnoughConnectionsInPool))
        } else if self.available.load(Ordering::SeqCst) < count {
            Err(CachemError::ConnectionPoolError(ConnectionPoolError::NotEnoughConnectionsAvailable))
        } else {
            // dencrease pool count and remove the connections
            self.pool_size.fetch_sub(count, Ordering::SeqCst);
            self.available.fetch_sub(count, Ordering::SeqCst);
            self.drop(count).await?;
            Ok(())
        }
    }

    /// Releases a connection back into the connection pool
    pub(crate) fn release(&self, connection: Connection) {
        self.connections.lock().unwrap().push_back(connection);
        self.available.fetch_add(1, Ordering::SeqCst);
    }

    /// Fills the internal connection pool with the given amount of connections
    async fn connect(&self, count: usize) -> Result<(), CachemError> {
        let mut connections = VecDeque::new();
        for _ in 0..count {
            let stream = TcpStream::connect(&self.url)
                .await
                .map_err(|_| CachemError::ConnectionPoolError(ConnectionPoolError::CannotConnect))?;
            connections.push_back(Connection::new(stream));
        }
        self.connections.lock().unwrap().extend(connections);
        Ok(())
    }

    /// Drops the given amount of connections
    async fn drop(&self, count: usize) -> Result<(), CachemError> {
        let mut connections = self.connections.lock().unwrap();
        for _ in 0..count {
            std::mem::drop(connections.pop_front());
        }
        Ok(())
    }
}

/// This guard wrappes a connection from the pool.
///
/// When the guard is dropped, the connection is returned to the connectiton
/// pool and can be used for further usage
pub struct ConnectionGuard {
    pool: ConnectionPool,
    connection: Option<Connection>,
}

impl Drop for ConnectionGuard {
    fn drop(&mut self) {
        self.pool.release(self.connection.take().unwrap());
    }
}

impl Deref for ConnectionGuard {
    type Target = Connection;

    fn deref(&self) -> &Self::Target {
        &self.connection.as_ref().unwrap()
    }
}

impl DerefMut for ConnectionGuard {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.connection.as_mut().unwrap()
    }
}
