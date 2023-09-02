use tui::{
    widgets::{ListState, List},
};
use crate::disas::GlobalState;

pub mod func_asm;
pub mod func_list;

use func_list::FuncList;
use func_asm::FuncAsm;

pub enum ItemType {
    FunctionList(FuncList),
    FunctionDisas(FuncAsm),
}

impl ScreenItem for ItemType {
    fn go_in(&self, s: &GlobalState) -> Option<ItemType> {
        match self {
            Self::FunctionList(e) => e.go_in(s),
            Self::FunctionDisas(e) => e.go_in(s),
        }
    }

    fn list_size(&self) -> usize {
        match self {
            Self::FunctionList(s) => s.list_size(),
            Self::FunctionDisas(s) => s.list_size(),
        }
    }

    fn state(&mut self) -> &mut ListState {
        match self {
            Self::FunctionList(s) => s.state(),
            Self::FunctionDisas(s) => s.state(),
        }
    }

    fn draw(&mut self) -> (List, &mut ListState) {
        match self {
            Self::FunctionList(s) => s.draw(),
            Self::FunctionDisas(s) => s.draw(),
        }
    }
}

pub trait ScreenItem {
    fn list_size(&self) -> usize;
    fn state(&mut self) -> &mut ListState;
    fn draw(&mut self) -> (List, &mut ListState);
    fn go_in(&self, s: &GlobalState) -> Option<ItemType>;

    fn next(&mut self) {
        let size = self.list_size();
        let s = self.state();
        let selected = s.selected().unwrap();

        s.select(Some(next_state(size, selected)));
    }

    fn prev(&mut self) {
        let size = self.list_size();
        let s = self.state();
        let selected = s.selected().unwrap();

        s.select(Some(prev_state(size, selected)));
    }
}

// Helper functions

fn next_state(size: usize, state: usize) -> usize {
    (state + 1) % size
}

fn prev_state(size: usize, state: usize) -> usize {
    if state == 0 {
        size - 1
    } else {
        state - 1
    }
}
