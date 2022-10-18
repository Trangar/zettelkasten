mod view;

use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use snafu::ResultExt;
use std::{io::Stdout, sync::Arc};
use tui::backend::CrosstermBackend;
use zettelkasten_shared::{storage, Front};

pub(crate) type Terminal = tui::Terminal<CrosstermBackend<Stdout>>;

pub struct Tui {
    pub(crate) terminal: Terminal,
    pub(crate) system_config: storage::SystemConfig,
    pub(crate) storage: Arc<dyn storage::Storage>,
    pub(crate) running: bool,
}

impl Tui {
    pub(crate) fn can_register(&self) -> view::Result<bool> {
        match self.system_config.user_mode {
            storage::UserMode::MultiUser => Ok(true),
            _ => {
                let user_count = zettelkasten_shared::block_on(self.storage.user_count())
                    .context(view::DatabaseSnafu)?;
                Ok(user_count == 0)
            }
        }
    }
}

impl Front for Tui {
    type Config = ();

    fn run(
        _: Self::Config,
        system_config: storage::SystemConfig,
        storage: Arc<dyn storage::Storage>,
    ) {
        enable_raw_mode().expect("Could not enable raw mode");
        let mut stdout = std::io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)
            .expect("Could not enable alternate screen and mouse capture");

        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend).expect("Could not instantiate the terminal");
        let mut view = view::View::new(&system_config, &storage);
        let mut tui = Self {
            terminal,
            system_config,
            storage,
            running: true,
        };

        while tui.running {
            match view.render(&mut tui) {
                Ok(Some(next_state)) => view = next_state,
                Ok(None) => {}
                Err(e) => {
                    let keycode = view::alert(&mut tui.terminal, |f| {
                        f.title("Could not render page")
                            .text(e.to_string())
                            .action(KeyCode::Char('q'), "quit")
                            .action(KeyCode::Char('c'), "continue")
                    })
                    .expect("Double fault, time to crash to desktop");
                    match keycode {
                        KeyCode::Char('q') => return,
                        KeyCode::Char('c') => {}
                        _ => unreachable!(),
                    }
                }
            }
        }
    }
}

impl Drop for Tui {
    fn drop(&mut self) {
        if std::thread::panicking() {
            return;
        }
        let _ = disable_raw_mode();
        let _ = crossterm::execute!(
            self.terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        );
        let _ = self.terminal.show_cursor();
    }
}
