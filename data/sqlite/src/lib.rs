mod regexp;

use async_lock::Mutex;
use snafu::ResultExt;
use sqlx::{ConnectOptions as _, Row};
use std::{str::FromStr, sync::Arc};
use storage::{
    BcryptSnafu, ConnectableStorage, Error, InvalidRegexSnafu, JsonSnafu, SearchOpts,
    SqlxMigrateSnafu, SqlxSnafu, Storage, SystemConfig, User, UserId, Zettel, ZettelHeader,
    ZettelId,
};
use zettelkasten_shared::{
    futures::{future::LocalBoxFuture, FutureExt},
    storage,
};

pub struct Connection {
    conn: Arc<Mutex<sqlx::SqliteConnection>>,
}

#[zettelkasten_shared::async_trait]
impl Storage for Connection {
    async fn user_count(&self) -> Result<u64, Error> {
        let query = sqlx::query!("SELECT COUNT(*) as count FROM users");
        query
            .fetch_one(&mut *self.conn.lock().await)
            .await
            .map(|result| result.count as u64)
            .context(SqlxSnafu)
    }

    async fn login_single_user(&self) -> Result<User, Error> {
        let query = sqlx::query_as!(
            User,
            "SELECT user_id as id, username as name, password, last_visited_zettel FROM users",
        );
        query
            .fetch_one(&mut *self.conn.lock().await)
            .await
            .context(SqlxSnafu)
    }

    async fn login(&self, username: &str, password: &str) -> Result<Option<User>, Error> {
        let query = sqlx::query_as!(
            User,
            "SELECT user_id as id, username as name, password, last_visited_zettel FROM users WHERE username = ?",
            username,
        );
        let user = query
            .fetch_optional(&mut *self.conn.lock().await)
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
        let mut conn = self.conn.lock().await;
        let query = sqlx::query!(
            "SELECT COUNT(user_id) as count FROM users WHERE username = ?",
            username
        );
        let user = query.fetch_one(&mut *conn).await.context(SqlxSnafu)?;

        if user.count != 0 {
            return Err(Error::UserAlreadyExists);
        }

        let password = bcrypt::hash(password, bcrypt::DEFAULT_COST).context(BcryptSnafu)?;

        let query = sqlx::query_as!(
            User,
            r#"INSERT INTO users (username, password) VALUES (?, ?) RETURNING user_id as "id!", username as "name!", password as "password!", last_visited_zettel"#,
            username,
            password
        );
        let user = query.fetch_one(&mut *conn).await.context(SqlxSnafu)?;

        Ok(user)
    }

    async fn get_zettels(
        &self,
        user: UserId,
        search: SearchOpts<'_>,
    ) -> Result<Vec<ZettelHeader>, Error> {
        if search.list_all {
            let results = sqlx::query(
                "SELECT zettel_id as id, path FROM zettel WHERE user_id = ? ORDER BY path ASC",
            )
            .bind(user)
            .fetch_all(&mut *self.conn.lock().await)
            .await
            .context(SqlxSnafu)?;

            Ok(results
                .into_iter()
                .map(|row| ZettelHeader {
                    id: row.get(0),
                    path: row.get(1),
                    highlight_text: None,
                })
                .collect())
        } else if !search.query.trim().is_empty() {
            let regex = regex::Regex::new(search.query).context(InvalidRegexSnafu)?;

            // we're using REGEXP here, and sql does not understand that because we need to register it ourselves
            // therefor we can't use `sqlx::query_as!()` and instead have to do it manually.
            let query = sqlx::query(
                "SELECT zettel_id as id, path, body FROM zettel WHERE user_id = ? AND (body REGEXP ? OR path REGEXP ?)",
            ).bind(user)
            .bind(search.query)
            .bind(search.query);
            let results = query
                .fetch_all(&mut *self.conn.lock().await)
                .await
                .context(SqlxSnafu)?;

            Ok(results
                .into_iter()
                .map(|row| {
                    let body: String = row.get(2);
                    let highlight_text = if let Some(m) = regex.find(&body) {
                        let start = if m.start() < 10 { 0 } else { m.start() - 10 };
                        let end = if m.end() + 10 >= body.len() {
                            body.len()
                        } else {
                            m.end() + 10
                        };
                        Some(body[start..end].to_owned())
                    } else {
                        None
                    };
                    ZettelHeader {
                        id: row.get(0),
                        path: row.get(1),
                        highlight_text,
                    }
                })
                .collect())
        } else {
            Err(Error::InvalidSearchOpts)
        }
    }

    async fn get_zettel(&self, user: UserId, id: ZettelId) -> Result<Zettel, Error> {
        let mut conn = self.conn.lock().await;
        let conn = &mut *conn;
        let result = sqlx::query!(
            "SELECT zettel_id, path, body FROM zettel WHERE user_id = ? AND zettel_id = ?",
            user,
            id
        )
        .fetch_one(conn)
        .await
        .context(SqlxSnafu)?;

        Ok(Zettel {
            id: result.zettel_id,
            path: result.path,
            body: result.body,
            attachments: Vec::new(),
        })
    }

