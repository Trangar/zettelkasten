pub mod storage;

pub use async_trait::async_trait;
pub use futures;

use std::sync::Arc;
use storage::SystemConfig;

pub trait Front: Send + Sync {
    type Config: serde::ser::Serialize + serde::de::DeserializeOwned + Default;
    fn run(config: Self::Config, system_config: SystemConfig, storage: Arc<dyn storage::Storage>);
}
