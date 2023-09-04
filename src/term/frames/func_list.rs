use super::func_asm::FuncAsm;
use super::{ItemType, ScreenItem};
use crate::disas::GlobalState;
use tui::{
    style::{Color, Style},
    widgets::{Block, Borders, List, ListItem, ListState},
};

pub struct FuncList {
    func_list: Vec<(usize, String)>, // Should be smth better for prefix finding
    state: ListState,
    ui_list: Vec<ListItem<'static>>,
}

impl FuncList {
    pub fn new(l: Vec<String>) -> Self {
        let mut cnt = 0;
        let func_list: Vec<_> = l
                .into_iter()
                .map(|x| {
                    let new = (cnt, x);
                    cnt += 1;
                    new
                })
                .collect();

        Self {
            state: ListState::default().with_selected(Some(0)),
            ui_list: func_list
                .iter()
                .map(|i| ListItem::new(i.1.clone()))
                .collect(),
            func_list,
        }
    }
}

impl ScreenItem for FuncList {
    fn draw(&mut self) -> (List, &mut ListState) {
        let list = List::new(self.ui_list.clone())
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
        self.func_list.len()
    }

    fn go_in(&self, state: &GlobalState) -> Option<ItemType> {
        let s = self.state.selected().unwrap();
        let new = FuncAsm::new(self.func_list[s].1.clone(), &state);

        Some(ItemType::FunctionDisas(new))
    }

    fn find(&mut self, s: &str) {
        // use fuzzy_match::fuzzy_match;

        log_info!("Tryng to find {}", s);

        for i in self.state.selected().unwrap()..self.func_list.len() {
            if self.func_list[i].1.contains(s) {
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
