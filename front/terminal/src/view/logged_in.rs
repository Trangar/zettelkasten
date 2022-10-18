use crossterm::event::{Event, KeyCode};
use snafu::ResultExt;
use tui::{
    style::{Color, Style},
    text::{Span, Spans, Text},
    widgets::{Block, Borders, Paragraph},
};
use zettelkasten_shared::storage;

pub struct LoggedIn {
    pub user: storage::User,
    pub zettel: Option<storage::Zettel>,
}

impl From<storage::User> for LoggedIn {
    fn from(user: storage::User) -> Self {
        Self { user, zettel: None }
    }
}

impl LoggedIn {
    pub(crate) fn render(&self, tui: &mut crate::Tui) -> super::Result<Option<Transition>> {
        loop {
            let mut lines = Vec::with_capacity(6);
            lines.push(Spans(vec![
                Span::raw("Hello "),
                Span::styled(&self.user.name, Style::default().fg(Color::Yellow)),
            ]));

            lines.push(Spans(vec![]));
            lines.push(Spans(vec!["q: quit, L: log out".into()]));

            let paragraph = Paragraph::new(Text { lines })
                .block(Block::default().borders(Borders::ALL).title("Zettelkasten"));
            let size = tui.terminal.size().context(super::TerminalSizeSnafu)?;
            tui.terminal
                .draw(|f| f.render_widget(paragraph, size))
                .context(super::RenderFrameSnafu)?;

            let event = crossterm::event::read().context(super::EventSnafu)?;
            if let Event::Key(key_event) = event {
                match key_event.code {
                    KeyCode::Char('l') => return Ok(Some(Transition::Logout)),
                    KeyCode::Char('q') => return Ok(Some(Transition::Exit)),
                    _ => {}
                }
            }
        }
    }
}

#[allow(dead_code)]
pub enum Transition {
    Exit,
    Logout,
}
