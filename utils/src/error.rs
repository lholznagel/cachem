#[derive(Debug)]
pub enum CachemError {
    Empty,
    IoError(std::io::Error),
    StringParseError(std::string::FromUtf8Error),
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

/// All errors that can be thrown by the connection pool
#[derive(Debug)]
pub enum CachemConnectionPoolError {
    /// The pool is currently empty
    NoConnectionAvailable,
    /// Connecting to the remote server did not work
    CannotConnect,
    /// The pool has not enough connections to scale down
    NotEnoughConnectionsInPool,
    /// There are not enough connection available to scale down
    NotEnoughConnectionsAvailable,
    TimeoutGettingConnection,
}
impl std::error::Error for CachemConnectionPoolError {}

impl std::fmt::Display for CachemConnectionPoolError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
