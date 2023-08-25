use crate::disas::GlobalState;
use crate::term::frames::func_list::*;
use std::any::Any;
use std::io;
use tui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, ListState, Widget},
    Frame, Terminal,
};

use crossterm::{
    event::{
        self, DisableMouseCapture, EnableBracketedPaste, EnableFocusChange, EnableMouseCapture,
    },
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

use std::collections::LinkedList;

pub type Backend = CrosstermBackend<std::io::Stdout>;

pub enum ItemType {
    FunctionList,
    FunctionDisas,
}

pub struct Term {
    t: Terminal<Backend>,
    frame_list: LinkedList<Box<dyn ScreenItem>>, // Like a cache for now
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

        let mut main = Box::new(FuncList::new(funcs));
        self.t.draw(|f| main.draw(f)).unwrap();

        self.frame_list.push_back(main);
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

        self.t.draw(|f| self.frame_list.front_mut().unwrap().draw(f));
    }

    pub fn go_in(&mut self, state: GlobalState) {
        // We know it exist
        let current = self.frame_list.front_mut().unwrap();
        let s = current.state();

        match current.whoami() {
            ItemType::FunctionList => {
                let mut c = current.as_any().downcast_mut::<FuncList>().unwrap();
                let mut new = None;

                self.t.draw(|f| {
                    new = c.go_in(f, state);
                });

                if let Some(s) = new {
                    self.frame_list.push_front(s);
                }
            }
            ItemType::FunctionDisas => {},
            _ => todo!(),
        }
    }
}

pub trait ScreenItem {
    fn whoami(&self) -> ItemType;
    fn list_size(&self) -> usize;
    fn state(&mut self) -> &mut ListState;
    fn draw(&mut self, f: &mut Frame<Backend>);
    fn as_any(&mut self) -> &mut dyn Any;

    fn next(&mut self, f: &mut Frame<Backend>) {
        let size = self.list_size();
        let s = self.state();
        let selected = s.selected().unwrap();

        s.select(Some(next_state(size, selected)));
        self.draw(f);
    }

    fn prev(&mut self, f: &mut Frame<Backend>) {
        let size = self.list_size();
        let s = self.state();
        let selected = s.selected().unwrap();

        s.select(Some(prev_state(size, selected)));
        self.draw(f);
    }

    fn go_in(&mut self, f: &mut Frame<Backend>, s: GlobalState) -> Option<Box<dyn ScreenItem>>;
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

// Helper functions

fn next_state(size: usize, state: usize) -> usize {
    (state + 1) % size
}

fn prev_state(size: usize, state: usize) -> usize {
    if state == 0 {
        size - 1
    } else {
        state - 1
    }
}
