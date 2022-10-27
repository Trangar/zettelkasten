use crossterm::event::KeyCode;

pub struct Config {
    pub parent_page: Option<Box<super::View>>,
}

impl Config {
    pub(crate) fn new(parent_page: Option<super::View>, _tui: &mut crate::Tui) -> Self {
        Self {
            parent_page: parent_page.map(Box::new),
        }
    }

    pub(crate) fn render(&mut self, tui: &mut crate::Tui) -> super::Result<Option<Transition>> {
        super::alert(&mut tui.terminal, |f| {
            f.title("Config")
                .text("Not implemented")
                .action(KeyCode::Char('c'), "continue")
        })?;
        Ok(Some(Transition::Pop))
    }
}

pub enum Transition {
    Pop,
}
