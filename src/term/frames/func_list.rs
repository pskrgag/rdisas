use crate::disas::GlobalState;
use crate::term::term::Backend;
use std::any::Any;
use tui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, ListState, Widget},
    Frame, Terminal,
};
use super::func_asm::FuncAsm;
use super::{ScreenItem, ItemType};

pub struct FuncList {
    list: Vec<String>, // Should be smth better for prefix finding
    state: ListState,
}

impl FuncList {
    pub fn new(l: Vec<String>) -> Self {
        Self {
            list: l,
            state: ListState::default().with_selected(Some(0)),
        }
    }
}

impl ScreenItem for FuncList {
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

    fn go_in(&self, f: &mut Frame<Backend>, state: &GlobalState) -> Option<ItemType> {
        let s = self.state.selected().unwrap();
        let mut new = FuncAsm::new(&self.list[s], &state);

        new.draw(f);
        Some(ItemType::FunctionDisas(new))
    }
}
