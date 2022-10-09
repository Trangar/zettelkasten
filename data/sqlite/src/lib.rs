use async_lock::Mutex;
use sqlx::{Connection as _, Executor};
use std::{collections::HashMap, sync::Arc};
use zettelkasten_shared::{
    futures::{future::LocalBoxFuture, FutureExt},
    storage,
};

pub struct Connection {
    conn: Arc<Mutex<sqlx::SqliteConnection>>,
}

#[zettelkasten_shared::async_trait]
impl storage::Storage for Connection {
    async fn login_single_user(&self) -> Result<storage::User, storage::Error> {
        let maybe_user = self.login("root", "").await?;
        maybe_user.ok_or(storage::Error::SingleUserNotFound)
    }

    async fn login(
        &self,
        username: &str,
        password: &str,
    ) -> Result<Option<storage::User>, storage::Error> {
        let user = sqlx::query_as!(
            storage::User,
            "SELECT user_id as id, username as name, password, last_visited_page FROM users WHERE username = ?",
            username,
        )
        .fetch_optional(&mut *self.conn.lock().await)
        .await?;

        if let Some(user) = user {
            if bcrypt::verify(password, &user.password)? {
                return Ok(Some(user));
            }
        }

        Ok(None)
    }

    async fn register(
        &self,
        username: &str,
        password: &str,
    ) -> Result<Option<storage::User>, storage::Error> {
        todo!()
    }

    async fn get_pages(
        &self,
        user: storage::UserId,
        search: Option<storage::SearchOpts>,
    ) -> Result<Vec<storage::PageHeader>, storage::Error> {
        todo!()
    }

    async fn get_page(
        &self,
        user: storage::UserId,
        id: storage::PageId,
    ) -> Result<Vec<storage::Page>, storage::Error> {
        todo!()
    }

    async fn get_page_by_url(
        &self,
        user: storage::UserId,
        url: &str,
    ) -> Result<Option<storage::Page>, storage::Error> {
        todo!()
    }

    async fn update_page(
        &self,
        user: storage::UserId,
        page: &storage::Page,
    ) -> Result<(), storage::Error> {
        todo!()
    }
}

impl storage::ConnectableStorage for Connection {
    type ConnectionArgs = String;

    fn connect<'a>(
        connection_args: Self::ConnectionArgs,
    ) -> LocalBoxFuture<'a, Result<(Self, storage::SystemConfig), storage::Error>> {
        async move {
            let mut connection = sqlx::SqliteConnection::connect(&connection_args).await?;
            sqlx::migrate!().run(&mut connection).await?;
            let connection = Connection {
                conn: Arc::new(Mutex::new(connection)),
            };

            let config = connection.load_config().await?;
            dbg!(&config);

            Ok((connection, config))
        }
        .boxed_local()
    }
}

impl Connection {
    async fn load_config(&self) -> Result<storage::SystemConfig, storage::Error> {
        // load all the key-value entries from the database
        let result = sqlx::query!("SELECT key, value FROM config")
            .fetch_all(&mut *self.conn.lock().await)
            .await?;

        dbg!(&result);
        // Map them into a `serde_json::Map`
        let map: serde_json::Map<String, serde_json::Value> = match result
            .into_iter()
            .map(|o| Ok((o.key, serde_json::from_str(&o.value)?)))
            .collect()
        {
            Ok(map) => map,
            Err(inner) => return Err(storage::Error::JsonDeserializeError { inner }),
        };

        // Now we can deserialize this from a `serde_json::Value::Object(map)`
        serde_json::from_value(serde_json::Value::Object(map))
            .map_err(|inner| storage::Error::JsonDeserializeError { inner })
    }
}
