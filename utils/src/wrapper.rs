use crate::{CachemError, Parse};

use async_trait::async_trait;
use tokio::io::{AsyncBufRead, AsyncRead, AsyncBufReadExt, AsyncReadExt, AsyncWrite, AsyncWriteExt};

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
        buf.read_until(0u8, &mut val).await?;
        Ok(String::from_utf8(val)?)
    }

    async fn write<B>(
        &self,
        buf: &mut B
    ) -> Result<(), CachemError>
    where
        B: AsyncWrite + Send + Unpin {

        buf.write_all(&self.as_bytes()).await?;
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
