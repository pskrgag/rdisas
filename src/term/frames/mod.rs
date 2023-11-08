use crate::elf::Elf;
use capstone::Capstone;
use tui::widgets::{List, ListState};

pub mod func_asm;
pub mod func_list;

use func_asm::FuncAsm;
use func_list::FuncList;

pub enum ItemType {
    FunctionList(FuncList),
    FunctionDisas(FuncAsm),
}

impl ScreenItem for ItemType {
    fn go_in(&mut self, elf: &Elf, cs: &'static Capstone, state: &ListState) -> Option<ItemType> {
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
}

pub trait ScreenItem {
    fn list_size(&self) -> usize;
    fn draw(&self) -> List;
    fn go_in(&mut self, elf: &Elf, cs: &'static Capstone, state: &ListState) -> Option<ItemType>;

    fn cursor_move(&mut self, _state: &ListState) {}

    fn find(&mut self, _state: &mut ListState, _s: &str) {
        crate::log_warn!("Unimplemented!");
    }
}
