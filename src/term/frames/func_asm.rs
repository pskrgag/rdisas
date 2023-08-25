use crate::disas::GlobalState;
use crate::elf_disas::CapstoneWrapper;
use crate::term::term::{Backend, ItemType, ScreenItem};
use std::any::Any;
use tui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, ListState, Widget},
    Frame, Terminal,
};

pub struct FuncAsm {
    list: Vec<String>, // Should be smth better for prefix finding
    state: ListState,
}

impl FuncAsm {
    pub fn new(function_name: &String, state: GlobalState) -> Self {
        let d = CapstoneWrapper::new_x86().unwrap();
        let code = state.elf().func_code(function_name);
        let code = d.disas_all(code, 0x0);

        let mut s = Self {
            list: code,
            state: ListState::default(),
        };

        s.state.select(Some(0));
        s
    }
}

impl ScreenItem for FuncAsm {
    fn whoami(&self) -> ItemType {
        ItemType::FunctionDisas
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

    fn as_any(&mut self) -> &mut dyn Any {
        self
    }

    fn go_in(&mut self, f: &mut Frame<Backend>, state: GlobalState) -> Option<Box<dyn ScreenItem>> {
        None
    }
}
