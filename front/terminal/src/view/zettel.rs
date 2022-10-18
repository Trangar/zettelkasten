use crossterm::event::{Event, KeyCode};
use snafu::ResultExt;
use tui::{
    layout::Rect,
    text::{Spans, Text},
    widgets::{Block, Borders, Paragraph},
};
use zettelkasten_shared::storage;

pub struct Zettel {
    pub user: storage::User,
    pub zettel: Option<storage::Zettel>,
}

impl From<storage::User> for Zettel {
    fn from(user: storage::User) -> Self {
        Self { user, zettel: None }
    }
}

const ENTRY_TEXT: &str = r#"Welcome to Zettelkasten

You can see the available controls at the bottom of the page. If you are an admin, make sure to check out the config page (`C`).

- Q: Exit zettelkasten
- E: Edit the current page
- C: Open up the [system config](sys:config)
- F: Follow a link on the current page
  - Links are marked by `[name]` or `[name](path)` (only the `name` will be rendered)
- S: Search in all zettels
- L: Log out
"#;

impl Zettel {
    fn load_zettel(&mut self, tui: &crate::Tui) -> super::Result<&storage::Zettel> {
        // Rust is having some fuckery here, if we use the normal method, we get lifetime errors:
        // if let Some(zettel) = self.zettel.as_ref() {
        // so instead we have to check for `.is_some()` and `unwrap()`
        if self.zettel.is_some() {
            return Ok(self.zettel.as_ref().expect("unreachable"));
        } else if let Some(id) = self.user.last_visited_zettel {
            let result = zettelkasten_shared::block_on(tui.storage.get_zettel(self.user.id, id));
            if let Ok(zettel) = result {
                Ok(self.zettel.insert(zettel))
            } else {
                self.user.last_visited_zettel = None;
                // make sure we reset the zettel for the next time around, as we failed to load one
                zettelkasten_shared::block_on(
                    tui.storage.set_user_last_visited_zettel(self.user.id, None),
                )
                .context(super::DatabaseSnafu)?;
                Err(super::ViewError::ZettelIdNotFound { id })
            }
        } else {
            Ok(self.zettel.insert(storage::Zettel {
                body: ENTRY_TEXT.into(),
                ..Default::default()
            }))
        }
    }

    pub(crate) fn render(&mut self, tui: &mut crate::Tui) -> super::Result<Option<Transition>> {
        let zettel = self.load_zettel(tui)?;
        loop {
            let body = Paragraph::new(Text {
                lines: render_zettel(zettel),
            })
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(zettel.title.as_str()),
            );

            let terminal_size = tui.terminal.size().context(super::TerminalSizeSnafu)?;

            let action = Paragraph::new(Text {
                lines: vec![
                    "Q: exit, E: edit, C: config, F: follow link, S: search, L: log out".into(),
                ],
            });
            tui.terminal
                .draw(|f| {
                    f.render_widget(
                        body,
                        Rect {
                            height: terminal_size.height - 1,
                            ..terminal_size
                        },
                    );
                    f.render_widget(
                        action,
                        Rect {
                            y: terminal_size.height - 1,
                            height: 1,
                            ..terminal_size
                        },
                    );
                })
                .context(super::RenderFrameSnafu)?;

            let event = crossterm::event::read().context(super::EventSnafu)?;
            if let Event::Key(key_event) = event {
                match key_event.code {
                    KeyCode::Char('q') => return Ok(Some(Transition::Exit)),
                    KeyCode::Char('e') => return Err(super::ViewError::NotImplemented),
                    KeyCode::Char('c') => return Err(super::ViewError::NotImplemented),
                    KeyCode::Char('f') => return Err(super::ViewError::NotImplemented),
                    KeyCode::Char('s') => return Err(super::ViewError::NotImplemented),
                    KeyCode::Char('l') => return Ok(Some(Transition::Logout)),
                    _ => {}
                }
            }
        }
    }
}

fn render_zettel(zettel: &storage::Zettel) -> Vec<Spans> {
    let mut result = Vec::new();

    for line in zettel.body.lines() {
        result.push(line.into());
    }
    result
}

#[allow(dead_code)]
pub enum Transition {
    Exit,
    Logout,
}
