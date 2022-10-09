use crate::async_trait;
use futures::future::LocalBoxFuture;
use std::path::PathBuf;

pub type PageId = i64;
pub type UserId = i64;

#[async_trait]
pub trait Storage: Send + Sync {
    async fn login(&self, username: &str, password: &str) -> Result<Option<User>, Error>;
    async fn register(&self, username: &str, password: &str) -> Result<Option<User>, Error>;
    async fn get_pages(
        &self,
        user: UserId,
        search: Option<SearchOpts>,
    ) -> Result<Vec<PageHeader>, Error>;
    async fn get_page(&self, user: UserId, id: PageId) -> Result<Vec<Page>, Error>;
    async fn get_page_by_url(&self, user: UserId, url: &str) -> Result<Option<Page>, Error>;
    async fn update_page(&self, user: UserId, page: &Page) -> Result<(), Error>;
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
}

pub struct SearchOpts {}

#[derive(sqlx::FromRow)]
pub struct PageHeader {
    pub id: PageId,
    pub url: String,
    pub title: String,
    pub highlight_text: Option<String>,
}

#[derive(sqlx::FromRow)]
pub struct Page {
    pub id: PageId,
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

#[derive(Default, Clone)]
pub struct SystemConfig {
    pub user_mode: UserMode,
    pub terminal_editor: Option<PathBuf>,
}

#[derive(Default, Clone, Copy)]
pub enum UserMode {
    SingleUserAutoLogin,
    #[default]
    SingleUserManualLogin,
    MultiUser,
}
