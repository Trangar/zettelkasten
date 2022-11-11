mod user;

pub use user::User;

use snafu::ResultExt;
use zettelkasten_shared::storage;

pub(crate) fn render_template(tmpl: impl askama::Template) -> tide::Result {
    let str = tmpl.render().context(crate::AskamaSnafu)?;
    let mut response = tide::Response::new(200);
    response.set_content_type("text/HTML");
    response.set_body(str);
    Ok(response)
}

pub fn default_zettel() -> storage::Zettel {
    storage::Zettel {
        path: "/home".to_string(),
        body: r#"Welcome to zettelkasten\n\n**bold**"#.to_string(),
        ..Default::default()
    }
}
