use crossterm::event::{Event, KeyCode};
use snafu::{ResultExt, Snafu};
use std::{borrow::Cow, sync::Arc};
use tui::{
    text::{Spans, Text},
    widgets::{Block, Borders, Paragraph},
};
use zettelkasten_shared::storage;

#[derive(Default)]
pub struct Login {
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

#[derive(Debug, Snafu)]
pub enum LoginError {
    #[snafu(display("Storage error: {source:?}"))]
    Storage { source: storage::Error },
    #[snafu(display("Login failed"))]
    LoginFailed,
}

impl Login {
    pub(crate) fn render(&mut self, tui: &mut crate::Tui) -> super::Result<Option<Transition>> {
        let can_register = tui.can_register()?;
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
            lines.push(Spans::default());

            let mut actions: Cow<'static, str> = "<enter> login, <esc> exit".into();
            if can_register {
                actions = format!("<tab> register, {actions}").into();
            }
            lines.push(Spans(vec![actions.as_ref().into()]));

            let paragraph = Paragraph::new(Text { lines })
                .block(Block::default().borders(Borders::ALL).title("Log in"));
            let size = tui.terminal.size().context(super::TerminalSizeSnafu)?;
            tui.terminal
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
                    KeyCode::Tab if can_register => return Ok(Some(Transition::Register)),
                    KeyCode::Enter => match self.cursor {
                        Cursor::Username => self.cursor = Cursor::Password,
                        Cursor::Password => match self.try_login(tui.storage) {
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

    fn try_login(
        &mut self,
        storage: &Arc<dyn storage::Storage>,
    ) -> Result<Option<Transition>, LoginError> {
        if let Some(user) =
            zettelkasten_shared::block_on(storage.login(&self.username, &self.password))
                .context(StorageSnafu)?
        {
            return Ok(Some(Transition::Login { user }));
        }
        *self = Self {
            error: Some(LoginError::LoginFailed),
            ..Default::default()
        };
        Ok(None)
    }
}

pub enum Transition {
    Exit,
    Register,
    Login { user: storage::User },
}
