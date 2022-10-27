use super::utils::ParsedZettel;
use crossterm::event::{Event, KeyCode};
use snafu::ResultExt;
use tui::{
    layout::Rect,
    text::Text,
    widgets::{Block, Borders, Paragraph},
};
use zettelkasten_shared::storage;

#[derive(Clone)]
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

const DISALLOWED_CHARS: &[char] = &['q', 'e', 'c', 'f', 's', 'l'];

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
        let current_zettel = self.load_zettel(tui)?;
        let mut render_link_input: Option<String> = None;
        let mut rendered_zettel = None;
        loop {
            let title = &current_zettel.title;
            let zettel = rendered_zettel.get_or_insert_with(|| {
                ParsedZettel::parse(
                    current_zettel,
                    DISALLOWED_CHARS,
                    render_link_input.is_some(),
                    Default::default(),
                )
            });

            let body = Paragraph::new(Text {
                lines: zettel.lines.clone(),
            })
            .block(Block::default().borders(Borders::ALL).title(title.as_str()));

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
                    KeyCode::Char('e') => return Ok(Some(Transition::Edit)),
                    KeyCode::Char('c') => return Ok(Some(Transition::OpenConfig)),
                    KeyCode::Char('f') => {
                        if render_link_input.is_some() {
                            render_link_input = None;
                        } else {
                            render_link_input = Some(String::with_capacity(zettel.link_char_size));
                        }
                        rendered_zettel.take();
                        continue;
                    }
                    KeyCode::Char('s') => return Err(super::ViewError::NotImplemented),
                    KeyCode::Char('l') => return Ok(Some(Transition::Logout)),
                    _ => {}
                }

                if let Some(filter) = &mut render_link_input {
                    if let KeyCode::Char(c) = key_event.code {
                        filter.push(c);
                        if filter.len() == zettel.link_char_size {
                            if let Some(link) = zettel.links.get(filter) {
                                return Ok(Some(Transition::NavigateTo {
                                    path: link.to_string(),
                                }));
                            }
                            render_link_input = None;
                        }
                    } else if let KeyCode::Backspace = key_event.code {
                        filter.pop();
                    } else if let KeyCode::Esc = key_event.code {
                        render_link_input = None;
                        rendered_zettel.take();
                    }
                }
            }
        }
    }
}

#[allow(dead_code)]
pub enum Transition {
    Exit,
    Logout,
    NavigateTo { path: String },
    OpenConfig,
    Edit,
}
