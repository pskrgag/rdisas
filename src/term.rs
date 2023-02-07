use std::io;
use tui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, ListState, Widget},
    Frame, Terminal,
};

use std::collections::LinkedList;

type Backend = CrosstermBackend<std::io::Stdout>;

pub struct Term {
    t: Terminal<Backend>,
    frame_list: LinkedList<Box<dyn ScreenItem>>,
}

struct FuncList {
    list: Vec<String>,
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
    fn draw(&mut self, f: &mut Frame<Backend>) {
        let items: Vec<ListItem> = self.list.iter().map(|i| ListItem::new(&**i)).collect();
        let list = List::new(items)
            .block(Block::default().title("Function list").borders(Borders::ALL))
            .style(Style::default().fg(Color::White))
            .highlight_style(Style::default().bg(Color::Blue));

        f.render_stateful_widget(list, f.size(), &mut self.state);
    }

    fn next(&mut self, f: &mut Frame<Backend>) {}

    fn prev(&mut self, f: &mut Frame<Backend>) {}
}

// End of frames

impl Term {
    pub fn new() -> Option<Self> {
        let stdout = io::stdout();
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend).ok()?;

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
}

trait ScreenItem {
    fn draw(&mut self, f: &mut Frame<Backend>);
    fn next(&mut self, f: &mut Frame<Backend>);
    fn prev(&mut self, f: &mut Frame<Backend>);
}
