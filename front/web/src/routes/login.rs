use super::Web;
use snafu::ResultExt;
use tide::{Redirect, Request, Result};

pub async fn get(req: Request<Web>) -> Result {
    crate::render_template(Login {
        can_register: req.state().can_register().await,
        username: "",
        error: None,
    })
}

pub async fn post(mut req: Request<Web>) -> Result {
    let login = req.body_form::<PostLogin>().await?;
    match req
        .state()
        .storage
        .login(&login.username, &login.password)
        .await
        .context(crate::StorageSnafu)
    {
        Ok(Some(user)) => {
            let user = crate::User::new_session(user, req.session_mut());
            Ok(user.redirect_to_latest_zettel(&req.state().storage).await)
        }
        Ok(None) => crate::render_template(Login {
            username: &login.username,
            error: Some(crate::Error::UserNotFound),
            can_register: req.state().can_register().await,
        }),
        Err(e) => crate::render_template(Login {
            username: &login.username,
            error: Some(e),
            can_register: req.state().can_register().await,
        }),
    }
}

#[derive(askama::Template)]
#[template(path = "login.html")]
struct Login<'a> {
    error: Option<crate::Error>,
    username: &'a str,
    can_register: bool,
}

#[derive(serde::Deserialize)]
struct PostLogin {
    username: String,
    password: String,
}

pub(crate) fn redirect() -> tide::Result {
    Ok(Redirect::new("/sys:login").into())
}
