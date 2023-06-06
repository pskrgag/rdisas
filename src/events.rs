use crossterm::event::{read, Event, KeyEvent, KeyCode};
use crate::disas::State;

pub enum KeyboardEvent {
    Key(char),
    Enter,
    Next,
    Prev,
    Exit,
}

fn __wait_event() -> Option<KeyEvent> {
    loop {
        match read().ok()? {
            Event::Key(event) => return Some(event),
            _ => {},    // do nothing
        };
    }
}

pub fn wait_event(s: &State) -> KeyboardEvent {
    let mut res;

    while {
        let key = __wait_event().unwrap();

        res = match s {
            State::Insert => {
                match key.code {
                    KeyCode::Char(c) => Some(KeyboardEvent::Key(c)),
                    KeyCode::Enter => Some(KeyboardEvent::Enter),
                    _ => None,
                }
            },
            State::Control => {
                match key.code {
                    KeyCode::Enter => Some(KeyboardEvent::Enter),
                    KeyCode::Up => Some(KeyboardEvent::Prev),
                    KeyCode::Down => Some(KeyboardEvent::Next),
                    KeyCode::Char('q') => Some(KeyboardEvent::Exit),
                    _ => None,
                }
            }
        };

        res.is_none()
    } { }

    res.unwrap()
}
