use super::Web;
use crate::req;
use tide::{Request, Result};

pub async fn get(req: Request<Web>) -> Result {
    let Ok(user) = crate::req::User::from_req(&req).await else {
        return super::login::redirect();
    };
    let path = req.url().path();
    let zettel = match req.state().storage.get_zettel_by_url(user.id(), path).await {
        Ok(Some(zettel)) => zettel,
        Ok(None) if path == "/" => user.load_last_visited_zettel(&req.state().storage).await,
        Ok(None) => crate::req::default_zettel(),
        Err(e) => {
            return Ok(e.to_string().into());
        }
    };

    crate::req::render_template(Zettel {
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
    pub user: &'a req::User,
    pub path: &'a str,
    pub body: &'a str,
}

mod filters {
    pub fn render(s: &str) -> ::askama::Result<String> {
        Ok(markdown::to_html(s).replace("\\n", "<br />"))
    }
}
