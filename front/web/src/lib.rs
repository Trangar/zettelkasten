mod routes;
mod user;

pub(crate) use user::User;

use snafu::ResultExt;
use std::{net::SocketAddr, sync::Arc};
use tide::{log::warn, sessions::SessionMiddleware};
use zettelkasten_shared::{async_trait, storage};

#[cfg(not(feature = "runtime-async-std"))]
compile_error!("zettelkasten_web requires feature `runtime-async-std`");

#[derive(Clone)]
pub struct Web {
    pub(crate) system_config: storage::SystemConfig,
    pub(crate) storage: Arc<dyn storage::Storage>,
}
impl Web {
    async fn can_register(&self) -> bool {
        self.storage.user_count().await.unwrap_or(0) == 0
            || matches!(self.system_config.user_mode, storage::UserMode::MultiUser)
    }
}

#[async_trait]
impl zettelkasten_shared::Front for Web {
    type Config = ServerConfig;
    async fn run(
        config: Self::Config,
        system_config: storage::SystemConfig,
        storage: Arc<dyn storage::Storage>,
    ) {
        let mut app = tide::with_state(Web {
            system_config,
            storage,
        });
        app.with(SessionMiddleware::new(
            tide::sessions::CookieStore::new(),
            config.session_secret.as_deref().unwrap_or_else(|| {
                let warning =
                    "Missing session secret. This should be set in a production environment.";
                warn!("{warning}");
                warning.as_bytes()
            }),
        ));
        if std::path::Path::new("static").exists() {
            app.at("/static")
                .serve_dir("static")
                .expect("Could not serve static dir");
        } else if cfg!(debug_assertions) && std::path::Path::new("front/web/static").exists() {
            app.at("/static")
                .serve_dir("front/web/static")
                .expect("Could not serve static dir");
        } else {
            panic!("`static` directory not found");
        }
        app.at("/").get(routes::get_index);
        app.at("/sys:login")
            .get(routes::login::get)
            .post(routes::login::post);
        app.at("/sys:register")
            .get(routes::register::get)
            .post(routes::register::post);
        app.at("/sys:config")
            .get(routes::get_config)
            .post(routes::post_config);
        app.at("/*")
            .get(routes::zettel::get)
            .post(routes::zettel::post);
        app.listen(config.bind_addr)
            .await
            .expect("Could not bind to addr");
    }
}

pub struct ServerConfig {
    pub bind_addr: SocketAddr,
    pub session_secret: Option<Vec<u8>>,
}

#[derive(snafu::Snafu, Debug)]
#[snafu(visibility(pub(crate)))]
pub enum Error {
    #[snafu(display("Internal storage issue: {source:?}"))]
    Storage { source: storage::Error },
    #[snafu(display("You are not logged in"))]
    NotLoggedIn,
    #[snafu(display("User not found"))]
    UserNotFound,
    #[snafu(display("This system does not permit registering a new account"))]
    CannotRegister,
    #[snafu(display("Passwords do not match"))]
    PasswordMismatch,
    #[snafu(display("Internal rendering issue"))]
    Askama { source: askama::Error },
}

pub type Result<T = ()> = std::result::Result<T, Error>;

pub(crate) fn render_template(tmpl: impl askama::Template) -> tide::Result {
    let str = tmpl.render().context(crate::AskamaSnafu)?;
    let mut response = tide::Response::new(200);
    response.set_content_type("text/HTML");
    response.set_body(str);
    Ok(response)
}

pub(crate) fn default_zettel() -> storage::Zettel {
    storage::Zettel {
        path: "/home".to_string(),
        body: r#"Welcome to zettelkasten

**bold**

[sys:config]

[config](sys:config)"#
            .to_string(),
        ..Default::default()
    }
}
