use super::{ItemType, ScreenItem};
use crate::disas::GlobalState;
use crate::term::term::Backend;
use capstone::Instructions;
use tui::{
    style::{Color, Style},
    widgets::{Block, Borders, List, ListItem, ListState},
    Frame,
};

pub struct FuncAsm {
    list: Instructions<'static>, // Should be smth better for prefix finding
    state: ListState,
}

impl FuncAsm {
    pub fn new(function_name: &String, state: &GlobalState) -> Self {
        let code = state.elf().func_code_exe(function_name);
        let code = state.capstone().disasm_all(code, 0x0).unwrap();

        Self {
            list: code,
            state: ListState::default().with_selected(Some(0)),
        }
    }
}

impl ScreenItem for FuncAsm {
    fn draw(&mut self, f: &mut Frame<Backend>) {
        let items: Vec<ListItem> = self
            .list
            .iter()
            .map(|i| ListItem::new(i.to_string()))
            .collect();

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

    fn go_in(&self, _f: &mut Frame<Backend>, _state: &GlobalState) -> Option<ItemType> {
        None
    }
}
