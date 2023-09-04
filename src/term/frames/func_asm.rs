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
use capstone::arch;

const CALL_INST: u8 = InsnGroupType::CS_GRP_CALL as u8;
const JUMP_INST: u8 = InsnGroupType::CS_GRP_JUMP as u8;

pub struct FuncAsm {
    insn_list: Instructions<'static>,
    string_list: Vec<ListItem<'static>>,
    state: ListState,
    name: String,
}

impl FuncAsm {
    pub fn new(function_name: String, state: &GlobalState) -> Self {
        let (code, addr) = state.elf().func_code(&function_name);
        let code = state.capstone().disasm_all(code, addr).unwrap();

        Self {
            name: function_name,
            state: ListState::default().with_selected(Some(0)),
            string_list: code
                .iter()
                .map(|i| ListItem::new(Self::inst_to_string(state, i)))
                .collect(),
            insn_list: code,
        }
    }

    fn inst_to_string(c: &GlobalState, inst: &Insn) -> String {
        let detail = c.capstone().insn_detail(inst);

        if let Ok(d) = detail {
            let group = d.groups();
            let mut call_name = None;

            for i in group {
                match i {
                    InsnGroupId(CALL_INST) => {
                        for op in d.arch_detail().operands() {
                            if let arch::ArchOperand::X86Operand(op) = op {
                                if let arch::x86::X86OperandType::Imm(x) = op.op_type {
                                    let e = c.elf();

                                    call_name = e.function_name_by_addr(x as u64);

                                    log_info!("Found call inst at addr {} to 0x{:x}", inst.address(), x);
                                }
                            }
                        }
                    }
                    InsnGroupId(JUMP_INST) => {
                        log_info!("Found jump inst at addr {}", inst.address());
                    }
                    _ => {}
                }
            }

            if let Some(call) = call_name {
                format!("{}   <{}>", inst.to_string(), call).to_string()
            } else {
                inst.to_string()
            }
        } else {
            inst.to_string()
        }
    }
}

impl ScreenItem for FuncAsm {
    fn draw(&mut self) -> (List, &mut ListState) {
        let list = List::new(self.string_list.clone())
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
        self.insn_list.len()
    }

    fn go_in(&self, _state: &GlobalState) -> Option<ItemType> {
        None
    }
}
