use crossterm::event::{Event, KeyCode};
use snafu::ResultExt;
use std::sync::Arc;
use tui::{
    layout::Rect,
    style::{Color, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Paragraph},
};
use zettelkasten_shared::storage;

pub struct Search {
    user: Arc<storage::User>,
    input: String,
    selected: usize,
    results: Vec<storage::ZettelHeader>,
}

impl Search {
    pub(crate) fn new(user: Arc<storage::User>) -> Self {
        Self {
            user,
            input: String::new(),
            selected: 0,
            results: Vec::new(),
        }
    }

    pub(crate) fn render(&mut self, tui: &mut crate::Tui) -> super::Result<Option<Transition>> {
        loop {
            tui.terminal
                .draw(|f| {
                    let size = f.size();
                    let search = Paragraph::new(self.input.as_str())
                        .block(Block::default().borders(Borders::all()).title("Search"));

                    let mut entries = Vec::new();
                    for (idx, zettel) in self.results.iter().enumerate() {
                        let mut spans = Spans::default();

                        spans
                            .0
                            .push(Span::raw(if self.selected == idx { "> " } else { "  " }));
                        spans.0.push(Span::styled(
                            zettel.path.as_str(),
                            Style::default().fg(Color::Yellow),
                        ));
                        if let Some(text) = &zettel.highlight_text {
                            spans.0.push(Span::raw(" "));
                            spans.0.push(Span::raw(text.replace(['\n', '\r'], "")));
                        }

                        entries.push(spans);
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

            let event = crossterm::event::read().context(super::EventSnafu)?;
            if let Event::Key(key_event) = event {
                match key_event.code {
                    KeyCode::Char(c) => {
                        self.input.push(c);
                        self.update_search(tui)?;
                    }
                    KeyCode::Backspace => {
                        self.input.pop();
                        self.update_search(tui)?;
                    }
                    KeyCode::Esc => return Ok(Some(Transition::Pop)),
                    KeyCode::Enter => {
                        let Some(zettel_header) = self.results.get(self.selected) else { continue };
                        match zettelkasten_shared::block_on(
                            tui.storage.get_zettel(self.user.id, zettel_header.id),
                        ) {
                            Ok(zettel) => {
                                return Ok(Some(Transition::NewZettel(
                                    super::zettel::Zettel::new_with_zettel(
                                        Arc::clone(&self.user),
                                        zettel,
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
                        if self.selected + 1 < self.results.len() {
                            self.selected += 1
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    fn update_search(&mut self, tui: &mut crate::Tui) -> super::Result {
        if self.input.is_empty() {
            self.results.clear();
        } else {
            let query = storage::SearchOpts {
                query: &self.input,
                ..Default::default()
            };
            self.results =
                zettelkasten_shared::block_on(tui.storage.get_zettels(self.user.id, query))
                    .context(super::DatabaseSnafu)?;
        }
        if self.results.is_empty() {
            self.selected = 0;
        } else if self.selected >= self.results.len() {
            self.selected = self.results.len() - 1;
        }
        // TODO
        Ok(())
    }
}

pub enum Transition {
    NewZettel(super::zettel::Zettel),
    Pop,
}
