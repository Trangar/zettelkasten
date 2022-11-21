#![deny(clippy::perf)]
#![warn(clippy::pedantic)]

mod view;

use crossterm::{
    event::KeyCode,
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use snafu::ResultExt;
use std::{io::Stdout, sync::Arc};
use tui::backend::CrosstermBackend;
use zettelkasten_shared::{storage, Front};

pub(crate) type Terminal = tui::Terminal<CrosstermBackend<Stdout>>;

pub struct Tui<'a> {
    pub(crate) terminal: &'a mut Terminal,
    pub(crate) system_config: &'a mut storage::SystemConfig,
    pub(crate) storage: &'a Arc<dyn storage::Storage>,
    pub(crate) running: bool,
}

impl Tui<'_> {
    pub(crate) fn can_register(&self) -> view::Result<bool> {
        if let storage::UserMode::MultiUser = self.system_config.user_mode {
            Ok(true)
        } else {
            let user_count = zettelkasten_shared::block_on(self.storage.user_count())
                .context(view::DatabaseSnafu)?;
            Ok(user_count == 0)
        }
    }
}

impl Front for Tui<'_> {
    type Config = ();

    fn run(
        _: Self::Config,
        mut system_config: storage::SystemConfig,
        storage: Arc<dyn storage::Storage>,
    ) {
        enable_raw_mode().expect("Could not enable raw mode");
        let mut stdout = std::io::stdout();
        execute!(stdout, EnterAlternateScreen).expect("Could not enable alternate screen");

        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend).expect("Could not instantiate the terminal");
        let mut view = view::View::new(&system_config, &storage);
        let mut tui = Tui {
            terminal: &mut terminal,
            system_config: &mut system_config,
            storage: &storage,
            running: true,
        };

        while tui.running {
            let Err(e) = view.render(&mut tui) else { continue };
            let keycode = view::alert(tui.terminal, |f| {
                f.title("Could not render page")
                    .text(e.to_string())
                    .action(KeyCode::Char('q'), "quit")
                    .action(KeyCode::Enter, "continue")
            })
            .expect("Double fault, time to crash to desktop");

            if keycode == KeyCode::Char('q') {
                tui.running = false;
            }
        }
        drop(tui.terminal.clear());
    }
}

impl Drop for Tui<'_> {
    fn drop(&mut self) {
        drop(disable_raw_mode());

        if !std::thread::panicking() {
            drop(self.terminal.backend_mut().execute(LeaveAlternateScreen));
        }
        drop(self.terminal.show_cursor());
    }
}
