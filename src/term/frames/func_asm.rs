use super::{ItemType, ScreenItem};
use crate::disas::GlobalState;
use capstone::{Instructions, Inst};
use tui::{
    style::{Color, Style},
    widgets::{Block, Borders, List, ListItem, ListState},
};

pub struct FuncAsm {
    list: Instructions<'static>, // Should be smth better for prefix finding
    state: ListState,
    name: String,
}

impl FuncAsm {
    pub fn new(function_name: String, state: &GlobalState) -> Self {
        let (code, addr) = state.elf().func_code(&function_name);
        let code = state.capstone().disasm_all(code, addr).unwrap();

        Self {
            name: function_name,
            list: code,
            state: ListState::default().with_selected(Some(0)),
        }
    }

    // fn inst_to_string(i: &Inst) {
    //     let detail
    // }
}

impl ScreenItem for FuncAsm {
    fn draw(&mut self) -> (List, &mut ListState) {
        let items: Vec<ListItem> = self
            .list
            .iter()
            .map(|i| ListItem::new(i.to_string()))
            .collect();

        let list = List::new(items)
            .block(
                Block::default()
                    .title(format!("Disassembly of {}", self.name))
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

    fn go_in(&self, _state: &GlobalState) -> Option<ItemType> {
        None
    }
}
