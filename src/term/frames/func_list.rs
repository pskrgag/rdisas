use super::func_asm::FuncAsm;
use super::{ItemType, ScreenItem};
use crate::disas::GlobalState;
use tui::{
    style::{Color, Style},
    widgets::{Block, Borders, List, ListItem, ListState},
};

pub struct FuncList {
    list: Vec<(usize, String)>, // Should be smth better for prefix finding
    state: ListState,
}

impl FuncList {
    pub fn new(l: Vec<String>) -> Self {
        let mut cnt = 0;

        Self {
            list: l
                .into_iter()
                .map(|x| {
                    let new = (cnt, x);
                    cnt += 1;
                    new
                })
                .collect(),
            state: ListState::default().with_selected(Some(0)),
        }
    }
}

impl ScreenItem for FuncList {
    fn draw(&mut self) -> (List, &mut ListState) {
        let items: Vec<ListItem> = self
            .list
            .iter()
            .map(|i| ListItem::new(i.1.as_str()))
            .collect();
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
        let new = FuncAsm::new(self.list[s].1.clone(), &state);

        Some(ItemType::FunctionDisas(new))
    }

    fn find(&mut self, s: &str) {
        // use fuzzy_match::fuzzy_match;

        log_info!("Tryng to find {}", s);

        for i in self.state.selected().unwrap()..self.list.len() {
            if self.list[i].1.contains(s) {
                self.state.select(Some(i));
                break;
            }
        }

        // let res = fuzzy_match(s, self.list.iter()
        //                                      .map(|x| (x.1.as_str(), x.0))
        //                                      .collect::<Vec<(&str, usize)>>()
        //                                      );
    }
}
