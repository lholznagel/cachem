use super::{Cache, Command};

use async_trait::*;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufStream};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::watch::{self, Sender, Receiver};

/// Struct for creating a new database server
pub struct Server {
    /// Address the server should listen to
    addr:    String,
    /// All manges caches
    entries: HashMap<u8, Arc<dyn Cache>>,
}

impl Server {
    /// Creates a new server instance
    ///
    /// # Params
    ///
    /// * `addr` - TCP addr to listen for incoming connections
    ///
    /// # Returns
    ///
    /// * `Receiver<Command>` - Receiver from the Command and Control network
    pub fn new(addr: String) -> (Receiver<Command>, Self) {
        let (tx, rx) = watch::channel(Command::Ping);
        let cnc = CommandAndControl::new(tx);

        let mut map: HashMap<u8, Arc<dyn Cache>> = HashMap::new();
        map.insert(255, Arc::new(cnc));

        let s = Self {
            addr,
            entries:      map,
        };

        (rx, s)
    }

    /// Adds a new managed cache
    ///
    /// # Params
    ///
    /// * `name` - Name of the cache, this must implement Into<u8>
    /// * `cache` - Instance of the cache
    pub fn add<T: Into<u8>>(&mut self, name: T, cache: Arc<dyn Cache>) -> &mut Self {
        self.entries.insert(name.into(), cache.clone());
        self
    }

    /// Stats the cnc network listener
    pub fn listen_cnc(&self) {
        let mut tasks = Vec::new();

        for (_, cache) in self.entries.clone() {
            tasks.push(tokio::task::spawn(async move { cache.cnc_listener().await }));
        }
    }

    /// Starts the tcp listener for incoming connections
    ///
    /// # Panics
    ///
    /// This should really not panic
    /// TODO
    ///
    pub async fn listen_tcp(&self) {
        let listener = TcpListener::bind(&self.addr).await.unwrap();
        loop {
            let entries_copy = self.entries.clone();
            let (mut socket, _) = listener.accept().await.unwrap();

            tokio::spawn(async move {
                let mut cmd: [u8; 1] = [0; 1];
                loop {
                    let mut buf_socket = tokio::io::BufStream::new(socket);
                    match buf_socket.read(&mut cmd).await {
                        // socket closed
                        Ok(n) if n == 0 => return,
                        Ok(n) => n,
                        Err(e) => {
                            eprintln!("failed to read from socket; err = {:?}", e);
                            return;
                        }
                    };

                    let cmd = Command::from(cmd[0]);
                    if cmd == Command::Ping {
                        buf_socket.write_u8(Command::Pong.into()).await.unwrap();
                        buf_socket.flush().await.unwrap();
                        socket = buf_socket.into_inner();
                        continue;
                    }

                    let cache = buf_socket.read_u8().await.unwrap();
                    if let Some(e) = entries_copy.get(&cache) {
                        e.handle(cmd, &mut buf_socket).await;
                    } else {
                        log::error!("Could not find cache");
                    }

                    buf_socket.flush().await.unwrap();

                    // return the socket so that we don´t consume it
                    socket = buf_socket.into_inner();
                }
            });
        }
    }
}

/// Command and control network for inter service communication
pub struct CommandAndControl {
    /// Sender for the network
    cnc_rec: Sender<Command>,
}

impl CommandAndControl {
    /// Creates a new cnc instance
    pub fn new(cnc_rec: Sender<Command>) -> Self {
        Self {
            cnc_rec
        }
    }
}

/// The cnc network is handled like a cache so that we can easily register it
#[async_trait]
impl Cache for CommandAndControl {
    fn name(&self) -> String {
        "Command n Control".into()
    }

    async fn handle(&self, _: Command, _: &mut BufStream<TcpStream>) {
        self.cnc_rec.send(Command::Get).unwrap();
    }

    async fn cnc_listener(&self) {  }
}

