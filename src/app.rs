use super::term::cmd::CommandLine;
use crate::elf::{Arch, Elf};
use crate::term::events::KeyboardEvent;
use crate::term::frames::func_list::*;
use capstone::prelude::*;
use tui::widgets::ListState;

use crate::term::frames::*;

use std::collections::LinkedList;

#[derive(Copy, Clone, PartialEq)]
pub enum State {
    Control,
    Insert,
}

pub struct App {
    frame_list: LinkedList<(ItemType, ListState)>, // Like a cache for now
    elf: Elf,

    /* I am done with rust lifetimes. I need to keep references
     * to instructions in Disassembly frame, but poisoning whole struct
     * with lifetime makes things x10 harder.
     */
    cs: &'static Capstone,
    state: State,

    pub cmd: CommandLine,
}

// End of frames
impl App {
    fn next_state(item: &ItemType, liststate: &mut ListState) {
        let size = item.list_size();
        let selected = liststate.selected().unwrap();

        liststate.select(Some(next_state(size, selected)));
    }

    fn prev_state(item: &ItemType, liststate: &mut ListState) {
        let size = item.list_size();
        let selected = liststate.selected().unwrap();

        liststate.select(Some(prev_state(size, selected)));
    }

    pub fn active_main_frame(&mut self) -> &mut (ItemType, ListState) {
        self.frame_list.front_mut().unwrap()
    }

    pub fn proccess_event(&mut self, e: KeyboardEvent) -> bool {
        if e == KeyboardEvent::CmdEnter {
            self.cmd.clear();
            self.state = State::Insert;

            return false;
        } else if e == KeyboardEvent::CmdEnd {
            self.state = State::Control;

            return false;
        }

        match e {
            KeyboardEvent::Next => self.next_elem(),
            KeyboardEvent::Prev => self.prev_elem(),
            KeyboardEvent::Enter => {
                self.state = State::Control;
                self.go_in();
            }
            KeyboardEvent::PrevFrame => self.prev_frame(),
            KeyboardEvent::Exit => return true,
            KeyboardEvent::Key(c) => self.input_char(Some(c)),
            KeyboardEvent::Delete => self.input_char(None),
            _ => {}
        }

        false
    }

    fn add_front_frame(&mut self, item: ItemType) {
        self.frame_list
            .push_back((item, ListState::default().with_selected(Some(0))))
    }

    pub fn state(&self) -> State {
        self.state
    }

    pub fn new(elf: Elf) -> Option<Self> {
        let funcs = elf.function_names()?;
        let list = FuncList::new(funcs.names());

        let cs = match elf.arch() {
            Arch::X86 => Capstone::new()
                .x86()
                .mode(arch::x86::ArchMode::Mode32)
                .detail(true)
                .build()
                .ok()?,
            Arch::X86_64 => Capstone::new()
                .x86()
                .mode(arch::x86::ArchMode::Mode32)
                .detail(true)
                .build()
                .ok()?,
            Arch::Arm64 => Capstone::new()
                .arm64()
                .mode(arch::arm64::ArchMode::Arm)
                .detail(true)
                .build()
                .ok()?,
            Arch::Arm => Capstone::new().arm().detail(true).build().ok()?,
            Arch::Riscv => Capstone::new().riscv().detail(true).build().ok()?,
            Arch::Mips => Capstone::new().mips().detail(true).build().ok()?,
        };

        let mut s = Self {
            cs: Box::leak(Box::new(cs)),
            frame_list: LinkedList::new(),
            cmd: CommandLine::new(),
            state: State::Control,
            elf,
        };

        s.add_front_frame(ItemType::FunctionList(list));
        Some(s)
    }

    pub fn next_elem(&mut self) {
        let fr = self.active_main_frame();
        Self::next_state(&fr.0, &mut fr.1);
        fr.0.cursor_move(&fr.1);
    }

    pub fn prev_elem(&mut self) {
        let fr = self.active_main_frame();
        Self::prev_state(&fr.0, &mut fr.1);
        fr.0.cursor_move(&fr.1);
    }

    pub fn prev_frame(&mut self) {
        if self.frame_list.len() == 1 {
            return;
        }

        self.frame_list.pop_front();
    }

    pub fn go_in(&mut self) {
        // We know it exist
        let (fr, state) = self.frame_list.front_mut().unwrap();

        let new = fr.go_in(&self.elf, self.cs, state);
        if let Some(s) = new {
            self.frame_list
                .push_front((s, ListState::default().with_selected(Some(0))));
        }
    }

    pub fn input_char(&mut self, c: Option<char>) {
        self.cmd.proccess_char(c);

        let (fr, state) = self.frame_list.front_mut().unwrap();

        fr.find(state, self.cmd.dump_raw());
    }
}

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
