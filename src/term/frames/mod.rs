use std::any::Any;
use tui::{
    widgets::ListState,
    Frame,
};
use crate::disas::GlobalState;
use crate::term::term::Backend;

pub mod func_asm;
pub mod func_list;

use func_list::FuncList;
use func_asm::FuncAsm;

pub enum ItemType {
    FunctionList(FuncList),
    FunctionDisas(FuncAsm),
}

impl ScreenItem for ItemType {
    fn go_in(&self, f: &mut Frame<Backend>, s: &GlobalState) -> Option<ItemType> {
        match self {
            Self::FunctionList(e) => e.go_in(f, s),
            Self::FunctionDisas(e) => e.go_in(f, s),
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

    fn draw(&mut self, f: &mut Frame<Backend>) {
        match self {
            Self::FunctionList(s) => s.draw(f),
            Self::FunctionDisas(s) => s.draw(f),
        }
    }
}


pub trait ScreenItem {
    fn list_size(&self) -> usize;
    fn state(&mut self) -> &mut ListState;
    fn draw(&mut self, f: &mut Frame<Backend>);
    fn go_in(&self, f: &mut Frame<Backend>, s: &GlobalState) -> Option<ItemType>;

    fn next(&mut self, f: &mut Frame<Backend>) {
        let size = self.list_size();
        let s = self.state();
        let selected = s.selected().unwrap();

        s.select(Some(next_state(size, selected)));
        self.draw(f);
    }

    fn prev(&mut self, f: &mut Frame<Backend>) {
        let size = self.list_size();
        let s = self.state();
        let selected = s.selected().unwrap();

        s.select(Some(prev_state(size, selected)));
        self.draw(f);
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
