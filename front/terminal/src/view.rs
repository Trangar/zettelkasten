mod config;
mod login;
mod register;
mod search;
mod utils;
mod zettel;

use crossterm::event::{Event, KeyCode};
use snafu::{ResultExt, Snafu};
use std::{borrow::Cow, sync::Arc};
use tui::{
    text::{Spans, Text},
    widgets::{Block, Borders, Paragraph},
};
use zettelkasten_shared::storage;

pub struct View {
    layers: Vec<ViewLayer>,
}

enum ViewLayer {
    Config(config::Config),
    Login(login::Login),
    Register(register::Register),
    Zettel(zettel::Zettel),
    Search(search::Search),
}

enum ViewReplace {
    Pop,
    Push(ViewLayer),
    Replace(ViewLayer),
}

impl From<ViewLayer> for View {
    fn from(l: ViewLayer) -> Self {
        Self { layers: vec![l] }
    }
}

impl From<login::Login> for ViewLayer {
    fn from(v: login::Login) -> Self {
        ViewLayer::Login(v)
    }
}

impl From<zettel::Zettel> for ViewLayer {
    fn from(v: zettel::Zettel) -> Self {
        ViewLayer::Zettel(v)
    }
}
impl From<config::Config> for ViewLayer {
    fn from(v: config::Config) -> Self {
        ViewLayer::Config(v)
    }
}
impl From<search::Search> for ViewLayer {
    fn from(v: search::Search) -> Self {
        ViewLayer::Search(v)
    }
}

impl View {
    pub fn new(system_config: &storage::SystemConfig, storage: &Arc<dyn storage::Storage>) -> Self {
        let layer = match system_config.user_mode {
            storage::UserMode::SingleUserAutoLogin => {
                match zettelkasten_shared::block_on(storage.login_single_user()) {
                    // Successfully logged in
                    Ok(user) => zettel::Zettel::new_with_user(storage, user).into(),
                    // Failed to log in, show the login view and the error
                    Err(source) => login::Login {
                        error: Some(login::LoginError::Storage { source }),
                        ..Default::default()
                    }
                    .into(),
                }
            }
            storage::UserMode::MultiUser | storage::UserMode::SingleUserManualLogin => {
                // show the login view
                login::Login::default().into()
            }
        };
        Self {
            layers: vec![layer],
        }
    }

    pub(crate) fn render(&mut self, tui: &mut crate::Tui) -> Result {
        use ViewReplace::{Pop, Push, Replace};

        let layer: &mut ViewLayer = self.layers.last_mut().unwrap();

        let next: ViewReplace = match layer {
            ViewLayer::Zettel(zettel) => match zettel.render(tui)? {
                Some(t) => match t.into_view_replace(zettel, tui)? {
                    Some(replace) => replace,
                    None => return Ok(()),
                },
                None => return Ok(()),
            },
            ViewLayer::Login(login) => match login.render(tui)? {
                Some(login::Transition::Exit) => {
                    tui.running = false;
                    return Ok(());
                }
                Some(login::Transition::Register) => {
                    Replace(ViewLayer::Register(Default::default()))
                }
                Some(login::Transition::Login { user }) => {
                    Replace(zettel::Zettel::new_with_user(&tui.storage, user).into())
                }
                None => return Ok(()),
            },
            ViewLayer::Register(reg) => match reg.render(tui)? {
                Some(register::Transition::Exit) => {
                    tui.running = false;
                    return Ok(());
                }
                Some(register::Transition::Registered { user }) => {
                    Replace(zettel::Zettel::new_with_user(&tui.storage, user).into())
                }
                Some(register::Transition::Login) => Replace(ViewLayer::Login(Default::default())),
                None => return Ok(()),
            },
            ViewLayer::Config(config) => match config.render(tui)? {
                Some(config::Transition::Pop) => Pop,
                None => return Ok(()),
            },
            ViewLayer::Search(search) => match search.render(tui)? {
                Some(search::Transition::Pop) => Pop,
                None => return Ok(()),
            },
        };

        match next {
            Pop => {
                let _ = self.layers.pop();
                assert!(!self.layers.is_empty());
            }
            Push(layer) => {
                self.layers.push(layer);
            }
            Replace(layer) => {
                self.layers = vec![layer];
            }
        }

        Ok(())
    }
}

pub type Result<T = ()> = std::result::Result<T, ViewError>;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum ViewError {
    #[snafu(display("Could not retrieve the terminal size"))]
    TerminalSize {
        source: std::io::Error,
    },
    #[snafu(display("Could not render a frame"))]
    RenderFrame {
        source: std::io::Error,
    },
    #[snafu(display("Could not get the next terminal event"))]
    Event {
        source: std::io::Error,
    },
    #[snafu(display("Database error: {source:?}"))]
    Database {
        source: storage::Error,
    },
    #[snafu(display("Zettel ID {id} not found"))]
    ZettelIdNotFound {
        id: i64,
    },
    #[snafu(display("Not implemented"))]
    NotImplemented,
    Io {
        source: std::io::Error,
    },
}

pub fn alert<F>(terminal: &mut super::Terminal, cb: F) -> Result<KeyCode>
where
    F: Fn(ViewBuilder) -> ViewBuilder,
{
    loop {
        let size = terminal.size().context(TerminalSizeSnafu)?;
        let builder = ViewBuilder::default();
        let builder = cb(builder);
        terminal
            .draw(|f| {
                let mut lines = Vec::<Spans>::with_capacity(
                    builder.lines.len() + if builder.actions.is_empty() { 0 } else { 2 },
                );
                for line in &builder.lines {
                    lines.push(line.as_ref().into());
                }
                if !builder.actions.is_empty() {
                    lines.push(Spans::default());
                    let mut actions = String::new();
                    for (idx, (key, text)) in builder.actions.iter().enumerate() {
                        if idx != 0 {
                            actions += ", ";
                        }
                        match key {
                            KeyCode::Char(c) => actions.push(c.to_ascii_uppercase()),
                            KeyCode::Enter => actions += "<enter>",
                            _ => unreachable!(),
                        }
                        actions += ": ";
                        actions += text;
                    }
                    lines.push(actions.into());
                }
                let mut block = Block::default().borders(Borders::ALL);
                if let Some(title) = &builder.title {
                    block = block.title(title.as_ref());
                }

                let paragraph = Paragraph::new(Text { lines }).block(block);
                f.render_widget(paragraph, size)
            })
            .context(RenderFrameSnafu)?;

        let event = crossterm::event::read().context(EventSnafu)?;
        if let Event::Key(key) = event {
            if builder.actions.iter().any(|(k, _)| k == &key.code) {
                break Ok(key.code);
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
