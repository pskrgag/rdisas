use crate::app::App;
use crate::app::State;
use crate::term::events::{wait_event, KeyboardEvent};
use crossterm::{
    event::EnableFocusChange,
    execute,
    terminal::{disable_raw_mode, enable_raw_mode},
};
use tui::{backend::CrosstermBackend, Terminal};

// management structure
pub struct Tui {
    term: Terminal<Backend>,
}

pub type Backend = CrosstermBackend<std::io::Stdout>;

impl Tui {
    pub fn new() -> Option<Self> {
        let stdout = std::io::stdout();
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend).ok()?;

        enable_raw_mode().ok().unwrap();
        execute!(terminal.backend_mut(), EnableFocusChange,).unwrap();

        terminal.clear().unwrap();

        Some(Self { term: terminal })
    }

    pub fn next_event(&self, s: State) -> KeyboardEvent {
        wait_event(&s)
    }

    pub fn draw(&mut self, app: &mut App) {
        self.term.draw(|f| crate::term::ui::render(app, f)).unwrap();
    }
}

impl Drop for Tui {
    fn drop(&mut self) {
        disable_raw_mode().unwrap();
        self.term.clear().unwrap();
    }
}
