use std::io;
use tui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, ListState, Widget},
    Frame, Terminal,
};

use crossterm::{
    event::{self, DisableMouseCapture, EnableFocusChange, EnableMouseCapture, EnableBracketedPaste},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

use std::collections::LinkedList;

type Backend = CrosstermBackend<std::io::Stdout>;

enum ItemType {
    FunctionList,
    FunctionDisas,
}

pub struct Term {
    t: Terminal<Backend>,
    frame_list: LinkedList<Box<dyn ScreenItem>>, // Like a cache for now
}

struct FuncList {
    list: Vec<String>, // Should be smth better for prefix finding
    state: ListState,
}

impl FuncList {
    pub fn new(l: Vec<String>) -> Self {
        let mut s = Self {
            list: l,
            state: ListState::default(),
        };

        s.state.select(Some(0));
        s
    }
}

impl ScreenItem for FuncList {
    fn whoami(&self) -> ItemType {
        ItemType::FunctionList
    }

    fn draw(&mut self, f: &mut Frame<Backend>) {
        let items: Vec<ListItem> = self.list.iter().map(|i| ListItem::new(&**i)).collect();
        let list = List::new(items)
            .block(
                Block::default()
                    .title("Function list")
                    .borders(Borders::ALL),
            )
            .style(Style::default().fg(Color::White))
            .highlight_style(Style::default().bg(Color::Blue));

        f.render_stateful_widget(list, f.size(), &mut self.state);
    }

    fn state(&mut self) -> &mut ListState {
        &mut self.state
    }

    fn list_size(&self) -> usize {
        self.list.len()
    }

    fn go_in(&mut self, f: &mut Frame<Backend>, next: Box<dyn ScreenItem>) {

    }
}

// End of frames

impl Term {
    pub fn new() -> Option<Self> {
        let stdout = io::stdout();
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend).ok()?;

        enable_raw_mode().ok()?;

        execute!(
            terminal.backend_mut(),
            EnableFocusChange,
        ).ok()?;

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

        self.t.draw(|f| fr.next(f));
    }

    pub fn prev_elem(&mut self) {
        let fr = self.frame_list.front_mut().unwrap();

        self.t.draw(|f| fr.prev(f));
    }

    pub fn go_in(&mut self) {
        // We know it exist
        let current = self.frame_list.front_mut().unwrap();
        let s = current.state();

        match current.whoami() {
            ItemType::FunctionList => {
                let current = current as &mut Box<FuncList>;
            },
            _ => todo!(),
        }
    }
}

trait ScreenItem {
    fn whoami(&self) -> ItemType;
    fn list_size(&self) -> usize;
    fn state(&mut self) -> &mut ListState;

    fn draw(&mut self, f: &mut Frame<Backend>);

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

    fn go_in(&mut self, f: &mut Frame<Backend>, next: Box<dyn ScreenItem>);
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
    if state == 0 { size - 1 } else { state - 1 }
}
