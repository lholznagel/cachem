use crate::{CachemError, ConnectionPoolError};

use std::collections::VecDeque;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use tokio::net::TcpStream;
use tokio::time::{Duration, sleep};

use super::{Connection, ConnectionGuard};

/// Manages connections to the database.
///
/// # Acquire and release a connection
/// To request a new connection use [`ConnectionPool::acquire()`]. After all
/// operations on that connection are done, return it using
/// [`ConnectionPool::release()`]
///
/// ## Example:
/// ```no_run
/// # use cachem::v2::*;
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
/// # use cachem::v2::*;
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
    available: Arc<AtomicUsize>,
    dead_con:  Arc<AtomicUsize>,
    pool_size: Arc<AtomicUsize>,

    connections: Arc<Mutex<VecDeque<Connection>>>,
    url:         &'static str,
}

impl ConnectionPool {
    const ACQUIRE_TIMEOUT_MSEC:   u64 = 1000u64;
    const CHECK_CONNECTIONS_MSEC: u64 = 1000u64;

    /// Creates a new pool. The given number is the number of connections the
    /// pool will hold. The returned pool is already filled with connections
    /// and can be used.
    pub async fn new(url: &'static str, count: usize) -> Result<Self, CachemError> {
        let pool = Self {
            available: Arc::new(AtomicUsize::new(count)),
            dead_con:  Arc::new(AtomicUsize::new(0)),
            pool_size: Arc::new(AtomicUsize::new(count)),

            connections: Arc::new(Mutex::new(VecDeque::new())),
            url,
        };

        let mut connections = VecDeque::new();
        for _ in 0..count {
            connections.push_back(pool.connect().await?)
        }
        pool.connections.lock().unwrap().extend(connections);

        pool.reconnect_task();

        Ok(pool)
    }

    /// Returns the currently available connections in the pool
    pub fn available_connections(&self) -> usize {
        self.available.load(Ordering::SeqCst)
    }

    /// This function will try to acquire a connection within 5 seconds.
    /// If no connection could be acquired in this time an error returned.
    pub async fn acquire(&self) -> Result<ConnectionGuard, CachemError> {
        let sleep = sleep(Duration::from_millis(Self::ACQUIRE_TIMEOUT_MSEC));
        tokio::pin!(sleep);

        tokio::select! {
            _ = &mut sleep => {
                Err(CachemError::ConnectionPoolError(ConnectionPoolError::TimeoutGettingConnection))
            }
            c = self.try_acquire() => {
                c
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

        // Required, removing this will cause some problems regarding Send and await
        let con = { self.connections.lock().unwrap() }.pop_front();
        if let Some(mut con) = con {
            if con.is_healthy().await {
                self.available.fetch_sub(1, Ordering::SeqCst);
                Ok(ConnectionGuard::new(self.clone(), con))
            } else {
                if let Ok(con) = self.connect().await {
                    self.available.fetch_sub(1, Ordering::SeqCst);
                    Ok(ConnectionGuard::new(self.clone(), con))
                } else {
                    self.available.fetch_sub(1, Ordering::SeqCst);
                    self.dead_con.fetch_add(1, Ordering::SeqCst);
                    Err(CachemError::ConnectionPoolError(ConnectionPoolError::CannotConnect))
                }
            }
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
        let mut cons = self.connections.lock().unwrap();
        for _ in 0..count {
            std::mem::drop(cons.pop_front());
        }
        Ok(())
    }

    /// Starts a task trying to reconnect dead connections
    fn reconnect_task(&self) {
        let self_copy = self.clone();
        let connections_copy = self.connections.clone();

        tokio::task::spawn(async move {
            loop {
                let dead = self_copy.dead_con.load(Ordering::SeqCst);
                if dead > 0 {
                    for _ in 0..dead {
                        if let Ok(con) = self_copy.connect().await {
                            let mut cons = connections_copy.lock().unwrap();
                            cons.push_back(con);

                            self_copy.dead_con.fetch_sub(1, Ordering::SeqCst);
                            self_copy.available.fetch_add(1, Ordering::SeqCst);
                        }
                    }
                }
                std::thread::sleep(std::time::Duration::from_millis(Self::CHECK_CONNECTIONS_MSEC));
            }
        });
    }
}

