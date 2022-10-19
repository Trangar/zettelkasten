use crossterm::event::{Event, KeyCode};
use snafu::ResultExt;
use tui::{
    layout::Rect,
    style::{Color, Style},
    text::{Span, Spans, Text},
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
        let mut render_links = false;
        let mut rendered_zettel = render_zettel(zettel, render_links);
        loop {
            let body = Paragraph::new(Text {
                lines: rendered_zettel.lines.clone(),
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
                    KeyCode::Char('f') => {
                        render_links = !render_links;
                        rendered_zettel = render_zettel(zettel, render_links);
                    }
                    KeyCode::Char('s') => return Err(super::ViewError::NotImplemented),
                    KeyCode::Char('l') => return Ok(Some(Transition::Logout)),
                    _ => {}
                }

                if render_links {
                    for (idx, char) in CHARS.iter().enumerate() {
                        if key_event.code == KeyCode::Char(*char) {
                            drop(zettel);
                            return Ok(Some(Transition::NavigateTo {
                                path: rendered_zettel.links[idx].to_string(),
                            }));
                        }
                    }
                }
            }
        }
    }
}

lazy_static::lazy_static! {
    /// https://regex101.com/r/koCcVt/1
    /// matches:
    /// - [asd]
    /// - [asd](dsa)
    /// but not:
    /// - `[asd]
    /// - `[asd](dsa)
    static ref LINK_REGEX: regex::Regex = regex::Regex::new(r#"[^`](\[[^\]]+\])(\([^)]+\))?"#).unwrap();
}

const CHARS: &[char] = &['a', 'b', 'd', 'g', 'h', 'i', 'j', 'k', 'm', 'n', 'n'];

fn render_zettel(zettel: &storage::Zettel, render_links: bool) -> RenderResult {
    let mut lines = Vec::new();
    let mut links = Vec::new();

    for mut line in zettel.body.lines() {
        let mut parts = Vec::new();

        for link in LINK_REGEX.captures_iter(line) {
            let text = link.get(1).unwrap();
            let maybe_link = link.get(2);
            if text.start() > 0 {
                parts.push(Span::raw(&line[..text.start()]));
            }
            if render_links {
                // if we're rendering links, show the next CHAR as a highlight on top of the link
                let char = CHARS[links.len()];
                parts.push(Span::styled(
                    format!("[{char}]"),
                    Style::default().bg(Color::Yellow),
                ));
                // Render the rest of the link
                let range = text.range();
                parts.push(Span::styled(
                    &line[range.start + 3..range.end],
                    Style::default().fg(Color::Yellow),
                ));
            } else {
                parts.push(Span::styled(
                    &line[text.range()],
                    Style::default().fg(Color::Yellow),
                ));
            }
            let end = maybe_link.map(|l| l.end()).unwrap_or(text.end());
            line = &line[end..];

            let url = if let Some(url) = maybe_link {
                url.as_str()
            } else {
                text.as_str()
            };
            links.push(url);
        }
        if !line.is_empty() {
            parts.push(line.into());
        }
        lines.push(Spans(parts));
    }

    RenderResult { links, lines }
}

struct RenderResult<'a> {
    links: Vec<&'a str>,
    lines: Vec<Spans<'a>>,
}

#[allow(dead_code)]
pub enum Transition {
    Exit,
    Logout,
    NavigateTo { path: String },
}
