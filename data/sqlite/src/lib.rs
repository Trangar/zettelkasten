use async_lock::Mutex;
use snafu::ResultExt;
use sqlx::ConnectOptions as _;
use std::{str::FromStr, sync::Arc};
use zettelkasten_shared::{
    futures::{future::LocalBoxFuture, FutureExt},
    storage,
};

pub struct Connection {
    conn: Arc<Mutex<sqlx::SqliteConnection>>,
}

#[allow(unused_variables)]
#[zettelkasten_shared::async_trait]
impl storage::Storage for Connection {
    async fn user_count(&self) -> Result<u64, storage::Error> {
        let query = sqlx::query!("SELECT COUNT(*) as count FROM users");
        query
            .fetch_one(&mut *self.conn.lock().await)
            .await
            .map(|result| result.count as u64)
            .context(storage::SqlxSnafu)
    }

    async fn login_single_user(&self) -> Result<storage::User, storage::Error> {
        let maybe_user = self.login("root", "").await?;
        maybe_user.ok_or(storage::Error::SingleUserNotFound)
    }

    async fn login(
        &self,
        username: &str,
        password: &str,
    ) -> Result<Option<storage::User>, storage::Error> {
        let query = sqlx::query_as!(
            storage::User,
            "SELECT user_id as id, username as name, password, last_visited_zettel FROM users WHERE username = ?",
            username,
        );
        let user = query
            .fetch_optional(&mut *self.conn.lock().await)
            .await
            .context(storage::SqlxSnafu)?;

        if let Some(user) = user {
            if bcrypt::verify(password, &user.password).context(storage::BcryptSnafu)? {
                return Ok(Some(user));
            }
        }

        Ok(None)
    }

    async fn register(
        &self,
        username: &str,
        password: &str,
    ) -> Result<storage::User, storage::Error> {
        let mut conn = self.conn.lock().await;
        let query = sqlx::query!(
            "SELECT COUNT(user_id) as count FROM users WHERE username = ?",
            username
        );
        let user = query
            .fetch_one(&mut *conn)
            .await
            .context(storage::SqlxSnafu)?;

        if user.count != 0 {
            return Err(storage::Error::UserAlreadyExists);
        }

        let password =
            bcrypt::hash(password, bcrypt::DEFAULT_COST).context(storage::BcryptSnafu)?;

        let query = sqlx::query_as!(
            storage::User,
            r#"INSERT INTO users (username, password) VALUES (?, ?) RETURNING user_id as "id!", username as "name!", password as "password!", last_visited_zettel"#,
            username,
            password
        );
        let user = query
            .fetch_one(&mut *conn)
            .await
            .context(storage::SqlxSnafu)?;

        Ok(user)
    }

    async fn get_zettels(
        &self,
        user: storage::UserId,
        search: Option<storage::SearchOpts>,
    ) -> Result<Vec<storage::ZettelHeader>, storage::Error> {
        todo!()
    }

    async fn get_zettel(
        &self,
        user: storage::UserId,
        id: storage::ZettelId,
    ) -> Result<storage::Zettel, storage::Error> {
        todo!()
    }

    async fn get_zettel_by_url(
        &self,
        user: storage::UserId,
        url: &str,
    ) -> Result<Option<storage::Zettel>, storage::Error> {
        dbg!(url);
        todo!()
    }

    async fn update_zettel(
        &self,
        user: storage::UserId,
        zettel: &storage::Zettel,
    ) -> Result<(), storage::Error> {
        todo!()
    }

    async fn set_user_last_visited_zettel(
        &self,
        user: storage::UserId,
        zettel_id: Option<storage::ZettelId>,
    ) -> Result<(), storage::Error> {
        let query = sqlx::query!(
            "UPDATE users SET last_visited_zettel = ? WHERE user_id = ?",
            zettel_id,
            user
        );
        query
            .execute(&mut *self.conn.lock().await)
            .await
            .context(storage::SqlxSnafu)?;
        Ok(())
    }
}

impl storage::ConnectableStorage for Connection {
    type ConnectionArgs = String;

    fn connect<'a>(
        connection_args: Self::ConnectionArgs,
    ) -> LocalBoxFuture<'a, Result<(Self, storage::SystemConfig), storage::Error>> {
        async move {
            println!("Opening {connection_args:?}");
            let mut connection = sqlx::sqlite::SqliteConnectOptions::from_str(&connection_args)
                .expect("Invalid SQLite connection string")
                .create_if_missing(true)
                .connect()
                .await
                .context(storage::SqlxSnafu)?;
            sqlx::migrate!()
                .run(&mut connection)
                .await
                .context(storage::SqlxMigrateSnafu)?;
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
            .await
            .context(storage::SqlxSnafu)?;

        // Map them into a `serde_json::Map`
        let map: serde_json::Map<String, serde_json::Value> = result
            .into_iter()
            .map(|o| Ok((o.key, serde_json::from_str(&o.value)?)))
            .collect::<Result<_, _>>()
            .context(storage::JsonDeserializeSnafu)?;

        // Now we can deserialize this from a `serde_json::Value::Object(map)`
        serde_json::from_value(serde_json::Value::Object(map))
            .context(storage::JsonDeserializeSnafu)
    }
}
