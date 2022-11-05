pub mod storage;

pub use async_trait::async_trait;
pub use futures;

use std::future::Future;
use std::sync::Arc;
use storage::SystemConfig;

pub trait Front: Send + Sync {
    type Config: serde::ser::Serialize + serde::de::DeserializeOwned + Default;
    fn run(config: Self::Config, system_config: SystemConfig, storage: Arc<dyn storage::Storage>);
}

#[cfg(feature = "runtime-async-std")]
pub fn block_on<F, RET>(f: F) -> RET
where
    F: Future<Output = RET>,
{
    async_std::task::block_on(f)
}

#[cfg(feature = "runtime-async-std")]
pub use async_std::main;
