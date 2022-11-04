#[derive(Default)]
pub struct Search {}

impl Search {
    pub(crate) fn render(&mut self, tui: &mut crate::Tui) -> super::Result<Option<Transition>> {
        super::alert(tui.terminal, |v| {
            v.title("Not implemented").text("Not implemented")
        })?;
        Ok(Some(Transition::Pop))
    }
}

pub enum Transition {
    Pop,
}
