use crate::CachemError;

use async_trait::async_trait;
use tokio::{fs::OpenOptions, io::AsyncWriteExt};
use tokio::io::{AsyncBufRead, AsyncRead, AsyncWrite, BufStream};

#[async_trait]
pub trait Storage: Sized {
    /// Returns the filename
    fn file() -> &'static str;

    /// Loads the cache from the given buffer
    async fn load<B>(&self, buf: &mut B) -> Result<(), CachemError> 
        where B: AsyncBufRead + AsyncRead + Send + Unpin;

    /// Saves the current cache to the buffer
    async fn save<B>(&self, buf: &mut B) -> Result<(), CachemError>
        where B: AsyncWrite + Send + Unpin;

    /// Loads a cache from file. Uses [Storage::load] internally.
    async fn load_from_file(&self) -> Result<(), CachemError> {
        let file = OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .open(Self::file())
            .await?;
        let mut buf = BufStream::new(file);
        self.load(&mut buf).await
    }

    /// Saves the current cache to file. Uses [Storage::save] internally.
    async fn save_to_file(&self) -> Result<(), CachemError> {
        let file = OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .open(Self::file())
            .await?;
        let mut buf = BufStream::new(file);
        self.save(&mut buf).await?;
        buf.flush().await?;

        Ok(())
    }
}

