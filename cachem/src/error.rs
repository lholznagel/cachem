#[derive(Debug)]
pub enum CachemError {
    Empty,
    NotReachable,
    IoError(std::io::Error),
    StringParseError(std::string::FromUtf8Error),
    ConnectionPoolError(ConnectionPoolError),
}
impl std::error::Error for CachemError {}

impl std::fmt::Display for CachemError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self) 
    }
}

impl From<std::io::Error> for CachemError {
    fn from(x: std::io::Error) -> Self {
        CachemError::IoError(x)
    }
}

impl From<std::string::FromUtf8Error> for CachemError {
    fn from(x: std::string::FromUtf8Error) -> Self {
        CachemError::StringParseError(x)
    }
}

#[derive(Debug)]
pub enum ConnectionPoolError {
    /// The pool is currently empty
    NoConnectionAvailable,
    /// Connecting to the remote server did not work
    CannotConnect,
    /// The pool has not enough connections to scale down
    NotEnoughConnectionsInPool,
    /// There are not enough connection available to scale down
    NotEnoughConnectionsAvailable,
    /// There was no connection available in the timeout
    TimeoutGettingConnection,
}
