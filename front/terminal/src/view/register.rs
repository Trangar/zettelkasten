use crossterm::event::{Event, KeyCode};
use snafu::{ResultExt, Snafu};
use std::sync::Arc;
use tui::{
    text::{Spans, Text},
    widgets::{Block, Borders, Paragraph},
};
use zettelkasten_shared::storage;

#[derive(Default)]
pub struct Register {
    pub username: String,
    pub password: String,
    pub repeat_password: String,
    pub cursor: Cursor,
    pub error: Option<RegisterError>,
}

#[derive(Debug, Default, PartialEq, Eq)]
pub enum Cursor {
    #[default]
    Username,
    Password,
    RepeatPassword,
}

#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Snafu)]
pub enum RegisterError {
    #[snafu(display("Storage error"))]
    Storage { source: storage::Error },
    #[snafu(display("Register failed"))]
    RegisterFailed,
    #[snafu(display("Passwords do not match"))]
    PasswordsDontMatch,
}

impl Register {
    pub(crate) fn render(&mut self, tui: &mut crate::Tui) -> super::Result<Option<Transition>> {
        if !tui.can_register()? {
            return Ok(Some(Transition::Login));
        }
        loop {
            let mut lines = Vec::with_capacity(6);
            if let Some(error) = &self.error {
                lines.push(error.to_string().into());
                lines.push(Spans::default());
            }
            lines.push(Spans(vec![
                "Username: ".into(),
                self.username.as_str().into(),
                if self.cursor == Cursor::Username {
                    "_"
                } else {
                    ""
                }
                .into(),
            ]));
            lines.push(Spans(vec![
                "Password: ".into(),
                if self.cursor == Cursor::Password {
                    "_"
                } else {
                    ""
                }
                .into(),
            ]));
            lines.push(Spans(vec![
                "Repeat password: ".into(),
                if self.cursor == Cursor::RepeatPassword {
                    "_"
                } else {
                    ""
                }
                .into(),
            ]));
            lines.push(Spans::default());

            lines.push(Spans(vec![
                "<tab> login, <enter> register, <esc> exit".into()
            ]));

            let paragraph = Paragraph::new(Text { lines })
                .block(Block::default().borders(Borders::ALL).title("Register"));
            let size = tui.terminal.size().context(super::TerminalSizeSnafu)?;
            tui.terminal
                .draw(|f| f.render_widget(paragraph, size))
                .context(super::RenderFrameSnafu)?;

            let event = crossterm::event::read().context(super::EventSnafu)?;
            let input = match self.cursor {
                Cursor::Username => &mut self.username,
                Cursor::Password => &mut self.password,
                Cursor::RepeatPassword => &mut self.repeat_password,
            };
            if let Event::Key(key_event) = event {
                match key_event.code {
                    KeyCode::Char(c) => input.push(c),
                    KeyCode::Backspace => {
                        let _ = input.pop();
                    }
                    KeyCode::Tab => return Ok(Some(Transition::Login)),
                    KeyCode::Enter => match self.cursor {
                        Cursor::Username => self.cursor = Cursor::Password,
                        Cursor::Password => self.cursor = Cursor::RepeatPassword,
                        Cursor::RepeatPassword => match self.try_register(tui.storage) {
                            Ok(v) => return Ok(v),
                            Err(e) => {
                                self.error = Some(e);
                            }
                        },
                    },
                    KeyCode::Esc => return Ok(Some(Transition::Exit)),
                    _ => {}
                }
            }
        }
    }

    fn try_register(
        &mut self,
        storage: &Arc<dyn storage::Storage>,
    ) -> Result<Option<Transition>, RegisterError> {
        if self.password != self.repeat_password {
            *self = Self {
                error: Some(RegisterError::PasswordsDontMatch),
                ..Default::default()
            };
            return Ok(None);
        }
        match zettelkasten_shared::block_on(storage.register(&self.username, &self.password)) {
            Ok(user) => Ok(Some(Transition::Registered { user })),
            Err(storage::Error::UserAlreadyExists) => {
                *self = Self {
                    error: Some(RegisterError::RegisterFailed),
                    ..Default::default()
                };
                Ok(None)
            }
            Err(source) => Err(RegisterError::Storage { source }),
        }
    }
}

pub enum Transition {
    Exit,
    Registered { user: storage::User },
    Login,
}
