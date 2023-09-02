use crate::disas::GlobalState;
use tui::{
    style::{Color, Style},
    widgets::{Block, Borders, List, ListItem, ListState},
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
    fn draw(&mut self) -> (List, &mut ListState) {
        let items: Vec<ListItem> = self.list.iter().map(|i| ListItem::new(&**i)).collect();
        let list = List::new(items)
            .block(
                Block::default()
                    .title("Function list")
                    .borders(Borders::ALL),
            )
            .style(Style::default().fg(Color::White))
            .highlight_style(Style::default().bg(Color::Cyan));

        (list, &mut self.state)
    }

    fn state(&mut self) -> &mut ListState {
        &mut self.state
    }

    fn list_size(&self) -> usize {
        self.list.len()
    }

    fn go_in(&self, state: &GlobalState) -> Option<ItemType> {
        let s = self.state.selected().unwrap();
        let new = FuncAsm::new(self.list[s].clone(), &state);

        Some(ItemType::FunctionDisas(new))
    }
}