    async fn get_zettel_by_url(&self, user: UserId, path: &str) -> Result<Option<Zettel>, Error> {
        let mut conn = self.conn.lock().await;
        let conn = &mut *conn;
        let result = match sqlx::query!(
            "SELECT zettel_id, path, body FROM zettel WHERE user_id = ? AND path = ?",
            user,
            path
        )
        .fetch_optional(conn)
        .await
        .context(SqlxSnafu)?
        {
            Some(zettel) => zettel,
            None => return Ok(None),
        };

        Ok(Some(Zettel {
            id: result.zettel_id,
            path: result.path,
            body: result.body,
            attachments: Vec::new(),
        }))
    }

    async fn update_zettel(&self, user: UserId, zettel: &mut Zettel) -> Result<(), Error> {
        let mut conn = self.conn.lock().await;
        if zettel.id == 0 {
            let query = sqlx::query!(
                r#"INSERT INTO zettel
                (user_id, path, body, created_on, last_modified_on)
                VALUES
                (?, ?, ?, datetime(), datetime())
                "#,
                user,
                zettel.path,
                zettel.body,
            );
            let id = query
                .execute(&mut *conn)
                .await
                .context(SqlxSnafu)?
                .last_insert_rowid();
            zettel.id = id;
            sqlx::query!(
                "UPDATE users SET last_visited_zettel = ? WHERE user_id = ?",
                zettel.id,
                user
            )
            .execute(&mut *conn)
            .await
            .context(SqlxSnafu)?;

            Ok(())
        } else {
            sqlx::query!(
                r#"UPDATE zettel
                SET path = ?, body = ?, last_modified_on = datetime()
                WHERE zettel_id = ?"#,
                zettel.path,
                zettel.body,
                zettel.id,
            )
            .execute(&mut *conn)
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
        let query = sqlx::query!(
            "UPDATE users SET last_visited_zettel = ? WHERE user_id = ?",
            zettel_id,
            user
        );
        query
            .execute(&mut *self.conn.lock().await)
            .await
            .context(SqlxSnafu)?;
        Ok(())
    }

    async fn update_config(&self, config: &SystemConfig) -> Result<(), Error> {
        let values = if let serde_json::Value::Object(map) =
            serde_json::to_value(config).context(JsonSnafu)?
        {
            map
        } else {
            panic!("SystemConfig did not serialize to an object")
        };

        let mut conn = self.conn.lock().await;
        for (key, value) in values {
            sqlx::query!("UPDATE config SET value = ? WHERE key = ?", value, key)
                .execute(&mut *conn)
                .await
                .context(SqlxSnafu)?;
        }
        Ok(())
    }
}

impl Connection {
    async fn load_config(&self) -> Result<SystemConfig, Error> {
        // load all the key-value entries from the database
        let result = sqlx::query!("SELECT key, value FROM config")
            .fetch_all(&mut *self.conn.lock().await)
            .await
            .context(SqlxSnafu)?;

        // Map them into a `serde_json::Map`
        let map: serde_json::Map<String, serde_json::Value> = result
            .into_iter()
            .map(|o| Ok((o.key, serde_json::from_str(&o.value)?)))
            .collect::<Result<_, _>>()
            .context(JsonSnafu)?;

        // Now we can deserialize this from a `serde_json::Value::Object(map)`
        serde_json::from_value(serde_json::Value::Object(map)).context(JsonSnafu)
    }
}

impl ConnectableStorage for Connection {
    type ConnectionArgs = String;

    fn connect<'a>(
        connection_args: Self::ConnectionArgs,
    ) -> LocalBoxFuture<'a, Result<(Self, SystemConfig), Error>> {
        async move {
            println!("Opening {connection_args:?}");
            let mut connection = sqlx::sqlite::SqliteConnectOptions::from_str(&connection_args)
                .expect("Invalid SQLite connection string")
                .create_if_missing(true)
                .connect()
                .await
                .context(SqlxSnafu)?;
            sqlx::migrate!()
                .run(&mut connection)
                .await
                .context(SqlxMigrateSnafu)?;

            regexp::register(&mut connection).await;

            let connection = Connection {
                conn: Arc::new(Mutex::new(connection)),
            };

            let config = connection.load_config().await?;

            Ok((connection, config))
        }
        .boxed_local()
    }
}

#[cfg(test)]
async fn test_db() -> (Connection, User) {
    let (conn, _config) = Connection::connect(String::from(":memory:"))
        .await
        .expect("Could not open db");
    let user = conn.register("test", "test").await.unwrap();
    conn.update_zettel(
        user.id,
        &mut Zettel {
            path: "home".to_owned(),
            ..Default::default()
        },
    )
    .await
    .unwrap();

    (conn, user)
}

#[test]
fn test_search_in_path() {
    zettelkasten_shared::block_on(async {
        let (db, user) = test_db().await;
        let result = db
            .get_zettels(
                user.id,
                SearchOpts {
                    query: "home",
                    ..Default::default()
                },
            )
            .await
            .unwrap();
        assert_eq!(result.len(), 1);
        let home = &result[0];
        assert_eq!(home.id, 1);
        assert_eq!(home.path, "home");
        assert_eq!(home.highlight_text, None);
    });
}
