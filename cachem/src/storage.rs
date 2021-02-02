use async_trait::async_trait;
use std::sync::Arc;
use tokio::signal::unix::{SignalKind, signal};

use crate::CachemError;

/// Support struct for saving the content of caches
///
/// ```
/// # use async_trait::*;
/// # use cachem_utils::*;
/// # use std::sync::Arc;
///
/// # #[tokio::main]
/// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
/// #[derive(Default)]
/// pub struct MyCache;
/// 
/// #[async_trait]
/// impl Save for MyCache {
///     async fn store(&self) -> Result<(), CachemError> {
///         // let mut buf = Cursor::new();
///         // ... write the content of the cache into the buffer
///         // save the file
///         // FileUtils::save("my_file", buf).await?;
///         Ok(())
///     }
/// }
///
/// // Create a new StorageHandler instance
/// let mut storage = StorageHandler::default();
/// // Register our cache
/// storage.register(Arc::new(MyCache::default()));
/// // Spawn a seperate task, so that the main thread is not blocked
/// tokio::task::spawn(async move {
///     storage.save_on_interrupt().await;
/// });
///
/// # Ok(())
/// # }
/// ```
#[derive(Clone, Default)]
pub struct StorageHandler {
    registered: Vec<Arc<dyn Save + Send + Sync>>
}

impl StorageHandler {
    /// Registers a new cache to be handled
    /// The cache must implement the [Save] trait.
    pub fn register(&mut self, save: Arc<dyn Save + Send + Sync>) {
        self.registered.push(save)
    }

    /// Saves all registered caches when an SIGINT is received.
    /// All errors that occur during saving are ignored.
    ///
    /// NOTE:
    /// This function is blocking when calling `.await`.
    /// It is recommended to spawn a seperate task.
    /// See the struct example for more information.
    pub async fn save_on_interrupt(&self) {
        // Register a signal for a SIGINT
        signal(SignalKind::interrupt())
            .expect("Creating a signal listener should be successful")
            .recv()
            .await;

        // iterate over all registered caches and call there store function
        for cache in self.registered.iter() {
            let _ = cache.store().await;
        }

        // cleanly exit the application
        std::process::exit(0);
    }
}

/// Trait for saving the current cache to a file.
/// This trait can be used in combination with [StorageHandler] for an easy
/// way to save the cacehs on SIGINT.
#[async_trait]
pub trait Save {
    async fn store(&self) -> Result<(), CachemError>;
}
