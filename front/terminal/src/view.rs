mod config;
mod list;
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
    List(list::List),
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

impl From<config::Config> for ViewLayer {
    fn from(v: config::Config) -> Self {
        ViewLayer::Config(v)
    }
}
impl From<login::Login> for ViewLayer {
    fn from(v: login::Login) -> Self {
        ViewLayer::Login(v)
    }
}
impl From<register::Register> for ViewLayer {
    fn from(v: register::Register) -> Self {
        ViewLayer::Register(v)
    }
}

impl From<zettel::Zettel> for ViewLayer {
    fn from(v: zettel::Zettel) -> Self {
        ViewLayer::Zettel(v)
    }
}

impl From<search::Search> for ViewLayer {
    fn from(v: search::Search) -> Self {
        ViewLayer::Search(v)
    }
}
impl From<list::List> for ViewLayer {
    fn from(v: list::List) -> Self {
        ViewLayer::List(v)
    }
}

impl View {
    pub fn new(system_config: &storage::SystemConfig, storage: &Arc<dyn storage::Storage>) -> Self {
        let layer = if let Ok(0) = zettelkasten_shared::block_on(storage.user_count()) {
            register::Register::default().into()
        } else {
            match system_config.user_mode {
                storage::UserMode::SingleUserAutoLogin => {
                    match zettelkasten_shared::block_on(storage.login_single_user()) {
                        // Successfully logged in
                        Ok(user) => zettel::Zettel::new_with_user(storage, Arc::new(user)).into(),
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
                Some(zettel::Transition::Edit) => {
                    if let Some(str) = utils::edit(&zettel.zettel, tui)? {
                        zettel.zettel.body = str;
                        zettelkasten_shared::block_on(
                            tui.storage
                                .update_zettel(zettel.user.id, &mut zettel.zettel),
                        )
                        .context(DatabaseSnafu)?;
                    }
                    return Ok(());
                }
                Some(zettel::Transition::Exit) => {
                    tui.running = false;
                    return Ok(());
                }
                Some(zettel::Transition::Logout) => Replace(login::Login::default().into()),
                Some(zettel::Transition::OpenConfig) => Push(config::Config::new(tui).into()),
                Some(zettel::Transition::Search) => {
                    Push(search::Search::new(Arc::clone(&zettel.user)).into())
                }
                Some(zettel::Transition::ZettelList) => {
                    Push(list::List::new(Arc::clone(&zettel.user), tui)?.into())
                }
                Some(zettel::Transition::NavigateTo(new_zettel)) => Replace(
                    zettel::Zettel::new_with_zettel(
                        Arc::clone(&zettel.user),
                        new_zettel,
                        tui.storage,
                    )
                    .into(),
                ),
                Some(zettel::Transition::SysPage(page)) => Push(page),
                None => {
                    return Ok(());
                }
            },
            ViewLayer::Login(login) => match login.render(tui)? {
                Some(login::Transition::Exit) => {
                    tui.running = false;
                    return Ok(());
                }
                Some(login::Transition::Register) => Replace(register::Register::default().into()),
                Some(login::Transition::Login { user }) => {
                    Replace(zettel::Zettel::new_with_user(tui.storage, Arc::new(user)).into())
                }
                None => return Ok(()),
            },
            ViewLayer::Register(reg) => match reg.render(tui)? {
                Some(register::Transition::Exit) => {
                    tui.running = false;
                    return Ok(());
                }
                Some(register::Transition::Registered { user }) => {
                    Replace(zettel::Zettel::new_with_user(tui.storage, Arc::new(user)).into())
                }
                Some(register::Transition::Login) => Replace(login::Login::default().into()),
                None => return Ok(()),
            },
            ViewLayer::Config(config) => match config.render(tui)? {
                Some(config::Transition::Pop) => Pop,
                None => return Ok(()),
            },
            ViewLayer::Search(search) => match search.render(tui)? {
                Some(search::Transition::NewZettel(zettel)) => Replace(zettel.into()),
                Some(search::Transition::Pop) => Pop,
                None => return Ok(()),
            },
            ViewLayer::List(list) => match list.render(tui)? {
                Some(list::Transition::NewZettel(zettel)) => Replace(zettel.into()),
                Some(list::Transition::Pop) => Pop,
                None => return Ok(()),
            },
        };

        match next {
            Pop => {
                drop(self.layers.pop());
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

pub type Result<T = ()> = std::result::Result<T, Error>;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum Error {
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
    Io {
        source: std::io::Error,
    },
    #[snafu(display("Unknown system page {page:?}. `sys:` is a reserved prefix"))]
    UnknownSysPage {
        page: String,
    },
}

pub fn alert<F>(terminal: &mut super::Terminal, cb: F) -> Result<KeyCode>
where
    F: Fn(AlertBuilder) -> AlertBuilder,
{
    loop {
        let size = terminal.size().context(TerminalSizeSnafu)?;
        let builder = AlertBuilder::default();
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
                f.render_widget(paragraph, size);
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
pub struct AlertBuilder {
    width: u16,
    height: u16,
    title: Option<Cow<'static, str>>,
    lines: Vec<Cow<'static, str>>,
    actions: Vec<(KeyCode, Cow<'static, str>)>,
}

impl AlertBuilder {
    pub fn title(mut self, text: impl Into<Cow<'static, str>>) -> Self {
        self.title = Some(text.into());
        self
    }
    pub fn text(mut self, text: impl Into<Cow<'static, str>>) -> Self {
        let text = text.into();
        self.width = self
            .width
            .max(u16::try_from(text.chars().count()).unwrap() + 2);
        self.height += 1;
        self.lines.push(text);
        self
    }
    pub fn action(mut self, code: KeyCode, text: impl Into<Cow<'static, str>>) -> Self {
        let text = text.into();
        let mut line_width = u16::try_from(text.chars().count()).unwrap()
            + match code {
                KeyCode::Char(_) => 3, // 'c: '
                KeyCode::Enter => 9,   // '<return> ',
                _ => panic!("Unknown keycode character length: {code:?}"),
            };
        if self.actions.is_empty() {
            self.height += 1;
        } else {
            line_width += 2; // ', '
        }
        self.width = self.width.max(line_width);
        self.actions.push((code, text));
        self
    }
}
