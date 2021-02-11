use async_trait::async_trait;
use std::sync::Arc;
use tokio::signal::unix::{SignalKind, signal};

use crate::CachemError;

#[deprecated]
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

use tokio::io::{AsyncBufRead, AsyncRead, AsyncWrite};
#[async_trait]
pub trait Storage: Sized {
    async fn file() -> &'static str;

    async fn load<B>(buf: &mut B) -> Result<Self, CachemError> 
        where B: AsyncBufRead + AsyncRead + Send + Unpin;

    async fn save<B>(&self, buf: &mut B) -> Result<(), CachemError>
        where B: AsyncWrite + Send + Unpin;
}

