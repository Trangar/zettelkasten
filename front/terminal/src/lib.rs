mod view;

use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{io::Stdout, sync::Arc};
use tui::backend::CrosstermBackend;
use zettelkasten_shared::{storage, Front};

pub type Terminal = tui::Terminal<CrosstermBackend<Stdout>>;

pub struct Tui {
    terminal: Terminal,
    #[allow(dead_code)]
    system_config: storage::SystemConfig,
    storage: Arc<dyn storage::Storage>,
    running: bool,
    view: view::View,
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
        let view = view::View::new(&system_config, &storage);
        let mut tui = Self {
            terminal,
            system_config,
            storage,
            running: true,
            view,
        };
        while tui.running {
            match tui
                .view
                .render(&mut tui.running, &mut tui.terminal, &tui.storage)
            {
                Ok(Some(next_state)) => tui.view = next_state,
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
