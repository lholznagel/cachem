use std::ops::DerefMut;
use std::{collections::VecDeque, ops::Deref};
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicUsize, Ordering};
use metrix_exporter::MetrixSender;
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufStream};
use tokio::net::TcpStream;
use tokio::time::{Duration, sleep};

use crate::{CachemError, ConnectionPoolError};

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
    /// Keeps track of all broken connections
    broken: Arc<AtomicUsize>,
    /// All connections that are currently available
    connections: Arc<Mutex<VecDeque<Connection>>>,
    /// Url to the database
    url: &'static str,
    /// Connection to metrix server
    metrix: MetrixSender,
}

impl ConnectionPool {
    const ACQUIRE_TIMEOUT: u64 = 5u64;

    const METRIC_BROKEN_CONNECTIONS: &'static str = "connection_pool::broken_connections";
    const METRIC_AVAILABLE_CONNECTIONS: &'static str = "connection_pool::available_connections";

    /// Creates a new pool. The given number is the number of connections the
    /// pool will hold. The returned pool is already filled with connections
    /// and can be used.
    pub async fn new(url: &'static str, metrix: MetrixSender, count: usize) -> Result<Self, CachemError> {
        let pool = Self {
            available: Arc::new(AtomicUsize::new(count)),
            pool_size: Arc::new(AtomicUsize::new(count)),
            broken: Arc::new(AtomicUsize::new(0)),
            connections: Arc::new(Mutex::new(VecDeque::new())),
            url,
            metrix,
        };

        let mut connections = VecDeque::new();
        for _ in 0..count {
            connections.push_back(pool.connect().await?)
        }
        pool.connections.lock().unwrap().extend(connections);

        pool.health();

        Ok(pool)
    }

    pub fn health(&self) {
        let self_copy = self.clone();
        let metrix_copy = self.metrix.clone();
        let connection_copy = self.connections.clone();
        tokio::task::spawn(async move {
            loop {
                let mut latest_failed = false;
                let connection = { connection_copy.lock().unwrap().pop_back() };

                if let Some(mut c) = connection {
                    self_copy.available.fetch_sub(1, Ordering::SeqCst);

                    if !c.ping().await {
                        log::warn!("Broken connection");
                        // Try to create a new connection
                        if let Ok(c) = self_copy.connect().await {
                            log::info!("New connection.");
                            // Add the new connection to the pool
                            { connection_copy.lock().unwrap().push_front(c) };
                            let available_conn = self_copy.available.fetch_add(1, Ordering::SeqCst);
                            metrix_copy.send(Self::METRIC_AVAILABLE_CONNECTIONS, available_conn as u128 + 1).await;
                        } else {
                            // Keep track of the broken connections
                            let broken_conn = self_copy.broken.fetch_add(1, Ordering::SeqCst);
                            latest_failed = true;
                            metrix_copy.send(Self::METRIC_BROKEN_CONNECTIONS, broken_conn as u128 + 1).await;
                            log::error!("Error trying to connect");
                        }
                    } else {
                        // All good, readadd the connection
                        { connection_copy.lock().unwrap().push_front(c) };
                        let available_conn = self_copy.available.fetch_add(1, Ordering::SeqCst);
                        metrix_copy.send(Self::METRIC_AVAILABLE_CONNECTIONS, available_conn as u128 + 1).await;
                    }
                } else {
                    log::info!("No connection available.");
                }

                // If the last connection try failed, don´t try again
                if !latest_failed {
                    // Check if there are broken connections and try to connect
                    for _ in 0..self_copy.broken.load(Ordering::SeqCst) {
                        // Ignore if there is an error, just try it again
                        if let Ok(c) = self_copy.connect().await {
                            log::info!("New connection.");
                            { connection_copy.lock().unwrap().push_front(c) };
                            let available_conn = self_copy.available.fetch_add(1, Ordering::SeqCst);
                            let broken_conn = self_copy.broken.fetch_sub(1, Ordering::SeqCst);
                            metrix_copy.send(Self::METRIC_AVAILABLE_CONNECTIONS, available_conn as u128 + 1).await;
                            metrix_copy.send(Self::METRIC_BROKEN_CONNECTIONS, broken_conn as u128 - 1).await;
                        }
                    }
                }

                std::thread::sleep(std::time::Duration::from_millis(500));
            }
        });
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
        let mut connections = VecDeque::new();
        for _ in 0..count {
            connections.push_back(self.connect().await?)
        }
        self.connections.lock().unwrap().extend(connections);

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

    /// Opens a connection and returns it
    async fn connect(&self) -> Result<Connection, CachemError> {
        let stream = TcpStream::connect(&self.url)
            .await
            .map_err(|_| CachemError::ConnectionPoolError(ConnectionPoolError::CannotConnect))?;
        Ok(Connection::new(stream))
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
    pub fn get_mut(&mut self) -> &mut BufStream<TcpStream> {
        &mut self.0
    }

    pub async fn ping(&mut self) -> bool {
        if let Ok(_) = self.0.get_mut().write_u16(u16::MAX).await {
            self.0.flush().await.unwrap();
            if let Ok(_) = self.0.read_u16().await {
                true
            } else {
                false
            }
        } else {
            false
        }
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
