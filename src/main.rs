use std::sync::Arc;
use zettelkasten_shared::{
    storage::{self, ConnectableStorage},
    Front,
};

#[allow(clippy::redundant_clone)]
fn main() {
    zettelkasten_shared::block_on(async {
        let (connection, system_config) = data_policy_should_exist_exactly_once().await;
        #[cfg(feature = "front-terminal")]
        zettelkasten_terminal::Tui::run((), system_config.clone(), Arc::clone(&connection));
    });
}

#[cfg(feature = "data-sqlite")]
async fn data_policy_should_exist_exactly_once(
) -> (Arc<dyn storage::Storage>, storage::SystemConfig) {
    let _ = dotenv::dotenv();
    let connection_string =
        std::env::var("DATABASE_URL").expect("Missing environment variable DATABASE_URL");
    let (connection, config) = zettelkasten_sqlite::Connection::connect(connection_string)
        .await
        .expect("Could not open database");
    (Arc::new(connection), config)
}
