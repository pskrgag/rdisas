use std::collections::VecDeque;
use tui::{
    style::{Color, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph},
};

pub enum LogType {
    Info,
    Warn,
    Debug,
}

type LogEntry = (String, LogType);

pub struct Logger {
    log: VecDeque<LogEntry>,
}

pub static mut LOGGER: Logger = Logger::new();

impl Logger {
    pub const fn new() -> Self {
        Self {
            log: VecDeque::new(),
        }
    }

    pub fn push(&mut self, msg: String, t: LogType) {
        self.log.push_back((msg, t));
    }

    pub fn flush(&mut self) -> Paragraph {
        let lines: Vec<_> = self
            .log
            .iter()
            .map(|x| {
                let (style, footer) = match x.1 {
                    LogType::Info => (Style::default().fg(Color::Green), "INFO "),
                    LogType::Warn => (Style::default().fg(Color::Red), "WARN "),
                    LogType::Debug => (Style::default().fg(Color::Yellow), "DEBUG "),
                };

                Line::from(vec![Span::styled(footer, style), Span::raw(x.0.clone())])
            })
            .collect();

        Paragraph::new(Text::from(lines))
            .block(Block::default().title("Debug log").borders(Borders::ALL))
    }
}

#[macro_export]
macro_rules! dump_logger {
    () => {
        unsafe {
            crate::term::logger::LOGGER.flush()
        }
    }
}

#[macro_export]
macro_rules! log_info {
    ($fmt:expr, $($args:tt)*) => {
        unsafe {
            crate::term::logger::LOGGER.push(format!("{}", format_args!($fmt, $($args)*)), crate::term::logger::LogType::Info);
        }
    };
    ($fmt:expr) => {
        unsafe {
            crate::term::logger::LOGGER.push(($fmt).to_string(), crate::term::logger::LogType::Info);
        }
    }
}

#[macro_export]
macro_rules! log_debug {
    ($fmt:expr, $($arg:expr),*) => {
        unsafe {
            crate::term::logger::LOGGER.push(format!("{}", format_args!($fmt, $($args)*)), crate::term::logger::LogType::Debug);
        }
    };
    ($fmt:expr) => {
        unsafe {
            crate::term::logger::LOGGER.push($fmt, crate::term::logger::LogType::Debug);
        }
    }
}

#[macro_export]
macro_rules! log_warn {
    ($fmt:expr, $($arg:expr),*) => {
        unsafe {
            crate::term::logger::LOGGER.push(format!($fmt, $(unsafe_render(&$arg)),*), crate::term::logger::LogType::Warn);
        }
    };
    ($fmt:expr) => {
        unsafe {
            crate::term::logger::LOGGER.push(($fmt).to_string(), crate::term::logger::LogType::Warn);
        }
    }
}
