use std::sync::Arc;

use zettelkasten_shared::storage;

pub struct List {
    user: Arc<storage::User>,
}

impl List {
    pub fn new(user: Arc<storage::User>) -> Self {
        Self { user }
    }
    pub(crate) fn render(&mut self, _tui: &mut crate::Tui) -> super::Result<Option<Transition>> {
        Ok(Some(Transition::Pop))
    }
}

pub enum Transition {
    Pop,
}
