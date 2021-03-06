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
/// # use async_trait::*;
/// # use cachem::*;
/// # #[tokio::main]
/// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let pool = ConnectionPool::new("127.0.0.1:1337".into(), 1).await?;
/// let mut conn = pool.acquire().await?;
///
/// #[derive(Debug, Parse)]
/// struct FetchById(u32);
/// 
/// #[async_trait]
/// impl ProtocolRequest for FetchById {
///     fn action(&self) -> u8 {
///         0u8
///     }
/// 
///     fn cache(&self) -> u8 {
///         0u8
///     }
/// }
/// 
/// #[derive(Debug, Parse)]
/// struct IdEntry {
///     pub id: u32,
///     pub val: u32,
/// }
/// 
/// // Creates a new request to fetch an entry from the "Names" Cache by id
/// // The result of that request will be parsed into the "NameEntry" struct.
/// let response = Protocol::request::<_, IdEntry>(
///     &mut conn,
///     FetchById(18u32)
/// )
/// .await?;
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
        T: Parse + Request,
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
        T: Parse + Request,
        R: Parse {

        buf.write_u16(data.action().into()).await?;
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
/// # use cachem::*;
/// # use tokio::io::{AsyncBufRead, AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
/// # #[tokio::main]
/// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
/// #[derive(Debug)]
/// pub struct ExampleImplementation(pub u32);
/// 
/// #[async_trait]
/// impl Parse for ExampleImplementation {
///     async fn read<B>(
///         buf: &mut B,
///     ) -> Result<Self, CachemError>
///     where
///         B: AsyncBufRead + AsyncRead + Send + Unpin {
/// 
///         let x = buf.read_u32().await?;
///         Ok(Self(x))
///     }
///
///     async fn write<B>(
///         &self,
///         buf: &mut B,
///     ) -> Result<(), CachemError>
///         where
///             B: AsyncWrite + Send + Unpin {
///
///         buf.write_u32(self.0).await?;
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
        buf: &mut B,
    ) -> Result<Self, CachemError>
    where
        B: AsyncBufRead + AsyncRead + Send + Unpin;

    async fn write<B>(
        &self,
        buf: &mut B,
    ) -> Result<(), CachemError>
    where
        B: AsyncWrite + Send + Unpin;
}

/// Provides functions that are needed in order to send a request from the 
/// driver to the database
///
/// Requires that [`Parse`] is implemented. In this example the trait is implemented
/// using the derive proc macro.
///
/// ## Implementation example:
/// ```
/// # use async_trait::*;
/// # use cachem::*;
/// # #[tokio::main]
/// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
/// 
/// pub enum Actions {
///     Fetch
/// }
///
/// impl Into<u8> for Actions {
///     fn into(self) -> u8 {
///         0u8
///     }
/// }
/// 
/// #[derive(Debug, Parse)]
/// pub struct ExampleImplementation(Vec<u8>);
///
/// #[async_trait]
/// impl Request for ExampleImplementation {
///     fn action(&self) -> u8 {
///         Actions::Fetch.into()
///     }
/// }
/// # Ok(())
/// # }
/// ```
#[async_trait]
pub trait Request {
    /// Determines what type of action this request initiates
    fn action(&self) -> u16;
}
