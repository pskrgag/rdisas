use super::{ItemType, ScreenItem};
use crate::disas::GlobalState;
use crate::elf::Elf;
use capstone::arch;
use capstone::Capstone;
use capstone::InsnGroupId;
use capstone::InsnGroupType;
use capstone::{Insn, Instructions};
use std::ops::Range;
use tui::{
    style::{Color, Style},
    text::Text,
    widgets::{Block, Borders, List, ListItem, ListState},
};

const CALL_INST: u8 = InsnGroupType::CS_GRP_CALL as u8;
const JUMP_INST: u8 = InsnGroupType::CS_GRP_JUMP as u8;

enum BranchInst {
    Call(u64),
    Jump(u64),
}

struct InsnMeta {
    branch: Option<BranchInst>,
}

pub struct FuncAsm {
    insn_list: Instructions<'static>,
    string_list: Vec<Text<'static>>,
    state: ListState,
    name: String,
    cs: &'static Capstone,
    elf: &'static Elf,
    range_cleanup: Option<Range<usize>>,
}

const ADDRESS_OFFSET: usize = 10;

fn insn_to_string(i: &Insn) -> String {
    let mut res = format!("{:#x}:         ", i.address());

    if let Some(mnemonic) = i.mnemonic() {
        res += format!("{} ", mnemonic).as_str();
        if let Some(op_str) = i.op_str() {
            res += format!("{}", op_str).as_str();
        }
    }

    res
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
                .map(|i| Text::raw(Self::inst_to_string(state, i)))
                .collect(),
            insn_list: code,
            cs: state.capstone(),
            elf: state.elf(),
            range_cleanup: None,
        }
    }

    fn is_branch_inst(&self, inst: &Insn) -> Option<BranchInst> {
        let detail = self.cs.insn_detail(inst).ok()?;
        let group = detail.groups();

        for i in group {
            match i {
                InsnGroupId(CALL_INST) => {
                    for op in detail.arch_detail().operands() {
                        if let arch::ArchOperand::X86Operand(op) = op {
                            if let arch::x86::X86OperandType::Imm(x) = op.op_type {
                                return Some(BranchInst::Call(x as u64));
                            }
                        }
                    }
                }
                InsnGroupId(JUMP_INST) => {
                    for op in detail.arch_detail().operands() {
                        if let arch::ArchOperand::X86Operand(op) = op {
                            if let arch::x86::X86OperandType::Imm(x) = op.op_type {
                                return Some(BranchInst::Jump(x as u64));
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        None
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

                                    log_info!(
                                        "Found call inst at addr {} to 0x{:x}",
                                        inst.address(),
                                        x
                                    );
                                }
                            }
                        }
                    }
                    InsnGroupId(JUMP_INST) => {
                        log_info!("Found jump inst at addr {:x}", inst.address());
                    }
                    _ => {}
                }
            }

            if let Some(call) = call_name {
                format!("{}   <{}>", insn_to_string(inst), call).to_string()
            } else {
                insn_to_string(inst)
            }
        } else {
            insn_to_string(inst)
        }
    }

    fn cleanup_jump(&mut self) {
        if let Some(range) = self.range_cleanup.take() {
            for i in range {
                let text = self.string_list[i].lines[0].spans[0].content.to_mut();

                for j in ADDRESS_OFFSET + 5..text.len() {
                    let c = unsafe { &mut text.as_bytes_mut()[j] };
                    if *c != b'-' && *c != b'|' && *c != b' ' && *c != b'>' {
                        break;
                    } else {
                        *c = b' ';
                    }
                }
            }
        }
    }

    fn draw_jump(&mut self) {
        let idx = self.state.selected().unwrap();

        log_warn!("I am here");

        if let Some(inst) = self.is_branch_inst(&self.insn_list.as_ref()[idx]) {
            match inst {
                BranchInst::Jump(addr) => {
                    let self_addr = self.insn_list[idx].address();

                    let text = self.string_list[idx].lines[0].spans[0].content.to_mut();

                    if self.insn_list[idx].address() == addr {
                        for j in ADDRESS_OFFSET + 5..text.len() {
                            if text.as_bytes()[j] != b' ' {
                                break;
                            }

                            unsafe {
                                text.as_bytes_mut()[j] = b'-';
                            }
                        }
                    }

                    if addr < self_addr {
                        for i in (idx - 1..=0).rev() {
                            let text = self.string_list[i].lines[0].spans[0].content.to_mut();

                            if self.insn_list[i].address() == addr {
                                for j in ADDRESS_OFFSET + 5..text.len() {
                                    if text.as_bytes()[j] != b' ' {
                                        self.range_cleanup = Some(Range { start: i, end: idx + 1});
                                        break;
                                    }

                                    unsafe {
                                        if text.as_bytes()[j + 1] != b' ' {
                                            text.as_bytes_mut()[j] = b'>';
                                        } else {
                                            text.as_bytes_mut()[j] = b'-';
                                        }
                                    }
                                }

                                break;
                            }

                            unsafe {
                                text.as_bytes_mut()[ADDRESS_OFFSET + 5] = b'|';
                            }
                        }
                    } else {
                        for i in idx + 1..self.string_list.len() {
                            let text = self.string_list[i].lines[0].spans[0].content.to_mut();

                            if self.insn_list[i].address() == addr {
                                for j in ADDRESS_OFFSET + 5..text.len() {
                                    if text.as_bytes()[j] != b' ' {
                                        self.range_cleanup = Some(Range { start: idx, end: i + 1});
                                        break;
                                    }

                                    unsafe {
                                        if text.as_bytes()[j + 1] != b' ' {
                                            text.as_bytes_mut()[j] = b'>';
                                        } else {
                                            text.as_bytes_mut()[j] = b'-';
                                        }
                                    }
                                }

                                break;
                            }

                            unsafe {
                                text.as_bytes_mut()[ADDRESS_OFFSET + 5] = b'|';
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }
}

impl ScreenItem for FuncAsm {
    fn draw(&mut self) -> (List, &mut ListState) {
        let list = List::new(
            self.string_list
                .clone()
                .into_iter()
                .map(|x| ListItem::new(x))
                .collect::<Vec<ListItem>>(),
        )
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

    fn next(&mut self) {
        let size = self.list_size();
        let s = &self.state;
        let selected = s.selected().unwrap();

        self.cleanup_jump();

        self.state.select(Some(next_state(size, selected)));
        self.draw_jump();
    }

    fn prev(&mut self) {
        let size = self.list_size();
        let s = &self.state;
        let selected = s.selected().unwrap();

        self.cleanup_jump();

        self.state.select(Some(prev_state(size, selected)));
        self.draw_jump();
    }

    fn go_in(&mut self, state: &GlobalState) -> Option<ItemType> {
        let idx = self.state.selected().unwrap();
        self.cleanup_jump();

        if let Some(inst) = self.is_branch_inst(&self.insn_list.as_ref()[idx]) {
            match inst {
                BranchInst::Call(addr) => {
                    let call_name = self.elf.function_name_by_addr(addr).unwrap();
                    Some(ItemType::FunctionDisas(FuncAsm::new(call_name, state)))
                }
                BranchInst::Jump(addr) => {
                    let self_addr = self.insn_list[idx].address();

                    log_info!("Trying to find {:x}", addr);

                    if addr < self_addr {
                        for i in (idx..=0).rev() {
                            if self.insn_list[i].address() == addr {
                                self.state.select(Some(i));
                                break;
                            }
                        }
                    } else {
                        for i in idx..self.insn_list.len() {
                            if self.insn_list[i].address() == addr {
                                self.state.select(Some(i));
                                break;
                            }
                        }
                    }

                    None
                }
            }
        } else {
            None
        }
    }
}

fn next_state(size: usize, state: usize) -> usize {
    (state + 1) % size
}
fn prev_state(size: usize, state: usize) -> usize {
    if state == 0 {
        size - 1
    } else {
        state - 1
    }
}
