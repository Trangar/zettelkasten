use std::sync::Arc;

use super::utils::ParsedZettel;
use crossterm::event::{Event, KeyCode};
use snafu::ResultExt;
use tui::{
    layout::Rect,
    text::Text,
    widgets::{Block, Borders, Paragraph},
};
use zettelkasten_shared::storage;

const ENTRY_TEXT: &str = r#"Welcome to Zettelkasten

You can see the available controls at the bottom of the page. If you are an admin, make sure to check out the config page (`C`).

- A: Show all paths
- C: Open up the [system config](sys:config)
- E: Edit the current page
- F: Follow a link on the current page
  - Links are marked by `[name]` or `[name](path)` (only the `name` will be rendered)
- L: Log out
- Q: Exit zettelkasten
- S: Search in all zettels
"#;

const DISALLOWED_CHARS: &[char] = &['a', 'c', 'e', 'f', 'l', 'q', 's'];

#[derive(Clone)]
pub struct Zettel {
    pub(super) user: Arc<storage::User>,
    pub(super) zettel: storage::Zettel,
}

impl Zettel {
    pub(crate) fn new_with_user(
        storage: &Arc<dyn storage::Storage>,
        user: Arc<storage::User>,
    ) -> Self {
        let zettel: Option<storage::Zettel> = user.last_visited_zettel.and_then(|zettel_id| {
            zettelkasten_shared::block_on(storage.get_zettel(user.id, zettel_id)).ok()
        });
        Self::new_with_zettel(
            user,
            zettel.unwrap_or_else(|| storage::Zettel {
                id: 0,
                path: "home".into(),
                body: ENTRY_TEXT.into(),
                attachments: Vec::new(),
            }),
        )
    }
    pub(crate) fn new_with_zettel(user: Arc<storage::User>, zettel: storage::Zettel) -> Self {
        Self { user, zettel }
    }

    pub(crate) fn render(&mut self, tui: &mut crate::Tui) -> super::Result<Option<Transition>> {
        let mut render_link_input: Option<String> = None;
        let mut rendered_zettel = None;
        loop {
            let title = &self.zettel.path;
            let zettel = rendered_zettel.get_or_insert_with(|| {
                ParsedZettel::parse(
                    &self.zettel,
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
                    "A: All zettels, C: config, E: edit, F: follow link, L: log out, Q: exit, S: search".into(),
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
                    KeyCode::Char('a') => return Ok(Some(Transition::ZettelList)),
                    KeyCode::Char('c') => return Ok(Some(Transition::OpenConfig)),
                    KeyCode::Char('e') => return Ok(Some(Transition::Edit)),
                    KeyCode::Char('f') => {
                        if render_link_input.is_some() {
                            render_link_input = None;
                        } else {
                            render_link_input = Some(String::with_capacity(zettel.link_char_size));
                        }
                        rendered_zettel.take();
                        continue;
                    }
                    KeyCode::Char('l') => return Ok(Some(Transition::Logout)),
                    KeyCode::Char('q') => return Ok(Some(Transition::Exit)),
                    KeyCode::Char('s') => return Ok(Some(Transition::Search)),
                    _ => {}
                }

                if let Some(filter) = &mut render_link_input {
                    if let KeyCode::Char(c) = key_event.code {
                        filter.push(c);
                        if filter.len() == zettel.link_char_size {
                            if let Some(link) = zettel.links.get(filter) {
                                let zettel = zettelkasten_shared::block_on(
                                    tui.storage.get_zettel_by_url(self.user.id, link),
                                )
                                .context(super::DatabaseSnafu)?
                                .unwrap_or_else(|| {
                                    storage::Zettel {
                                        path: link.to_string(),
                                        ..Default::default()
                                    }
                                });
                                return Ok(Some(Transition::NavigateTo(zettel)));
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

pub enum Transition {
    Edit,
    Exit,
    Logout,
    OpenConfig,
    Search,
    ZettelList,
    NavigateTo(storage::Zettel),
}
