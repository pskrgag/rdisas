use super::func_asm::FuncAsm;
use super::{ItemType, ScreenItem};
use crate::elf::{Elf, Function};
use capstone::Capstone;
use tui::{
    style::{Color, Style},
    widgets::{List, ListItem, ListState},
};

pub struct FuncList {
    func_list: Vec<Function>, // Should be smth better for prefix finding
    ui_list: Vec<ListItem<'static>>,
}

impl FuncList {
    pub fn new(l: Vec<Function>) -> Self {
        let mut cnt = 0;
        let func_list: Vec<_> = l
            .iter()
            .map(|x| {
                let new = (cnt, x);
                cnt += 1;
                new
            })
            .collect();

        Self {
            ui_list: func_list
                .iter()
                .map(|i| ListItem::new(i.1.name().clone()))
                .collect(),
            func_list: l,
        }
    }
}

impl ScreenItem for FuncList {
    fn title(&self) -> String {
        "Function list".to_owned()
    }

    fn draw(&self) -> List {
        let list = List::new(self.ui_list.clone())
            .style(Style::default().fg(Color::White))
            .highlight_style(Style::default().bg(Color::DarkGray));

        list
    }

    fn list_size(&self) -> usize {
        self.func_list.len()
    }

    fn go_in(
        &mut self,
        elf: &Elf,
        cs: &'static Capstone,
        state: &mut ListState,
    ) -> Option<ItemType> {
        let new = FuncAsm::new(
            self.func_list[state.selected().unwrap()].clone(),
            elf,
            cs,
        );

        Some(ItemType::FunctionDisas(new))
    }

    fn find(&mut self, state: &mut ListState, s: &str) {
        // use fuzzy_match::fuzzy_match;

        // log_info!("Tryng to find {}", s);

        for i in state.selected().unwrap()..self.func_list.len() {
            if self.func_list[i].name().contains(s) {
                state.select(Some(i));
                break;
            }
        }

        // let res = fuzzy_match(s, self.list.iter()
        //                                      .map(|x| (x.1.as_str(), x.0))
        //                                      .collect::<Vec<(&str, usize)>>()
        //                                      );
    }
}
