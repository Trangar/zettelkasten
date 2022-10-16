use crate::async_trait;
use futures::future::LocalBoxFuture;
use std::path::PathBuf;

pub type ZettelId = i64;
pub type UserId = i64;

#[async_trait]
pub trait Storage: Send + Sync {
    async fn login_single_user(&self) -> Result<User, Error>;
    async fn login(&self, username: &str, password: &str) -> Result<Option<User>, Error>;
    async fn register(&self, username: &str, password: &str) -> Result<Option<User>, Error>;
    async fn get_zettels(
        &self,
        user: UserId,
        search: Option<SearchOpts>,
    ) -> Result<Vec<ZettelHeader>, Error>;
    async fn get_zettel(&self, user: UserId, id: ZettelId) -> Result<Vec<Zettel>, Error>;
    async fn get_zettel_by_url(&self, user: UserId, url: &str) -> Result<Option<Zettel>, Error>;
    async fn update_zettel(&self, user: UserId, zettel: &Zettel) -> Result<(), Error>;
}

pub trait ConnectableStorage: Storage + Sized {
    type ConnectionArgs;
    fn connect<'a>(
        connection_args: Self::ConnectionArgs,
    ) -> LocalBoxFuture<'a, Result<(Self, SystemConfig), Error>>;
}

#[derive(sqlx::FromRow)]
pub struct User {
    pub id: UserId,
    pub name: String,
    pub password: String,
    pub last_visited_zettel: Option<ZettelId>,
}

pub struct SearchOpts {}

#[derive(sqlx::FromRow)]
pub struct ZettelHeader {
    pub id: ZettelId,
    pub url: String,
    pub title: String,
    pub highlight_text: Option<String>,
}

#[derive(sqlx::FromRow)]
pub struct Zettel {
    pub id: ZettelId,
    pub url: String,
    pub title: String,
    pub body: String,
    pub attachments: Vec<Box<dyn Attachment>>,
}

#[async_trait]
pub trait Attachment: Send + Sync {
    fn name(&self) -> &str;
    async fn load(&self) -> Vec<u8>;
}

#[derive(Debug)]
pub enum Error {
    Sqlx { inner: sqlx::Error },
    SqlxMigrate { inner: sqlx::migrate::MigrateError },
    Bcrypt { inner: bcrypt::BcryptError },
    JsonDeserializeError { inner: serde_json::Error },

    SingleUserNotFound,
}

impl From<sqlx::Error> for Error {
    fn from(inner: sqlx::Error) -> Self {
        Self::Sqlx { inner }
    }
}

impl From<sqlx::migrate::MigrateError> for Error {
    fn from(inner: sqlx::migrate::MigrateError) -> Self {
        Self::SqlxMigrate { inner }
    }
}

impl From<bcrypt::BcryptError> for Error {
    fn from(inner: bcrypt::BcryptError) -> Self {
        Self::Bcrypt { inner }
    }
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
