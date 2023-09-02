use crate::elf::Elf;
use crate::term::{
    events::{wait_event, KeyboardEvent},
    term::Term,
};
use capstone::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

pub enum State {
    Control,
    Insert,
}

pub struct GlobalState {
    pub(crate) elf: Elf,
    pub(crate) cs: &'static Capstone,
}

// management structure
pub struct Disas {
    file: String,
    t: Rc<RefCell<Term>>,
    state: State,
    global_state: GlobalState,
}

impl GlobalState {
    pub fn elf(&self) -> &Elf {
        &self.elf
    }

    pub fn capstone(&self) -> &'static Capstone {
        self.cs
    }
}

impl Disas {
    pub fn new(file: String, mut e: Elf) -> Option<Self> {
        info!("Disas created from {} file", file);

        e.load_sections();

        Some(Self {
            file,
            global_state: GlobalState {
                elf: e,
                cs: Box::leak(Box::new(Capstone::new().x86().mode(arch::x86::ArchMode::Mode64).build().ok()?)),
            },
            t: Rc::new(RefCell::new(Term::new()?)),
            state: State::Control,
        })
    }

    pub fn exec(self) {
        info!("Starting main loop");

        self.t
            .borrow_mut().setup(format!("Disassembly for {}", self.file).as_str());

        let f = self
            .global_state
            .elf
            .function_names()
            .expect("Failed to get funtion names");

        self.t.borrow_mut().draw_func_list(f.names());

        loop {
            let e = wait_event(&self.state);

            let t = self.t.clone();

            match e {
                KeyboardEvent::Next => t.borrow_mut().next_elem(),
                KeyboardEvent::Prev => t.borrow_mut().prev_elem(),
                KeyboardEvent::Enter => t.borrow_mut().go_in(&self.global_state),
                KeyboardEvent::PrevFrame => t.borrow_mut().prev_frame(),
                KeyboardEvent::Exit => break,
                _ => {}
            }
        }
    }
}
