use crossterm::event::{Event, KeyCode};
use snafu::ResultExt;
use std::sync::Arc;
use tui::{
    layout::Rect,
    text::{Span, Spans},
    widgets::{Block, Borders, Paragraph},
};
use zettelkasten_shared::storage;

pub struct List {
    user: Arc<storage::User>,
    selected: usize,
    search: String,
    links: Vec<storage::ZettelHeader>,
}

impl List {
    pub fn new(user: Arc<storage::User>, tui: &crate::Tui) -> super::Result<Self> {
        let links = zettelkasten_shared::block_on(tui.storage.get_zettels(
            user.id,
            storage::SearchOpts {
                list_all: true,
                ..Default::default()
            },
        ))
        .context(super::DatabaseSnafu)?;
        Ok(Self {
            user,
            selected: 0,
            search: String::new(),
            links,
        })
    }

    pub(crate) fn render(&mut self, tui: &mut crate::Tui) -> super::Result<Option<Transition>> {
        loop {
            let links: Vec<&storage::ZettelHeader> = if self.search.trim().is_empty() {
                self.links.iter().collect()
            } else {
                self.links
                    .iter()
                    .filter(|l| {
                        l.path
                            .to_ascii_lowercase()
                            .contains(&self.search.to_ascii_lowercase())
                    })
                    .collect()
            };
            if links.is_empty() {
                self.selected = 0;
            } else if links.len() <= self.selected {
                self.selected = links.len() - 1;
            }
            self.draw(tui, &links)?;
            let event = crossterm::event::read().context(super::EventSnafu)?;
            if let Event::Key(key_event) = event {
                match key_event.code {
                    KeyCode::Char(c) => self.search.push(c),
                    KeyCode::Backspace => {
                        self.search.pop();
                    }
                    KeyCode::Esc => return Ok(Some(Transition::Pop)),
                    KeyCode::Enter => {
                        let Some(zettel_header) = links.get(self.selected) else { continue };
                        match zettelkasten_shared::block_on(
                            tui.storage.get_zettel(self.user.id, zettel_header.id),
                        ) {
                            Ok(zettel) => {
                                return Ok(Some(Transition::NewZettel(
                                    super::zettel::Zettel::new_with_zettel(
                                        Arc::clone(&self.user),
                                        zettel,
                                        tui.storage,
                                    ),
                                )))
                            }
                            Err(e) => {
                                super::alert(tui.terminal, |f| {
                                    f.title("Error")
                                        .text("Zettel not found")
                                        .text(e.to_string())
                                        .action(KeyCode::Enter, "Continue")
                                })?;
                            }
                        }
                    }
                    KeyCode::Up => {
                        if self.selected >= 1 {
                            self.selected -= 1;
                        }
                    }
                    KeyCode::Down => {
                        if self.selected + 1 < links.len() {
                            self.selected += 1;
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    fn draw(
        &self,
        tui: &mut crate::Tui,
        links: &[&storage::ZettelHeader],
    ) -> Result<(), super::Error> {
        tui.terminal
            .draw(|f| {
                let size = f.size();
                let search = Paragraph::new(self.search.as_str())
                    .block(Block::default().borders(Borders::all()).title("Search"));

                let mut entries = Vec::new();
                let mut previous_parts = None;
                for (idx, zettel) in links.iter().enumerate() {
                    let mut spans = Spans::default();

                    spans
                        .0
                        .push(Span::raw(if self.selected == idx { "> " } else { "  " }));
                    let parts = zettel.path.split('/').collect::<Vec<_>>();
                    let common_parts =
                        CommonParts::get(&parts, previous_parts.as_deref().unwrap_or_default());
                    for _ in 0..common_parts.common_length {
                        spans.0.push(Span::raw("  "));
                    }
                    for (idx, remaining) in common_parts.remaining.iter().enumerate() {
                        if idx != 0 {
                            spans.0.push(Span::raw("/"));
                        }
                        spans.0.push(Span::raw(*remaining));
                    }

                    entries.push(spans);
                    previous_parts = Some(parts);
                }
                let body = Paragraph::new(entries).block(
                    Block::default().borders(Borders::RIGHT | Borders::LEFT | Borders::BOTTOM),
                );
                let actions =
                    Paragraph::new("Up/Down: Select entry, Enter: Go to zettel, Esc: go back");

                f.render_widget(search, Rect { height: 3, ..size });
                f.render_widget(
                    body,
                    Rect {
                        height: size.height - 4,
                        y: 3,
                        ..size
                    },
                );
                f.render_widget(
                    actions,
                    Rect {
                        height: 1,
                        y: size.height - 1,
                        ..size
                    },
                );
            })
            .context(super::IoSnafu)?;
        Ok(())
    }
}

struct CommonParts<'a, 'b> {
    pub common_length: usize,
    pub remaining: &'a [&'b str],
}
impl<'a, 'b> CommonParts<'a, 'b> {
    fn get(current: &'a [&'b str], previous: &[&str]) -> Self {
        let common_length = current
            .iter()
            .zip(previous.iter())
            .take_while(|(left, right)| left.to_ascii_lowercase() == right.to_ascii_lowercase())
            .count();
        Self {
            common_length,
            remaining: &current[common_length..],
        }
    }
}

pub enum Transition {
    NewZettel(super::zettel::Zettel),
    Pop,
}
