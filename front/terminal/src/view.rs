mod logged_in;
mod not_logged_in;

use crossterm::event::{Event, KeyCode};
use std::{borrow::Cow, sync::Arc};
use tui::{
    layout::Rect,
    text::{Spans, Text},
    widgets::{Block, Borders, Paragraph},
};
use zettelkasten_shared::storage;

pub enum View {
    NotLoggedIn(not_logged_in::NotLoggedIn),
    LoggedIn(logged_in::LoggedIn),
}

impl From<not_logged_in::NotLoggedIn> for View {
    fn from(v: not_logged_in::NotLoggedIn) -> Self {
        Self::NotLoggedIn(v)
    }
}

impl From<logged_in::LoggedIn> for View {
    fn from(v: logged_in::LoggedIn) -> Self {
        Self::LoggedIn(v)
    }
}

impl View {
    pub fn new(system_config: &storage::SystemConfig, storage: &Arc<dyn storage::Storage>) -> Self {
        match system_config.user_mode {
            storage::UserMode::SingleUserAutoLogin => {
                match zettelkasten_shared::block_on(storage.login_single_user()) {
                    // Successfully logged in
                    Ok(user) => Self::LoggedIn(user.into()),
                    // Failed to log in, show the login view and the error
                    Err(e) => not_logged_in::NotLoggedIn {
                        error: Some(not_logged_in::LoginError::Storage(e)),
                        ..Default::default()
                    }
                    .into(),
                }
            }
            storage::UserMode::MultiUser | storage::UserMode::SingleUserManualLogin => {
                // show the login view
                not_logged_in::NotLoggedIn::default().into()
            }
        }
    }
}

pub fn alert<F>(terminal: &mut super::Terminal, cb: F) -> KeyCode
where
    F: Fn(ViewBuilder) -> ViewBuilder,
{
    loop {
        let size = terminal.size().unwrap();
        let builder = ViewBuilder::default();
        let builder = cb(builder);
        terminal
            .draw(|f| {
                let mut lines = Vec::<Spans>::with_capacity(
                    builder.lines.len() + if builder.actions.is_empty() { 0 } else { 1 },
                );
                for line in &builder.lines {
                    lines.push(line.as_ref().into());
                }
                let mut actions = String::new();
                for (idx, (key, text)) in builder.actions.iter().enumerate() {
                    if idx != 0 {
                        actions += ", ";
                    }
                    match key {
                        KeyCode::Char(c) => actions.push(*c),
                        KeyCode::Enter => actions += "<enter>",
                        _ => unreachable!(),
                    }
                    actions += ": ";
                    actions += text;
                }
                lines.push(actions.into());
                let mut block = Block::default().borders(Borders::ALL);
                if let Some(title) = &builder.title {
                    block = block.title(title.as_ref());
                }
                const VERT_MARGIN: u16 = 2;
                const HOR_MARGIN: u16 = 6;
                let x = (size.width - (builder.width + HOR_MARGIN)) / 2;
                let y = (size.height - (builder.height + VERT_MARGIN)) / 2;

                let paragraph = Paragraph::new(Text { lines }).block(block);
                f.render_widget(
                    paragraph,
                    Rect::new(
                        x,
                        y,
                        builder.width + HOR_MARGIN,
                        builder.height + VERT_MARGIN,
                    ),
                )
            })
            .unwrap();

        let event = crossterm::event::read().unwrap();
        if let Event::Key(key) = event {
            if builder.actions.iter().any(|(k, _)| k == &key.code) {
                break key.code;
            }
        }
    }
}

#[derive(Default)]
pub struct ViewBuilder {
    width: u16,
    height: u16,
    title: Option<Cow<'static, str>>,
    lines: Vec<Cow<'static, str>>,
    actions: Vec<(KeyCode, Cow<'static, str>)>,
}

impl ViewBuilder {
    pub fn title(mut self, text: impl Into<Cow<'static, str>>) -> Self {
        self.title = Some(text.into());
        self
    }
    pub fn text(mut self, text: impl Into<Cow<'static, str>>) -> Self {
        let text = text.into();
        self.width = self.width.max(text.chars().count() as u16 + 2);
        self.height += 1;
        self.lines.push(text);
        self
    }
    pub fn action(mut self, code: KeyCode, text: impl Into<Cow<'static, str>>) -> Self {
        let text = text.into();
        let mut line_width = text.chars().count() as u16
            + match code {
                KeyCode::Char(_) => 3, // 'c: '
                KeyCode::Enter => 9,   // '<return> ',
                _ => panic!("Unknown keycode character length: {code:?}"),
            };
        if !self.actions.is_empty() {
            line_width += 2; // ', '
        } else {
            self.height += 1;
        }
        self.width = self.width.max(line_width);
        self.actions.push((code, text));
        self
    }
}
