pub mod login;
pub mod register;
pub mod zettel;

use super::Web;
use crate::User;
use tide::{Redirect, Request, Result};

pub async fn get_index(req: Request<Web>) -> Result {
    match User::from_req(&req).await {
        Ok(user) => {
            let zettel = user.load_last_visited_zettel(&req.state().storage).await;
            crate::render_template(zettel::Zettel {
                user: &user,
                body: &zettel.body,
                path: &zettel.path,
            })
        }
        Err(_) => Ok(Redirect::new("/sys:login").into()),
    }
}

pub async fn get_config(req: Request<Web>) -> Result {
    let Ok(_user) = User::from_req(&req).await else {
        return login::redirect();
    };
    Ok("config".into())
}

pub async fn post_config(req: Request<Web>) -> Result {
    let Ok(_user) = User::from_req(&req).await else {
        return login::redirect();
    };
    Ok("config".into())
}
