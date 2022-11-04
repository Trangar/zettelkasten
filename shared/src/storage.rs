use crate::async_trait;
use futures::future::LocalBoxFuture;
use std::{path::PathBuf, sync::Arc};

pub type ZettelId = i64;
pub type UserId = i64;

#[async_trait]
pub trait Storage: Send + Sync {
    async fn user_count(&self) -> Result<u64, Error>;
    async fn login_single_user(&self) -> Result<User, Error>;
    async fn login(&self, username: &str, password: &str) -> Result<Option<User>, Error>;
    async fn register(&self, username: &str, password: &str) -> Result<User, Error>;
    async fn get_zettels(
        &self,
        user: UserId,
        search: SearchOpts<'_>,
    ) -> Result<Vec<ZettelHeader>, Error>;
    async fn get_zettel(&self, user: UserId, id: ZettelId) -> Result<Zettel, Error>;
    async fn get_zettel_by_url(&self, user: UserId, url: &str) -> Result<Option<Zettel>, Error>;
    async fn update_zettel(&self, user: UserId, zettel: &mut Zettel) -> Result<(), Error>;
    async fn set_user_last_visited_zettel(
        &self,
        user: UserId,
        zettel_id: Option<ZettelId>,
    ) -> Result<(), Error>;
    async fn update_config(&self, config: &SystemConfig) -> Result<(), Error>;
}

pub trait ConnectableStorage: Storage + Sized {
    type ConnectionArgs;
    fn connect<'a>(
        connection_args: Self::ConnectionArgs,
    ) -> LocalBoxFuture<'a, Result<(Self, SystemConfig), Error>>;
}

#[derive(sqlx::FromRow, Clone)]
pub struct User {
    pub id: UserId,
    pub name: String,
    pub password: String,
    pub last_visited_zettel: Option<ZettelId>,
}

pub struct SearchOpts<'a> {
    pub query: &'a str,
}

#[derive(sqlx::FromRow)]
pub struct ZettelHeader {
    pub id: ZettelId,
    pub path: String,
    pub highlight_text: Option<String>,
}

#[derive(sqlx::FromRow, Default, Clone, custom_debug::Debug)]
pub struct Zettel {
    pub id: ZettelId,
    pub path: String,
    pub body: String,
    #[debug(skip)]
    pub attachments: Vec<Arc<dyn Attachment>>,
}

#[async_trait]
pub trait Attachment: Send + Sync {
    fn name(&self) -> &str;
    async fn load(&self) -> Vec<u8>;
}

#[derive(Debug, snafu::Snafu)]
#[snafu(visibility(pub))]
pub enum Error {
    Bcrypt { source: bcrypt::BcryptError },
    Json { source: serde_json::Error },
    Sqlx { source: sqlx::Error },
    SqlxMigrate { source: sqlx::migrate::MigrateError },
    InvalidRegex { source: regex::Error },

    SingleUserNotFound,
    UserAlreadyExists,
}

#[derive(Default, Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct SystemConfig {
    #[serde(default)]
    pub user_mode: UserMode,
    #[serde(default)]
    pub terminal_editor: Option<PathBuf>,
}

#[derive(Default, Debug, Clone, Copy, serde::Deserialize, serde::Serialize)]
pub enum UserMode {
    SingleUserAutoLogin,
    #[default]
    SingleUserManualLogin,
    MultiUser,
}
