mod filters;

use super::Web;
use tide::{Request, Result};

pub async fn get(req: Request<Web>) -> Result {
    let Ok(user) = crate::User::from_req(&req).await else {
        return super::login::redirect();
    };
    let path = req.url().path();
    let zettel = match req.state().storage.get_zettel_by_url(user.id(), path).await {
        Ok(Some(zettel)) => zettel,
        Ok(None) if path == "/" => user.load_last_visited_zettel(&req.state().storage).await,
        Ok(None) => crate::default_zettel(),
        Err(e) => {
            return Ok(e.to_string().into());
        }
    };

    crate::render_template(Zettel {
        user: &user,
        path: &zettel.path,
        body: &zettel.body,
    })
}

pub async fn post(_req: Request<Web>) -> Result {
    todo!()
}

#[derive(askama::Template)]
#[template(path = "zettel.html")]
pub struct Zettel<'a> {
    pub user: &'a crate::User,
    pub path: &'a str,
    pub body: &'a str,
}
