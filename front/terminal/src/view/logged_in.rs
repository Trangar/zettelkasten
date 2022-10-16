use zettelkasten_shared::storage;

pub struct LoggedIn {
    pub user: storage::User,
}

impl From<storage::User> for LoggedIn {
    fn from(user: storage::User) -> Self {
        Self { user }
    }
}
impl LoggedIn {
    pub(crate) fn render(
        &self,
        _terminal: &mut crate::Terminal,
    ) -> super::Result<Option<Transition>> {
        todo!()
    }
}

pub enum Transition {
    Logout,
}
