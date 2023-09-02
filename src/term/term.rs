use crate::disas::GlobalState;
use crate::term::frames::func_list::*;
use std::io;
use tui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    widgets::{Block, Borders, Paragraph},
    Frame, Terminal,
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
    layout: Layout,
    frame_list: LinkedList<ItemType>, // Like a cache for now
}

// End of frames
impl Term {
    fn ui_layout() -> Layout {
        Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([Constraint::Percentage(80), Constraint::Percentage(20)].as_ref())
    }

    fn draw_list(layout: Layout, fr: &mut impl ScreenItem, f: &mut Frame<Backend>) {
        let (list, state) = fr.draw();
        let chunks = layout.split(f.size());

        f.render_stateful_widget(list, chunks[0], state);

        let debug = crate::dump_logger!();
        f.render_widget(debug, chunks[1]);
    }

    pub fn new() -> Option<Self> {
        let stdout = io::stdout();
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend).ok()?;

        enable_raw_mode().ok()?;

        execute!(terminal.backend_mut(), EnableFocusChange,).ok()?;

        Some(Self {
            layout: Self::ui_layout(),
            t: terminal,
            frame_list: LinkedList::new(),
        })
    }

    pub fn setup(&mut self, title: &str) {
        self.t.clear().unwrap();

        self.t
            .draw(|f| {
                let chunks = self.layout.split(f.size());

                let block = Block::default().title(title).borders(Borders::ALL);
                f.render_widget(block, chunks[0]);
                f.render_widget(crate::dump_logger!(), chunks[1]);
            })
            .unwrap();
    }

    pub fn draw_func_list(&mut self, funcs: Vec<String>) {
        assert!(self.frame_list.is_empty());

        let mut main = FuncList::new(funcs);

        self.t.draw(|f| Self::draw_list(self.layout.clone(), &mut main, f)).unwrap();

        self.frame_list.push_back(ItemType::FunctionList(main));
    }

    pub fn next_elem(&mut self) {
        let fr = self.frame_list.front_mut().unwrap();

        self.t
            .draw(|f| {
                fr.next();
                Self::draw_list(self.layout.clone(), fr, f)
            })
            .unwrap();
    }

    pub fn prev_elem(&mut self) {
        let fr = self.frame_list.front_mut().unwrap();

        self.t
            .draw(|f| {
                fr.prev();
                Self::draw_list(self.layout.clone(), fr, f)
            })
            .unwrap();
    }

    pub fn prev_frame(&mut self) {
        if self.frame_list.len() == 1 {
            return;
        }

        self.frame_list.pop_front();

        self.t
            .draw(|f| {
                let new = self.frame_list.front_mut().unwrap();
                Self::draw_list(self.layout.clone(), new, f)
            })
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
                            new = c.go_in(state);
                            if let Some(ref mut s) = new {
                                Self::draw_list(self.layout.clone(), s, f);
                            }
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
