use super::{ItemType, ScreenItem};
use crate::disas::GlobalState;
use capstone::Capstone;
use capstone::InsnGroupId;
use capstone::InsnGroupType;
use capstone::{Insn, Instructions};
use tui::{
    style::{Color, Style},
    widgets::{Block, Borders, List, ListItem, ListState},
};

const CALL_INST: u8 = InsnGroupType::CS_GRP_CALL as u8;
const JUMP_INST: u8 = InsnGroupType::CS_GRP_JUMP as u8;

pub struct FuncAsm {
    list: Instructions<'static>,
    state: ListState,
    name: String,
    cs: &'static Capstone,
}

impl FuncAsm {
    pub fn new(function_name: String, state: &GlobalState) -> Self {
        let (code, addr) = state.elf().func_code(&function_name);
        let code = state.capstone().disasm_all(code, addr).unwrap();

        Self {
            cs: state.capstone(),
            name: function_name,
            list: code,
            state: ListState::default().with_selected(Some(0)),
        }
    }

    fn inst_to_string(&self, inst: &Insn) -> String {
        let detail = self.cs.insn_detail(inst);

        if let Ok(d) = detail {
            let group = d.groups();

            for i in group {
                match i {
                    InsnGroupId(CALL_INST) => {
                        log_info!("Found call inst at addr {}", inst.address());
                    }
                    InsnGroupId(JUMP_INST) => {
                        log_info!("Found jump inst at addr {}", inst.address());
                    }
                    _ => {}
                }
            }
            inst.to_string()
        } else {
            inst.to_string()
        }
    }
}

impl ScreenItem for FuncAsm {
    fn draw(&mut self) -> (List, &mut ListState) {
        let items: Vec<ListItem> = self
            .list
            .iter()
            .map(|i| ListItem::new(self.inst_to_string(i)))
            .collect();

        let list = List::new(items)
            .block(
                Block::default()
                    .title(format!("Disassembly of {}", self.name))
                    .borders(Borders::ALL),
            )
            .style(Style::default().fg(Color::White))
            .highlight_style(Style::default().bg(Color::Cyan));

        (list, &mut self.state)
    }

    fn state(&mut self) -> &mut ListState {
        &mut self.state
    }

    fn list_size(&self) -> usize {
        self.list.len()
    }

    fn go_in(&self, _state: &GlobalState) -> Option<ItemType> {
        None
    }
}
