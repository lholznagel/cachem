//! This module exports all functions that are needed in order to establish
//! a communication between the driver and the database
//! See the internal modules for more information
//! Contains all traits that are used in the protocol module

use crate::{CachemError, Connection};

use async_trait::async_trait;
use tokio::io::{AsyncBufRead, AsyncRead, AsyncWrite, AsyncWriteExt};

/// Provides functions for working with the protocol between driver and database
///
/// ## Example sending a message to the database
/// ```no_run
/// # use carina::*;
/// # use carina::caches::*;
/// # #[tokio::main]
/// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let pool = ConnectionPool::new(1).await?;
/// let mut conn = pool.acquire().await?;
/// 
/// // Creates a new request to fetch an entry from the "Names" Cache by id
/// // The result of that request will be parsed into the "NameEntry" struct.
/// let response = Protocol::request::<_, NameEntry>(
///     &mut conn,
///     FetchNameEntryById(18u32)
/// )
/// .await?;
///
/// pool.release(conn).await;
/// # Ok(())
/// # }
pub struct Protocol;

impl Protocol {
    /// Takes a connection from the connection pool and builds the protocol
    /// based on the information of the given data.
    /// Afterwards it reads the data of the socket and tries to parse it into
    /// the given response data type
    pub async fn request<T, R>(
        conn: &mut Connection,
        data: T,
    ) -> Result<R, CachemError>
    where
        T: Parse + ProtocolRequest,
        R: Parse {

        Protocol::request_with_buf(conn.get_mut(), data).await
    }

    /// Takes a buffer and builds the protocol  based on the information of the
    /// given data.
    /// Afterwards it reads the data of the socket and tries to parse it into
    /// the given response data type
    pub(crate) async fn request_with_buf<B, T, R>(
        buf: &mut B,
        data: T,
    ) -> Result<R, CachemError>
    where
        B: AsyncBufRead + AsyncRead + AsyncWrite + Send + Unpin,
        T: Parse + ProtocolRequest,
        R: Parse {

        buf.write_u8(data.action().into()).await?;
        buf.write_u8(data.cache_type().into()).await?;
        data.write(buf).await?;
        buf.flush().await?;

        Ok(R::read(buf).await?)
    }

    /// Creates a new response to the driver. Takes a buffer and a response 
    /// struct
    pub async fn response<B, T>(
        buf: &mut B,
        data: T,
    ) -> Result<(), CachemError>
    where
        B: AsyncWrite + Send + Unpin,
        T: Parse {

        data.write(buf).await?;
        buf.flush().await?;
        Ok(())
    }

    /// Parses the given buffer into the given response struct
    pub async fn read<B, R>(
        buf: &mut B
    ) -> Result<R, CachemError>
    where
        B: AsyncBufRead + AsyncRead + Send + Unpin,
        R: Parse {

        Ok(R::read(buf).await?)
    }
}

/// Provides functions to parse a message into an struct
///
/// ## Implementation example:
/// ```
/// # use async_trait::*;
/// # use carina::*;
/// # use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
/// # #[tokio::main]
/// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
/// #[derive(Debug)]
/// pub struct ExampleImplementation(Vec<u8>);
/// 
/// #[async_trait]
/// impl Parse for ExampleImplementation {
///     async fn parse<B>(
///         mut data: B
///     ) -> Result<S, CachemErrorelf>
///         where
///             B: AsyncRead + Send + Unpin {
/// 
///         let mut buf = Vec::new();
///         data.read_buf(&mut buf).await?;
///         Ok(Self(buf))
///     }
///
///     async fn write<B>(
///         &self,
///         buf: &mut B,
///     ) -> Result<(, CachemError)>
///         where
///             B: AsyncWrite + Send + Unpin {
///
///         buf.write_all(&self.0).await?;
///         Ok(())
///     }
/// }
///
/// # Ok(())
/// # }
/// ```
#[async_trait]
pub trait Parse: Sized {
    /// An implementor should read the given buffer and parse that data into the
    /// implementing struct
    async fn read<B>(
        buf: &mut B
    ) -> Result<Self, CachemError>
    where
        B: AsyncBufRead + AsyncRead + Send + Unpin;

    async fn write<B>(
        &self,
        buf: &mut B
    ) -> Result<(), CachemError>
    where
        B: AsyncWrite + Send + Unpin;
}

/// Provides functions that are needed in order to send a request from the 
/// driver to the database
///
/// Requires that [`Parse`] is implemented
///
/// ## Implementation example:
/// ```
/// # use async_trait::*;
/// # use carina::*;
/// # use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
/// # #[tokio::main]
/// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
/// #[derive(Debug)]
/// pub struct ExampleImplementation(Vec<u8>);
/// 
/// #[async_trait]
/// impl Parse for ExampleImplementation {
///     async fn parse<B>(
///         mut data: B
///     ) -> Result<S, CachemErrorelf>
///         where
///             B: AsyncRead + Send + Unpin {
/// 
///         let mut buf = Vec::new();
///         data.read_buf(&mut buf).await?;
///         Ok(Self(buf))
///     }
/// }
/// 
/// #[async_trait]
/// impl ProtocolRequest for ExampleImplementation {
///     fn action(&self) -> Action {
///         Action::Fetch
///     }
/// 
///     fn cache_type(&self) -> CacheType {
///         CacheType::IdName
///     }
/// 
///     async fn write<B>(
///         self,
///         buf: &mut B
///     ) -> Result<(, CachemError)>
///         where B: AsyncWrite + Unpin + Send {
/// 
///         buf.write_all(&self.0).await?;
///         Ok(())
///     }
/// }
/// # Ok(())
/// # }
/// ```
#[async_trait]
pub trait ProtocolRequest {
    /// Determines what type of action this request initiates
    fn action(&self) -> u8;

    /// Determines what type of cache should be used
    fn cache_type(&self) -> u8;
}
