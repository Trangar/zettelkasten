use zettelkasten_shared::storage;

#[derive(Default)]
pub struct NotLoggedIn {
    pub username: String,
    pub password: String,
    pub error: Option<LoginError>,
}

pub enum LoginError {
    Storage(storage::Error),
}
