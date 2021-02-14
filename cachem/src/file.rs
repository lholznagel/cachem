use crate::{CachemError, Parse};

use tokio::fs::OpenOptions;
use tokio::io::{AsyncWriteExt, BufStream};

/// Wraps [`tokio::fs::File`] type for easier testability
///
/// When compiled as a test build, all filesystem based implementations
/// are replaced with non filesystem implementations, while mainining
/// the same interface.
///
/// When calling [`FileUtils::write`] it will not write directly in the file, instead
/// it will write in an internal buffer. Only on when [`FileUtils::save`] is
/// called the whole buffer will be written and afterwards cleared
pub struct FileUtils;

impl FileUtils {
    /// Loads the file and parses it into the given model
    ///
    /// If the file does not exist, it will be created
    pub async fn open<R>(
        path: &str
    ) -> Result<Vec<R>, CachemError>
    where 
        R: Parse {

        let file = OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .open(path)
            .await?;

        let file_size = std::fs::metadata(path)?.len();
        let mut buf = BufStream::new(file);

        if file_size > 0 {
            let length = u32::read(&mut buf).await?;
            let mut result = Vec::with_capacity(length as usize);
            for _ in 0..length {
                result.push(R::read(&mut buf).await?)
            }
            Ok(result)
        } else {
            Ok(Vec::new())
        }
    }

    /// Writes the internal buffer to the file, clears the buffer and flushes
    pub async fn save<T>(
        path: &str,
        entries: Vec<T>,
    ) -> Result<(), CachemError>
    where
        T: Parse {

        let file = OpenOptions::new()
            .write(true)
            .open(path)
            .await?;

        let mut buf = BufStream::new(file);

        u32::from(entries.len() as u32).write(&mut buf).await?;
        for entry in entries {
            entry.write(&mut buf).await?;
        }
        buf.flush().await?;
        Ok(())
    }
}
