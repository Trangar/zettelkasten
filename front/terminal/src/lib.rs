use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{io::Stdout, sync::Arc};
use tui::{
    backend::CrosstermBackend,
    widgets::{Block, Borders},
    Terminal,
};
use zettelkasten_shared::{storage, Front};

pub struct Tui {
    terminal: Terminal<CrosstermBackend<Stdout>>,
    system_config: storage::SystemConfig,
    storage: Arc<dyn storage::Storage>,
    running: bool,
    state: TuiState,
}

enum TuiState {
    NotLoggedIn { username: String, password: String },
}

impl Default for TuiState {
    fn default() -> Self {
        Self::NotLoggedIn {
            username: String::new(),
            password: String::new(),
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
        enable_raw_mode().unwrap();
        let mut stdout = std::io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture).unwrap();

        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend).unwrap();
        let mut tui = Self {
            terminal,
            system_config,
            storage,
            running: true,
            state: TuiState::default(),
        };
        while tui.running {
            tui.render().expect("Could not render TUI");
            tui.update().expect("Could not update TUI");
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
