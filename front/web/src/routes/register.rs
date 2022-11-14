use super::Web;
use snafu::ResultExt;
use tide::{Request, Result};

pub async fn get(_req: Request<Web>) -> Result {
    crate::render_template(Register::default())
}

pub async fn post(mut req: Request<Web>) -> Result {
    let register = req.body_form::<PostRegister>().await?;
    if !req.state().can_register().await {
        return crate::render_template(Register {
            error: Some(crate::Error::CannotRegister),
            username: "",
        });
    }
    if register.password != register.repeat_password {
        return crate::render_template(Register {
            error: Some(crate::Error::PasswordMismatch),
            username: &register.username,
        });
    }
    match req
        .state()
        .storage
        .register(&register.username, &register.password)
        .await
        .context(crate::StorageSnafu)
    {
        Ok(user) => {
            let user = crate::User::new_session(user, req.session_mut());
            Ok(user.redirect_to_latest_zettel(&req.state().storage).await)
        }
        Err(e) => crate::render_template(Register {
            error: Some(e),
            username: &register.username,
        }),
    }
}

#[derive(askama::Template, Default)]
#[template(path = "register.html")]
struct Register<'a> {
    error: Option<crate::Error>,
    username: &'a str,
}

#[derive(serde::Deserialize)]
struct PostRegister {
    username: String,
    password: String,
    repeat_password: String,
}
