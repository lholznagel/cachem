//! This module exports all functions that are needed in order to establish
//! a communication between the driver and the database
//! See the internal modules for more information
//! Contains all traits that are used in the protocol module

use crate::CachemError;

use async_trait::async_trait;
use tokio::io::{AsyncBufRead, AsyncRead, AsyncWrite};

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
    ///
    /// # Params
    ///
    /// * `buf` - Buffer for the network that can be read until its empty
    ///
    /// # Returns
    ///
    /// `Ok(Self)`         - if the reading was successfully
    /// `Err(CachemError)` - if their was an error while reading
    ///
    async fn read<B>(
        buf: &mut B,
    ) -> Result<Self, CachemError>
    where
        B: AsyncBufRead + AsyncRead + Send + Unpin;

    /// An implementor should write the struct into the buffer
    ///
    /// # Params
    ///
    /// * `buf` - Buffer for the network that can be written until the struct
    ///           is completly written
    ///
    /// # Returns
    ///
    /// `Ok(())`           - if the writing was successfully
    /// `Err(CachemError)` - if their was an error while reading
    ///
    async fn write<B>(
        &self,
        buf: &mut B,
    ) -> Result<(), CachemError>
    where
        B: AsyncWrite + Send + Unpin;
}

