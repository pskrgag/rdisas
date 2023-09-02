use crate::disas::GlobalState;
use crate::term::frames::func_list::*;
use std::io;
use tui::{
    backend::CrosstermBackend,
    widgets::{Block, Borders},
    Terminal,
};

use crate::term::frames::*;
use crossterm::{
    event::EnableFocusChange,
    execute,
    terminal::{disable_raw_mode, enable_raw_mode},
};

use std::collections::LinkedList;

pub type Backend = CrosstermBackend<std::io::Stdout>;

pub struct Term {
    t: Terminal<Backend>,
    frame_list: LinkedList<ItemType>, // Like a cache for now
}

// End of frames

impl Term {
    pub fn new() -> Option<Self> {
        let stdout = io::stdout();
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend).ok()?;

        enable_raw_mode().ok()?;

        execute!(terminal.backend_mut(), EnableFocusChange,).ok()?;

        Some(Self {
            t: terminal,
            frame_list: LinkedList::new(),
        })
    }

    pub fn setup(&mut self, title: &str) {
        self.t.clear().unwrap();

        self.t
            .draw(|f| {
                let size = f.size();
                let block = Block::default().title(title).borders(Borders::ALL);
                f.render_widget(block, size);
            })
            .unwrap();
    }

    pub fn draw_func_list(&mut self, funcs: Vec<String>) {
        assert!(self.frame_list.is_empty());

        let mut main = FuncList::new(funcs);
        self.t.draw(|f| main.draw(f)).unwrap();

        self.frame_list.push_back(ItemType::FunctionList(main));
    }

    pub fn next_elem(&mut self) {
        let fr = self.frame_list.front_mut().unwrap();

        self.t.draw(|f| fr.next(f)).unwrap();
    }

    pub fn prev_elem(&mut self) {
        let fr = self.frame_list.front_mut().unwrap();

        self.t.draw(|f| fr.prev(f)).unwrap();
    }

    pub fn prev_frame(&mut self) {
        if self.frame_list.len() == 1 {
            return;
        }

        self.frame_list.pop_front();

        self.t
            .draw(|f| self.frame_list.front_mut().unwrap().draw(f))
            .unwrap();
    }

    pub fn go_in(&mut self, state: &GlobalState) {
        let mut new = None;

        {
            // We know it exist
            let current = self.frame_list.front_mut().unwrap();

            match current {
                ItemType::FunctionList(c) => {
                    self.t
                        .draw(|f| {
                            new = c.go_in(f, state);
                        })
                        .unwrap();
                }
                ItemType::FunctionDisas(c) => {}
            }
        }

        if let Some(s) = new {
            self.frame_list.push_front(s);
        }
    }
}

impl Drop for Term {
    fn drop(&mut self) {
        // Restore terminal in normal state
        #[allow(unused_must_use)]
        {
            self.t.clear();
            disable_raw_mode();
        }
    }
}
