use super::cmd::CommandLine;
use crate::disas::Backend;
use crate::disas::GlobalState;
use crate::term::frames::func_list::*;
use tui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    Frame, Terminal,
};

use crate::term::frames::*;

use std::collections::LinkedList;

pub struct Term {
    layout: Layout,
    frame_list: LinkedList<ItemType>, // Like a cache for now
    cmd: CommandLine,
    cmd_active: bool,
}

// End of frames
impl Term {
    fn ui_layout() -> Layout {
        Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints(
                [
                    Constraint::Percentage(80),
                    Constraint::Percentage(15),
                    Constraint::Percentage(5),
                ]
                .as_ref(),
            )
    }

    fn draw_ui(&mut self, f: &mut Frame<Backend>) {
        let fr = self.frame_list.front_mut().unwrap();
        let (list, state) = fr.draw();
        let chunks = self.layout.split(f.size());

        f.render_stateful_widget(list, chunks[0], state);

        let debug = crate::dump_logger!();
        f.render_widget(debug, chunks[1]);

        let cmd = self.cmd.dump();

        if self.cmd_active {
            f.render_widget(cmd.style(Style::default().fg(Color::Blue)), chunks[2]);
        } else {
            f.render_widget(cmd, chunks[2]);
        }
    }

    pub fn new() -> Option<Self> {
        Some(Self {
            layout: Self::ui_layout(),
            frame_list: LinkedList::new(),
            cmd: CommandLine::new(),
            cmd_active: false,
        })
    }

    pub fn draw_initial_frame(&mut self, t: &mut Terminal<Backend>, funcs: Vec<String>) {
        assert!(self.frame_list.is_empty());

        let main = FuncList::new(funcs);

        self.frame_list.push_back(ItemType::FunctionList(main));
        t.draw(|f| self.draw_ui(f)).unwrap();
    }

    pub fn next_elem(&mut self, t: &mut Terminal<Backend>) {
        t.draw(|f| {
            {
                let fr = self.frame_list.front_mut().unwrap();
                fr.next();
            }
            self.draw_ui(f);
        })
        .unwrap();
    }

    pub fn prev_elem(&mut self, t: &mut Terminal<Backend>) {
        t.draw(|f| {
            {
                let fr = self.frame_list.front_mut().unwrap();
                fr.prev();
            }
            self.draw_ui(f)
        })
        .unwrap();
    }

    pub fn prev_frame(&mut self, t: &mut Terminal<Backend>) {
        if self.frame_list.len() == 1 {
            return;
        }

        self.frame_list.pop_front();

        t.draw(|f| self.draw_ui(f)).unwrap();
    }

    pub fn go_in(&mut self, t: &mut Terminal<Backend>, state: &GlobalState) {
        // We know it exist
        let current = self.frame_list.front_mut().unwrap();

        match current {
            ItemType::FunctionList(c) => {
                let new = c.go_in(state);
                if let Some(s) = new {
                    self.frame_list.push_front(s);
                }
            }
            ItemType::FunctionDisas(_c) => {}
        }

        self.cmd_active = false;
        t.draw(|f| self.draw_ui(f)).unwrap();
    }

    pub fn activate_cmd(&mut self, t: &mut Terminal<Backend>) {
        self.cmd_active = true;
        self.cmd.clear();
        t.draw(|f| self.draw_ui(f)).unwrap();
    }

    pub fn reset_cmd(&mut self, t: &mut Terminal<Backend>) {
        self.cmd_active = false;
        t.draw(|f| self.draw_ui(f)).unwrap();
    }

    pub fn input_char(&mut self, t: &mut Terminal<Backend>, c: Option<char>) {
        assert!(self.cmd_active);

        self.cmd.proccess_char(c);

        self.frame_list.front_mut().unwrap().find(self.cmd.dump_raw());

        t.draw(|f| {
            self.draw_ui(f);
        })
        .unwrap();
    }
}
