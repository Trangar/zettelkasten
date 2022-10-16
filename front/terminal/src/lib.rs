mod view;

use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{io::Stdout, sync::Arc};
use tui::{
    backend::CrosstermBackend,
    widgets::{Block, Borders},
};
use zettelkasten_shared::{storage, Front};

pub type Terminal = tui::Terminal<CrosstermBackend<Stdout>>;

pub struct Tui {
    terminal: Terminal,
    system_config: storage::SystemConfig,
    storage: Arc<dyn storage::Storage>,
    running: bool,
    state: view::View,
}

impl Front for Tui {
    type Config = ();

    fn run(
        _: Self::Config,
        system_config: storage::SystemConfig,
        storage: Arc<dyn storage::Storage>,
    ) {
        enable_raw_mode().unwrap();
        let mut stdout = std::io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture).unwrap();

        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend).unwrap();
        let state = view::View::new(&system_config, &storage);
        let mut tui = Self {
            terminal,
            system_config,
            storage,
            running: true,
            state,
        };
        let mut i = 0;
        while tui.running {
            let keycode = view::alert(&mut tui.terminal, |f| {
                f.title("Could not render page")
                    .text(format!("Not implemented ({i})"))
                    .action(KeyCode::Char('q'), "quit")
                    .action(KeyCode::Char('c'), "continue")
            });
            match keycode {
                KeyCode::Char('q') => return,
                KeyCode::Char('c') => {
                    i += 1;
                }
                _ => unreachable!(),
            }
        }
    }
}

impl Tui {
    fn render(&mut self) -> std::io::Result<()> {
        self.terminal.draw(|f| {
            let size = f.size();
            let block = Block::default().title("Block").borders(Borders::ALL);
            f.render_widget(block, size);
        })?;
        Ok(())
    }
    fn update(&mut self) -> std::io::Result<()> {
        let event = crossterm::event::read()?;
        if let Event::Key(key) = event {
            if let KeyCode::Char('q') = key.code {
                self.running = false;
            }
        }
        Ok(())
    }
}

impl Drop for Tui {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = crossterm::execute!(
            self.terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        );
        let _ = self.terminal.show_cursor();
    }
}
