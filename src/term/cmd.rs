use tui::{
    widgets::{Block, Borders, Paragraph},
};

pub struct CommandLine {
    buffer: String,
}

impl CommandLine {
    pub fn new() -> Self {
        Self {
            buffer: String::new(),
        }
    }

    pub fn proccess_char(&mut self, c: Option<char>) {
        match c {
            Some(s) => self.buffer.push(s),
            None => { self.buffer.pop(); },
        };
    }

    pub fn dump_raw(&self) -> &str {
        self.buffer.as_str()
    }

    pub fn dump(&self) -> Paragraph {
        Paragraph::new(self.buffer.clone())
            .block(Block::default().title("Find").borders(Borders::ALL))
    }

    pub fn clear(&mut self) {
        self.buffer.clear();
    }
}
