use crate::elf::{Elf, Functions};
use crate::term::{
    events::{wait_event, KeyboardEvent},
    term::Term,
};

pub enum State {
    Control,
    Insert,
}

// management structure
pub struct Disas<'a> {
    file: String,
    elf: Elf<'a>,
    t: Term,
    state: State,
}

pub struct GlobalState<'a> {
    pub(crate) elf: &'a Elf<'a>,
}

impl<'a> GlobalState<'a> {
    pub fn elf(&self) -> &'a Elf<'a> {
        self.elf
    }
}

impl<'a> Disas<'a> {
    pub fn new(file: String, mut e: Elf<'a>) -> Option<Self> {
        info!("Disas created from {} file", file);

        e.load_sections();

        Some(Self {
            file: file,
            elf: e,
            t: Term::new()?,
            state: State::Control,
        })
    }

    pub fn exec(&mut self) {
        info!("Starting main loop");

        self.t
            .setup(format!("Disassembly for {}", self.file).as_str());

        let f = self
            .elf
            .function_names()
            .expect("Failed to get funtion names");

        self.t.draw_func_list(f.names());

        loop {
            let e = wait_event(&self.state);

            match e {
                KeyboardEvent::Next => self.t.next_elem(),
                KeyboardEvent::Prev => self.t.prev_elem(),
                KeyboardEvent::Enter => self.t.go_in(GlobalState { elf: &self.elf }),
                KeyboardEvent::PrevFrame => self.t.prev_frame(),
                KeyboardEvent::Exit => break,
                _ => {}
            }
        }
    }
}
