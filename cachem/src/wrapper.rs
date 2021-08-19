//! Implementations of the [cachem::Parse] trait for the basic datatypes.

use std::{collections::HashMap, hash::Hash};

use crate::{CachemError, Parse};

use async_trait::async_trait;
use tokio::io::{AsyncBufRead, AsyncBufReadExt, AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

#[async_trait]
impl Parse for u8 {
    async fn read<B>(
        buf: &mut B
    ) -> Result<Self, CachemError>
    where
        B: AsyncBufRead + AsyncRead + Send + Unpin {

        Ok(buf.read_u8().await?)
    }

    async fn write<B>(
        &self,
        buf: &mut B
    ) -> Result<(), CachemError>
    where
        B: AsyncWrite + Send + Unpin {

        buf.write_u8(*self).await?;
        Ok(())
    }
}

#[async_trait]
impl Parse for u16 {
    async fn read<B>(
        buf: &mut B
    ) -> Result<Self, CachemError>
    where
        B: AsyncBufRead + AsyncRead + Send + Unpin {

        Ok(buf.read_u16().await?)
    }

    async fn write<B>(
        &self,
        buf: &mut B
    ) -> Result<(), CachemError>
    where
        B: AsyncWrite + Send + Unpin {

        buf.write_u16(*self).await?;
        Ok(())
    }
}

#[async_trait]
impl Parse for u32 {
    async fn read<B>(
        buf: &mut B
    ) -> Result<Self, CachemError>
    where
        B: AsyncBufRead + AsyncRead + Send + Unpin {

        Ok(buf.read_u32().await?)
    }

    async fn write<B>(
        &self,
        buf: &mut B
    ) -> Result<(), CachemError>
    where
        B: AsyncWrite + Send + Unpin {

        buf.write_u32(*self).await?;
        Ok(())
    }
}

#[async_trait]
impl Parse for u64 {
    async fn read<B>(
        buf: &mut B
    ) -> Result<Self, CachemError>
    where
        B: AsyncBufRead + AsyncRead + Send + Unpin {

        Ok(buf.read_u64().await?)
    }

    async fn write<B>(
        &self,
        buf: &mut B
    ) -> Result<(), CachemError>
    where
        B: AsyncWrite + Send + Unpin {

        buf.write_u64(*self).await?;
        Ok(())
    }
}

#[async_trait]
impl Parse for u128 {
    async fn read<B>(
        buf: &mut B
    ) -> Result<Self, CachemError>
    where
        B: AsyncBufRead + AsyncRead + Send + Unpin {

        Ok(buf.read_u128().await?)
    }

    async fn write<B>(
        &self,
        buf: &mut B
    ) -> Result<(), CachemError>
    where
        B: AsyncWrite + Send + Unpin {

        buf.write_u128(*self).await?;
        Ok(())
    }
}

#[async_trait]
impl Parse for i8 {
    async fn read<B>(
        buf: &mut B
    ) -> Result<Self, CachemError>
    where
        B: AsyncBufRead + AsyncRead + Send + Unpin {

        Ok(buf.read_i8().await?)
    }

    async fn write<B>(
        &self,
        buf: &mut B
    ) -> Result<(), CachemError>
    where
        B: AsyncWrite + Send + Unpin {

        buf.write_i8(*self).await?;
        Ok(())
    }
}

#[async_trait]
impl Parse for i16 {
    async fn read<B>(
        buf: &mut B
    ) -> Result<Self, CachemError>
    where
        B: AsyncBufRead + AsyncRead + Send + Unpin {

        Ok(buf.read_i16().await?)
    }

    async fn write<B>(
        &self,
        buf: &mut B
    ) -> Result<(), CachemError>
    where
        B: AsyncWrite + Send + Unpin {

        buf.write_i16(*self).await?;
        Ok(())
    }
}

#[async_trait]
impl Parse for i32 {
    async fn read<B>(
        buf: &mut B
    ) -> Result<Self, CachemError>
    where
        B: AsyncBufRead + AsyncRead + Send + Unpin {

        Ok(buf.read_i32().await?)
    }

    async fn write<B>(
        &self,
        buf: &mut B
    ) -> Result<(), CachemError>
    where
        B: AsyncWrite + Send + Unpin {

        buf.write_i32(*self).await?;
        Ok(())
    }
}

#[async_trait]
impl Parse for i64 {
    async fn read<B>(
        buf: &mut B
    ) -> Result<Self, CachemError>
    where
        B: AsyncBufRead + AsyncRead + Send + Unpin {

        Ok(buf.read_i64().await?)
    }

    async fn write<B>(
        &self,
        buf: &mut B
    ) -> Result<(), CachemError>
    where
        B: AsyncWrite + Send + Unpin {

        buf.write_i64(*self).await?;
        Ok(())
    }
}

#[async_trait]
impl Parse for i128 {
    async fn read<B>(
        buf: &mut B
    ) -> Result<Self, CachemError>
    where
        B: AsyncBufRead + AsyncRead + Send + Unpin {

        Ok(buf.read_i128().await?)
    }

    async fn write<B>(
        &self,
        buf: &mut B
    ) -> Result<(), CachemError>
    where
        B: AsyncWrite + Send + Unpin {

        buf.write_i128(*self).await?;
        Ok(())
    }
}

#[async_trait]
impl Parse for f32 {
    async fn read<B>(
        buf: &mut B
    ) -> Result<Self, CachemError>
    where
        B: AsyncBufRead + AsyncRead + Send + Unpin {

        let mut val = [0u8; 4];
        buf.read_exact(&mut val).await?;
        Ok(f32::from_be_bytes(val))
    }

    async fn write<B>(
        &self,
        buf: &mut B
    ) -> Result<(), CachemError>
    where
        B: AsyncWrite + Send + Unpin {

        buf.write_all(&self.to_be_bytes()).await?;
        Ok(())
    }
}

#[async_trait]
impl Parse for f64 {
    async fn read<B>(
        buf: &mut B
    ) -> Result<Self, CachemError>
    where
        B: AsyncBufRead + AsyncRead + Send + Unpin {

        let mut val = [0u8; 8];
        buf.read_exact(&mut val).await?;
        Ok(f64::from_be_bytes(val))
    }

    async fn write<B>(
        &self,
        buf: &mut B
    ) -> Result<(), CachemError>
    where
        B: AsyncWrite + Send + Unpin {

        buf.write_all(&self.to_be_bytes()).await?;
        Ok(())
    }
}

#[async_trait]
impl Parse for String {
    async fn read<B>(
        buf: &mut B
    ) -> Result<Self, CachemError>
    where
        B: AsyncBufRead + AsyncRead + Send + Unpin {

        let mut val = Vec::new();
        let read = buf.read_until(0u8, &mut val).await?;

        // Remove the trailing 0 byte
        let val = if read > 0 {
            val[0..read - 1].to_vec()
        } else {
            val
        };
        Ok(String::from_utf8(val)?)
    }

    async fn write<B>(
        &self,
        buf: &mut B
    ) -> Result<(), CachemError>
    where
        B: AsyncWrite + Send + Unpin {

        buf.write_all(self.as_bytes()).await?;
        buf.write_u8(0u8).await?;
        Ok(())
    }
}

#[async_trait]
impl Parse for bool {
    async fn read<B>(
        buf: &mut B
    ) -> Result<Self, CachemError>
    where
        B: AsyncBufRead + AsyncRead + Send + Unpin {

        let val = buf.read_u8().await?;
        Ok(val == 1)
    }

    async fn write<B>(
        &self,
        buf: &mut B
    ) -> Result<(), CachemError>
    where
        B: AsyncWrite + Send + Unpin {

        buf.write_u8(*self as u8).await?;
        Ok(())
    }
}

#[async_trait]
impl<T: Parse + Send + Sync> Parse for Vec<T> {
    async fn read<B>(
        buf: &mut B
    ) -> Result<Self, CachemError>
    where
        B: AsyncBufRead + AsyncRead + Send + Unpin {

        let entry_count = u32::read(buf).await?;
        let mut entries = Vec::with_capacity(entry_count as usize);

        for _ in 0..entry_count {
            entries.push(T::read(buf).await?);
        }

        Ok(entries)
    }

    async fn write<B>(
        &self,
        buf: &mut B
    ) -> Result<(), CachemError>
    where
        B: AsyncWrite + Send + Unpin {

        buf.write_u32(self.len() as u32).await?;
        for entry in self {
            entry.write(buf).await?;
        }

        Ok(())
    }
}

#[async_trait]
impl<K, V> Parse for HashMap<K, V>
where
    K: Parse + Eq + Hash + Send + Sync,
    V: Parse + Send + Sync {

    async fn read<B>(
        buf: &mut B
    ) -> Result<Self, CachemError>
    where
        B: AsyncBufRead + AsyncRead + Send + Unpin {

        let entry_count = u32::read(buf).await?;
        let mut entries = HashMap::with_capacity(entry_count as usize);

        for _ in 0..entry_count {
            let k = K::read(buf).await?;
            let v = V::read(buf).await?;
            entries.insert(k, v);
        }

        Ok(entries)
    }

    async fn write<B>(
        &self,
        buf: &mut B
    ) -> Result<(), CachemError>
    where
        B: AsyncWrite + Send + Unpin {

        buf.write_u32(self.len() as u32).await?;
        for (k, v) in self {
            k.write(buf).await?;
            v.write(buf).await?;
        }

        Ok(())
    }
}

#[async_trait]
impl<T> Parse for Option<T>
where 
    T: Parse + Send + Sync {

    async fn read<B>(
        buf: &mut B
    ) -> Result<Self, CachemError>
    where
        B: AsyncBufRead + AsyncRead + Send + Unpin {

        let res = bool::read(buf).await?;
        if res {
            let some = T::read(buf).await?;
            Ok(Some(some))
        } else {
            Ok(None)
        }
    }

    async fn write<B>(
        &self,
        buf: &mut B
    ) -> Result<(), CachemError>
    where
        B: AsyncWrite + Send + Unpin {

        match self {
            Some(x) => {
                true.write(buf).await?;
                x.write(buf).await?;
            },
            None => {
                false.write(buf).await?;
            }
        }

        Ok(())
    }
}

#[async_trait]
impl<T, E> Parse for Result<T, E>
where
    T: Parse + Send + Sync,
    E: Parse + Send + Sync {

    async fn read<B>(
        buf: &mut B
    ) -> Result<Self, CachemError>
    where
        B: AsyncBufRead + AsyncRead + Send + Unpin {

        let res = bool::read(buf).await?;
        if res {
            let ok = T::read(buf).await?;
            Ok(Ok(ok))
        } else {
            let err = E::read(buf).await?;
            Ok(Err(err))
        }
    }

    async fn write<B>(
        &self,
        buf: &mut B
    ) -> Result<(), CachemError>
    where
        B: AsyncWrite + Send + Unpin {

        match self {
            Ok(x) => {
                true.write(buf).await?;
                x.write(buf).await?;
            },
            Err(x) => {
                false.write(buf).await?;
                x.write(buf).await?;
            }
        }

        Ok(())
    }
}

#[async_trait]
impl Parse for () {
    async fn read<B>(
        _: &mut B
    ) -> Result<Self, CachemError>
    where
        B: AsyncBufRead + AsyncRead + Send + Unpin {

        Ok(())
    }

    async fn write<B>(
        &self,
        _: &mut B
    ) -> Result<(), CachemError>
    where
        B: AsyncWrite + Send + Unpin {

        Ok(())
    }
}

/// Wrapper for an empty message.
/// An empty message writes a single byte in the buffer so that the other side
/// knows that the transmission is over and no more data is expected.
///
#[derive(Debug, Default)]
pub struct EmptyMsg;

#[async_trait]
impl Parse for EmptyMsg {
    async fn read<B>(
        buf: &mut B,
    ) -> Result<Self, CachemError>
    where
        B: AsyncBufRead + AsyncRead + Send + Unpin  {

        let _ = buf.read_u8().await?;
        Ok(Self::default())
    }

    async fn write<B>(
        &self,
        buf: &mut B,
    ) -> Result<(), CachemError>
    where
        B: AsyncWrite + Send + Unpin {

        buf.write_u8(0u8).await?;
        Ok(())
    }
}

#[cfg(feature = "uuid")]
#[async_trait]
impl Parse for uuid::Uuid {
    async fn read<B>(
        buf: &mut B
    ) -> Result<Self, CachemError>
    where
        B: AsyncBufRead + AsyncRead + Send + Unpin {

        let val = buf.read_u128().await?;
        Ok(uuid::Uuid::from_u128(val))
    }

    async fn write<B>(
        &self,
        buf: &mut B
    ) -> Result<(), CachemError>
    where
        B: AsyncWrite + Send + Unpin {

        buf.write_u128(self.as_u128()).await?;
        Ok(())
    }
}
