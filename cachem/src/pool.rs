use crate::{CachemError, ConnectionPoolError};

use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use tokio::net::TcpStream;
use tokio::time::{Duration, sleep};

use super::{Connection, ConnectionGuard};

/// Manages connections to the database.
///
/// # Acquire and release a connection
/// To request a new connection use [`ConnectionPool::acquire()`].
/// The connection is returned when the variable is dropped.
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
#[derive(Clone)]
pub struct ConnectionPool {
    /// Number of available connections
    available:    Arc<AtomicUsize>,
    /// Size of the pool
    pool_size:    Arc<AtomicUsize>,
    /// When a dead connection is encoutered, this will be set to true
    has_dead_con: Arc<AtomicBool>,

    /// Holds all active connection
    connections: Arc<Mutex<VecDeque<Connection>>>,
    /// IP-Address to the database server
    url:         &'static str,
}

impl ConnectionPool {
    /// Timeout for acquiring a connection from the pool, in milliseconds
    const ACQUIRE_TIMEOUT_MSEC:   u64 = 1000u64;
    /// Interval when the subtask checkes if there are broken connection, in
    /// milliseconds
    const CHECK_CONNECTIONS_MSEC: u64 = 1000u64;

    /// Creates a new pool. The given number is the number of connections the
    /// pool will hold. The returned pool is already filled with connections
    /// and can be used.
    ///
    /// # Params
    ///
    /// * `url`   - Ip address + port of the database server
    /// * `count` - Number of connection to store
    ///
    /// # Returns
    ///
    /// New pool containing the given number of connections
    ///
    pub async fn new(url: &'static str, count: usize) -> Result<Self, CachemError> {
        let pool = Self {
            available:    Arc::new(AtomicUsize::new(count)),
            pool_size:    Arc::new(AtomicUsize::new(count)),
            has_dead_con: Arc::new(AtomicBool::new(false)),

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

    /// # Returns
    ///
    /// The number of currently available connections in the pool
    ///
    pub fn available_connections(&self) -> usize {
        self.available.load(Ordering::SeqCst)
    }

    /// Tries to acquire a connection in the given timeframe set by
    /// Self::ACQUIRE_TIMEOUT_MSEC.
    /// If there was no connection available it returns an error.
    ///
    /// # Returns
    ///
    /// If successful a connection from the pool, if not a
    /// [ConnectionPoolError::TimeoutGettingConnection] error.
    ///
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

    /// Tries to instantly get a connection from the pool.
    ///
    /// # Returns
    ///
    /// An error if there is either a dead connection, there are no connections
    /// in the pool or the healthcheck failed.
    /// If successful if will return a [`ConnectionGuard`].
    ///
    pub async fn try_acquire(&self) -> Result<ConnectionGuard, CachemError> {
        // Make sure that there is no dead connection
        if self.has_dead_con.load(Ordering::SeqCst) {
            log::error!("Dead connection");
            return Err(CachemError::ConnectionPoolError(ConnectionPoolError::NoConnectionAvailable));
        }

        // Before locking the connections mutex, check if there are connections
        // available, if not return an error
        if self.available.load(Ordering::SeqCst) == 0 {
            log::warn!("No connection available");
            return Err(CachemError::ConnectionPoolError(ConnectionPoolError::NoConnectionAvailable));
        }

        // Required, removing this will cause some problems regarding Send and await
        let con = { self.connections.lock().unwrap() }.pop_front();
        self.available.fetch_sub(1, Ordering::SeqCst);
        if let Some(mut con) = con {
            if con.is_healthy().await {
                Ok(ConnectionGuard::new(self.clone(), con))
            } else {
                // Connection is dead, set the flag
                self.has_dead_con.store(true, Ordering::Relaxed);
                Err(CachemError::ConnectionPoolError(ConnectionPoolError::CannotConnect))
            }
        } else {
            Err(CachemError::ConnectionPoolError(ConnectionPoolError::NoConnectionAvailable))
        }
    }

    /// Releases a connection back into the connection pool
    ///
    /// # Params
    ///
    /// * `connection` - Raw [Connection]
    ///
    pub(crate) fn release(&self, connection: Connection) {
        self.connections.lock().unwrap().push_back(connection);
        self.available.fetch_add(1, Ordering::SeqCst);
    }

    /// Opens a connection and returns it
    ///
    /// # Returns
    ///
    /// If successful a [Connection] if not an error
    ///
    async fn connect(&self) -> Result<Connection, CachemError> {
        let stream = TcpStream::connect(&self.url)
            .await
            .map_err(|_| CachemError::ConnectionPoolError(ConnectionPoolError::CannotConnect))?;
        Ok(Connection::new(stream))
    }

    /// Drops all connections from the pool
    ///
    fn drop_all(&self) {
        log::warn!("Dropping all connections");
        let mut cons = self.connections.lock().unwrap();
        for _ in 0..cons.len() {
            self.available.fetch_sub(1, Ordering::SeqCst);
            std::mem::drop(cons.pop_front());
        }
    }


    /// Task that periodically checks if there is a dead connection.
    ///
    /// The interval is defined by CHECK_CONNECTIONS_MSEC.
    ///
    /// If a dead connection is detected, all connections are dropped and
    /// it will try to fill the pool with the required amount of connections.
    ///
    fn reconnect_task(&self) {
        let self_copy = self.clone();
        let connections_copy = self.connections.clone();

        tokio::task::spawn(async move {
            loop {
                let dead = self_copy.has_dead_con.load(Ordering::SeqCst);
                if dead {
                    log::error!("Dead connection detected");
                    self_copy.drop_all();

                    log::info!("Reconnecting");
                    let pool_size = self_copy.pool_size.load(Ordering::SeqCst);
                    for _ in 0..pool_size {
                        if let Ok(con) = self_copy.connect().await {
                            let mut cons = connections_copy.lock().unwrap();
                            cons.push_back(con);

                            self_copy.available.fetch_add(1, Ordering::SeqCst);
                        }
                    }
                }
                self_copy.has_dead_con.store(false, Ordering::SeqCst);
                std::thread::sleep(std::time::Duration::from_millis(Self::CHECK_CONNECTIONS_MSEC));
            }
        });
    }
}

