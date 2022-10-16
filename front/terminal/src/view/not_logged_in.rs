use crossterm::event::{Event, KeyCode};
use snafu::ResultExt;
use tui::{
    text::{Spans, Text},
    widgets::{Block, Borders, Paragraph},
};
use zettelkasten_shared::storage;

#[derive(Default)]
pub struct NotLoggedIn {
    pub username: String,
    pub password: String,
    pub cursor: Cursor,
    pub error: Option<LoginError>,
}

#[derive(Debug, Default, PartialEq, Eq)]
pub enum Cursor {
    #[default]
    Username,
    Password,
}

pub enum LoginError {
    Storage(storage::Error),
}

impl NotLoggedIn {
    pub(crate) fn render(
        &mut self,
        terminal: &mut crate::Terminal,
    ) -> super::Result<Option<Transition>> {
        loop {
            let paragraph = Paragraph::new(Text {
                lines: vec![
                    Spans(vec![
                        "Username: ".into(),
                        self.username.as_str().into(),
                        if self.cursor == Cursor::Username {
                            "_"
                        } else {
                            ""
                        }
                        .into(),
                    ]),
                    Spans(vec![
                        "Password: ".into(),
                        if self.cursor == Cursor::Password {
                            "_"
                        } else {
                            ""
                        }
                        .into(),
                    ]),
                    Spans::default(),
                    Spans(vec!["<tab> next input, <enter> login, <esc> exit".into()]),
                ],
            })
            .block(Block::default().borders(Borders::ALL).title("Log in"));
            let size = terminal.size().context(super::TerminalSizeSnafu)?;
            terminal
                .draw(|f| f.render_widget(paragraph, size))
                .context(super::RenderFrameSnafu)?;

            let event = crossterm::event::read().context(super::EventSnafu)?;
            let input = match self.cursor {
                Cursor::Username => &mut self.username,
                Cursor::Password => &mut self.password,
            };
            if let Event::Key(key_event) = event {
                match key_event.code {
                    KeyCode::Char(c) => input.push(c),
                    KeyCode::Backspace => {
                        let _ = input.pop();
                    }
                    KeyCode::Tab => {
                        self.cursor = match self.cursor {
                            Cursor::Username => Cursor::Password,
                            Cursor::Password => Cursor::Username,
                        }
                    }
                    KeyCode::Enter => match self.cursor {
                        Cursor::Username => self.cursor = Cursor::Password,
                        Cursor::Password => return self.try_login(),
                    },
                    KeyCode::Esc => return Ok(Some(Transition::Exit)),
                    _ => {}
                }
            }
        }
    }

    fn try_login(&self) -> Result<Option<Transition>, super::ViewError> {
        todo!()
    }
}

pub enum Transition {
    Exit,
    Login { user: storage::User },
}
