use snafu::ResultExt;
use zettelkasten_shared::{
    async_trait,
    futures::{future::LocalBoxFuture, FutureExt},
    storage::{
        BcryptSnafu, ConnectableStorage, Error, JsonSnafu, SearchOpts, SqlxSnafu, Storage,
        SystemConfig, User, UserId, Zettel, ZettelHeader, ZettelId,
    },
};

pub struct Connection {
    conn: sqlx::PgPool,
}

#[async_trait]
impl Storage for Connection {
    async fn user_count(&self) -> Result<u64, Error> {
        let mut conn = self.conn.acquire().await.context(SqlxSnafu)?;
        let result = sqlx::query!("SELECT COUNT(*) as \"user_count!\" FROM users")
            .fetch_one(&mut conn)
            .await
            .context(SqlxSnafu)?;
        Ok(result.user_count.try_into().unwrap())
    }

    async fn login_single_user(&self) -> Result<User, Error> {
        let mut conn = self.conn.acquire().await.context(SqlxSnafu)?;
        sqlx::query_as!(
            User,
            "SELECT user_id as id, username as name, password, last_visited_zettel FROM users"
        )
        .fetch_one(&mut conn)
        .await
        .context(SqlxSnafu)
    }
    async fn login(&self, username: &str, password: &str) -> Result<Option<User>, Error> {
        let mut conn = self.conn.acquire().await.context(SqlxSnafu)?;
        let user = sqlx::query_as!(
            User,
            "SELECT user_id as id, username as name, password, last_visited_zettel FROM users WHERE username = $1",
            username
        )
        .fetch_optional(&mut conn)
        .await
        .context(SqlxSnafu)?;

        if let Some(user) = user {
            if bcrypt::verify(password, &user.password).context(BcryptSnafu)? {
                return Ok(Some(user));
            }
        }

        Ok(None)
    }
    async fn register(&self, username: &str, password: &str) -> Result<User, Error> {
        todo!()
    }
    async fn get_zettels(
        &self,
        user: UserId,
        search: SearchOpts<'_>,
    ) -> Result<Vec<ZettelHeader>, Error> {
        todo!()
    }
    async fn get_zettel(&self, user: UserId, id: ZettelId) -> Result<Zettel, Error> {
        todo!()
    }
    async fn get_zettel_by_url(&self, user: UserId, url: &str) -> Result<Option<Zettel>, Error> {
        todo!()
    }
    async fn update_zettel(&self, user: UserId, zettel: &mut Zettel) -> Result<(), Error> {
        todo!()
    }
    async fn set_user_last_visited_zettel(
        &self,
        user: UserId,
        zettel_id: Option<ZettelId>,
    ) -> Result<(), Error> {
        todo!()
    }
    async fn update_config(&self, config: &SystemConfig) -> Result<(), Error> {
        todo!()
    }
}

impl ConnectableStorage for Connection {
    type ConnectionArgs = String;

    fn connect<'a>(
        connection_args: Self::ConnectionArgs,
    ) -> LocalBoxFuture<'a, Result<(Self, SystemConfig), Error>> {
        async move {
            let pool = sqlx::PgPool::connect(&connection_args)
                .await
                .context(SqlxSnafu)?;
            let result = sqlx::query!("SELECT key, value FROM config")
                .fetch_all(&mut pool.acquire().await.context(SqlxSnafu)?)
                .await
                .context(SqlxSnafu)?;

            // Map them into a `serde_json::Map`
            let map: serde_json::Map<String, serde_json::Value> = result
                .into_iter()
                .map(|o| Ok((o.key, serde_json::from_str(&o.value)?)))
                .collect::<Result<_, _>>()
                .context(JsonSnafu)?;

            // Now we can deserialize this from a `serde_json::Value::Object(map)`
            let config =
                serde_json::from_value(serde_json::Value::Object(map)).context(JsonSnafu)?;

            Ok((Self { conn: pool }, config))
        }
        .boxed_local()
    }
}
