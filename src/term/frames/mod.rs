use crate::elf::Elf;
use capstone::Capstone;
use tui::widgets::{List, ListState, Paragraph};

pub mod func_asm;
pub mod func_list;

use func_asm::FuncAsm;
use func_list::FuncList;

pub enum ItemType {
    FunctionList(FuncList),
    FunctionDisas(FuncAsm),
}

impl ScreenItem for ItemType {
    fn go_in(
        &mut self,
        elf: &Elf,
        cs: &'static Capstone,
        state: &mut ListState,
    ) -> Option<ItemType> {
        match self {
            Self::FunctionList(e) => e.go_in(elf, cs, state),
            Self::FunctionDisas(e) => e.go_in(elf, cs, state),
        }
    }

    fn list_size(&self) -> usize {
        match self {
            Self::FunctionList(s) => s.list_size(),
            Self::FunctionDisas(s) => s.list_size(),
        }
    }

    fn draw(&self) -> List {
        match self {
            Self::FunctionList(s) => s.draw(),
            Self::FunctionDisas(s) => s.draw(),
        }
    }

    fn find(&mut self, state: &mut ListState, ss: &str) {
        match self {
            Self::FunctionList(s) => s.find(state, ss),
            Self::FunctionDisas(s) => s.find(state, ss),
        }
    }

    fn cursor_move(&mut self, state: &ListState) {
        match self {
            Self::FunctionList(s) => s.cursor_move(state),
            Self::FunctionDisas(s) => s.cursor_move(state),
        }
    }

    fn title(&self) -> String {
        match self {
            Self::FunctionList(s) => s.title(),
            Self::FunctionDisas(s) => s.title(),
        }
    }

    fn second_frame(&self) -> Option<Paragraph> {
        match self {
            Self::FunctionList(s) => s.second_frame(),
            Self::FunctionDisas(s) => s.second_frame(),
        }
    }
}

pub trait ScreenItem {
    fn title(&self) -> String;
    fn list_size(&self) -> usize;
    fn draw(&self) -> List;
    fn go_in(
        &mut self,
        elf: &Elf,
        cs: &'static Capstone,
        state: &mut ListState,
    ) -> Option<ItemType>;

    fn second_frame(&self) -> Option<Paragraph> {
        None
    }

    fn cursor_move(&mut self, _state: &ListState) {}

    fn find(&mut self, _state: &mut ListState, _s: &str) {
        crate::log_warn!("Unimplemented!");
    }
}
