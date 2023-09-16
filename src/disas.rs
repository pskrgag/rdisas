use crate::elf::Elf;
use crate::term::{
    events::{wait_event, KeyboardEvent},
    term::Term,
};
use capstone::prelude::*;
use crossterm::{
    event::EnableFocusChange,
    execute,
    terminal::{disable_raw_mode, enable_raw_mode},
};
use std::cell::RefCell;
use std::rc::Rc;
use tui::{backend::CrosstermBackend, Terminal};

pub enum State {
    Control,
    Insert,
}

pub struct GlobalState {
    pub(crate) elf: &'static Elf,
    pub(crate) cs: &'static Capstone,
}

// management structure
pub struct Disas {
    t: Rc<RefCell<Term>>,
    state: State,
    global_state: GlobalState,
}

impl GlobalState {
    pub fn elf(&self) -> &'static Elf {
        self.elf
    }

    pub fn capstone(&self) -> &'static Capstone {
        self.cs
    }
}

pub type Backend = CrosstermBackend<std::io::Stdout>;

impl Disas {
    pub fn new(file: String, mut e: Elf) -> Option<Self> {
        crate::log_info!("Disas created from {} file", file);

        e.load_sections();

        Some(Self {
            global_state: GlobalState {
                elf: Box::leak(Box::new(e)),
                cs: Box::leak(Box::new(
                    Capstone::new()
                        .x86()
                        .mode(arch::x86::ArchMode::Mode64)
                        .detail(true)
                        .build()
                        .ok()?,
                )),
            },
            t: Rc::new(RefCell::new(Term::new()?)),
            state: State::Control,
        })
    }

    pub fn exec(mut self) {
        info!("Starting main loop");

        let f = self
            .global_state
            .elf
            .function_names()
            .expect("Failed to get funtion names");

        let stdout = std::io::stdout();
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend).unwrap();

        enable_raw_mode().ok().unwrap();
        execute!(terminal.backend_mut(), EnableFocusChange,).unwrap();

        terminal.clear().unwrap();

        self.t
            .borrow_mut()
            .draw_initial_frame(&mut terminal, f.names());

        loop {
            let e = wait_event(&self.state);

            if e == KeyboardEvent::CmdEnter {
                self.t.borrow_mut().activate_cmd(&mut terminal);
                self.state = State::Insert;
                continue;
            } else if e == KeyboardEvent::CmdEnd {
                self.t.borrow_mut().reset_cmd(&mut terminal);
                self.state = State::Control;
                continue;
            }

            let t = self.t.clone();

            match e {
                KeyboardEvent::Next => t.borrow_mut().next_elem(&mut terminal),
                KeyboardEvent::Prev => t.borrow_mut().prev_elem(&mut terminal),
                KeyboardEvent::Enter => {
                    self.state = State::Control;
                    t.borrow_mut().go_in(&mut terminal, &self.global_state);
                }
                KeyboardEvent::PrevFrame => t.borrow_mut().prev_frame(&mut terminal),
                KeyboardEvent::Exit => break,
                KeyboardEvent::Key(c) => t.borrow_mut().input_char(&mut terminal, Some(c)),
                KeyboardEvent::Delete => t.borrow_mut().input_char(&mut terminal, None),
                _ => {}
            }
        }

        disable_raw_mode().unwrap();
        terminal.clear().unwrap();
    }
}
