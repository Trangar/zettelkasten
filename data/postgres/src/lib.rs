use snafu::ResultExt;
use zettelkasten_shared::{
    async_trait,
    futures::{future::LocalBoxFuture, FutureExt},
    storage::{
        BcryptSnafu, ConnectableStorage, Error, InvalidRegexSnafu, JsonSnafu, SearchOpts,
        SqlxSnafu, Storage, SystemConfig, User, UserId, Zettel, ZettelHeader, ZettelId,
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
        let mut conn = self.conn.acquire().await.context(SqlxSnafu)?;

        let password = bcrypt::hash(password, bcrypt::DEFAULT_COST).context(BcryptSnafu)?;

        sqlx::query_as!(
            User,
            r#"
            INSERT INTO users (username, password)
            VALUES ($1, $2)
            RETURNING user_id as id, username as name, password, last_visited_zettel
            "#,
            username,
            password
        )
        .fetch_one(&mut conn)
        .await
        .context(SqlxSnafu)
    }

    async fn get_user_by_id(&self, id: UserId) -> Result<User, Error> {
        let mut conn = self.conn.acquire().await.context(SqlxSnafu)?;
        let query = sqlx::query_as!(
            User,
            "SELECT user_id as id, username as name, password, last_visited_zettel FROM users WHERE user_id = $1",
            id
        );

        query.fetch_one(&mut conn).await.context(SqlxSnafu)
    }

    async fn get_zettels(
        &self,
        user: UserId,
        search: SearchOpts<'_>,
    ) -> Result<Vec<ZettelHeader>, Error> {
        let mut conn = self.conn.acquire().await.context(SqlxSnafu)?;

        if search.list_all {
            let results = sqlx::query!(
                "SELECT zettel_id, path FROM zettel WHERE user_id = $1",
                user
            )
            .fetch_all(&mut conn)
            .await
            .context(SqlxSnafu)?;
            Ok(results
                .into_iter()
                .map(|r| ZettelHeader {
                    id: r.zettel_id,
                    path: r.path,
                    highlight_text: None,
                })
                .collect())
        } else if !search.query.is_empty() {
            let regex = regex::Regex::new(search.query).context(InvalidRegexSnafu)?;

            let results = sqlx::query!(
                "SELECT zettel_id, path, body FROM zettel WHERE user_id = $1 AND (path ~ $2 OR body ~ $2)",
                user,
                search.query
            )
            .fetch_all(&mut conn)
            .await
            .context(SqlxSnafu)?;
            Ok(results
                .into_iter()
                .map(|row| {
                    let highlight_text = if let Some(m) = regex.find(&row.body) {
                        let start = if m.start() < 10 { 0 } else { m.start() - 10 };
                        let end = if m.end() + 10 >= row.body.len() {
                            row.body.len()
                        } else {
                            m.end() + 10
                        };
                        Some(row.body[start..end].to_owned())
                    } else {
                        None
                    };
                    ZettelHeader {
                        id: row.zettel_id,
                        path: row.path,
                        highlight_text,
                    }
                })
                .collect())
        } else {
            Err(Error::InvalidSearchOpts)
        }
    }

    async fn get_zettel(&self, user: UserId, id: ZettelId) -> Result<Zettel, Error> {
        let mut conn = self.conn.acquire().await.context(SqlxSnafu)?;

        let zettel = sqlx::query!(
            "SELECT zettel_id, path, body FROM zettel WHERE zettel_id = $1 AND user_id = $2",
            id,
            user
        )
        .fetch_one(&mut conn)
        .await
        .context(SqlxSnafu)?;
        Ok(Zettel {
            id: zettel.zettel_id,
            path: zettel.path,
            body: zettel.body,
            attachments: Vec::new(),
        })
    }

    async fn get_zettel_by_url(&self, user: UserId, url: &str) -> Result<Option<Zettel>, Error> {
        let mut conn = self.conn.acquire().await.context(SqlxSnafu)?;

        if let Some(zettel) = sqlx::query!(
            "SELECT zettel_id, path, body FROM zettel WHERE user_id = $1 AND path = $2",
            user,
            url,
        )
        .fetch_optional(&mut conn)
        .await
        .context(SqlxSnafu)?
        {
            Ok(Some(Zettel {
                id: zettel.zettel_id,
                path: zettel.path,
                body: zettel.body,
                attachments: Vec::new(),
            }))
        } else {
            Ok(None)
        }
    }

    async fn update_zettel(&self, user: UserId, zettel: &mut Zettel) -> Result<(), Error> {
        let mut conn = self.conn.acquire().await.context(SqlxSnafu)?;

        if zettel.id == 0 {
            let result = sqlx::query!(
                r#"
                INSERT INTO zettel (user_id, path, body)
                VALUES ($1, $2, $3)
                RETURNING zettel_id
                "#,
                user,
                zettel.path,
                zettel.body
            )
            .fetch_one(&mut conn)
            .await
            .context(SqlxSnafu)?;
            zettel.id = result.zettel_id;
            Ok(())
        } else {
            sqlx::query!(
                "UPDATE zettel SET body = $1, PATH = $2 WHERE zettel_id = $3 AND user_id = $4",
                zettel.body,
                zettel.path,
                zettel.id,
                user
            )
            .execute(&mut conn)
            .await
            .context(SqlxSnafu)?;
            Ok(())
        }
    }

    async fn set_user_last_visited_zettel(
        &self,
        user: UserId,
        zettel_id: Option<ZettelId>,
    ) -> Result<(), Error> {
        let mut conn = self.conn.acquire().await.context(SqlxSnafu)?;

        sqlx::query!(
            "UPDATE users SET last_visited_zettel = $1 WHERE user_id = $2",
            zettel_id,
            user,
        )
        .execute(&mut conn)
        .await
        .context(SqlxSnafu)?;
        Ok(())
    }

    async fn update_config(&self, config: &SystemConfig) -> Result<(), Error> {
        let mut conn = self.conn.acquire().await.context(SqlxSnafu)?;

        let serde_json::Value::Object(values) = serde_json::to_value(config).context(JsonSnafu)? else {
            panic!("SystemConfig did not serialize to an object");
        };

        for (key, value) in values {
            let str_value = serde_json::to_string(&value).context(JsonSnafu)?;
            sqlx::query!(
                "UPDATE config SET value = $1 WHERE key = $2",
                str_value,
                key
            )
            .execute(&mut conn)
            .await
            .context(SqlxSnafu)?;
        }
        Ok(())
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
