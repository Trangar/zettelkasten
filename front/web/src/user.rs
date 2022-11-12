use std::sync::Arc;

use tide::Redirect;
use zettelkasten_shared::storage;

pub struct User {
    user: storage::User,
}

impl User {
    pub async fn from_req(req: &tide::Request<crate::Web>) -> crate::Result<Self> {
        let session = req.session();
        let Some(user_id) = session.get::<storage::UserId>("user_id") else {
            return Err(crate::Error::NotLoggedIn)
        };

        match req.state().storage.get_user_by_id(user_id).await {
            Ok(user) => Ok(User { user }),
            Err(_) => Err(crate::Error::UserNotFound),
        }
    }

    pub(crate) async fn load_last_visited_zettel(
        &self,
        storage: &Arc<dyn storage::Storage>,
    ) -> storage::Zettel {
        if let Some(zettel_id) = self.user.last_visited_zettel {
            if let Ok(zettel) = storage.get_zettel(self.user.id, zettel_id).await {
                return zettel;
            }
        }
        super::default_zettel()
    }

    pub(crate) fn new_session(user: storage::User, session: &mut tide::sessions::Session) -> Self {
        session.insert("user_id", user.id).unwrap();
        Self { user }
    }

    pub(crate) async fn redirect_to_latest_zettel(
        &self,
        storage: &Arc<dyn storage::Storage>,
    ) -> tide::Response {
        if let Some(zettel_id) = self.user.last_visited_zettel {
            if let Ok(zettel) = storage.get_zettel(self.user.id, zettel_id).await {
                return Redirect::new(format!("/{}", zettel.path)).into();
            }
        }
        Redirect::new("/").into()
    }

    pub(crate) fn id(&self) -> i64 {
        self.user.id
    }
}
