use crate::elf::{Elf, Functions};
use crate::term::Term;

// management structure
pub struct Disas<'a> {
    file: String,
    elf: Elf<'a>,
    t: Term,
}

impl<'a> Disas<'a> {
    pub fn new(file: String, e: Elf<'a>) -> Option<Self> {
        info!("Disas created from {} file", file);

        Some(Self {
            file: file,
            elf: e,
            t: Term::new()?,
        })
    }

    pub fn exec(&mut self) -> ! {
        info!("Starting main loop");

        self.t.setup(format!("Disassembly for {}", self.file).as_str());

        let f = self.elf.function_names().expect("Failed to get funtion names");
        self.t.draw_func_list(f.names());

        loop { }
    }

    fn draw_functions(&mut self) {

    }
}
