pub mod storage;

pub use async_trait::async_trait;
pub use futures;

use std::future::Future;
use std::sync::Arc;
use storage::SystemConfig;

#[async_trait]
pub trait Front: Send + Sync {
    type Config;
    async fn run(
        config: Self::Config,
        system_config: SystemConfig,
        storage: Arc<dyn storage::Storage>,
    );
}

#[cfg(feature = "runtime-async-std")]
pub fn block_on<F, RET>(f: F) -> RET
where
    F: Future<Output = RET>,
{
    async_std::task::block_on(f)
}

#[cfg(feature = "runtime-async-std")]
pub fn spawn_blocking<F>(f: F) -> impl Future<Output = ()>
where
    F: FnOnce() + Send + 'static,
{
    async_std::task::spawn_blocking(f)
}
