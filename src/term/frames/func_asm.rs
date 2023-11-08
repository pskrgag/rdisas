use super::{ItemType, ScreenItem};
use crate::elf::Elf;
use capstone::arch;
use capstone::Capstone;
use capstone::InsnGroupId;
use capstone::InsnGroupType;
use capstone::{Insn, Instructions};
use itertools::Either;
use std::ops::Range;
use tui::{
    style::{Color, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, List, ListItem, ListState},
};

const CALL_INST: u8 = InsnGroupType::CS_GRP_CALL as u8;
const JUMP_INST: u8 = InsnGroupType::CS_GRP_JUMP as u8;

enum BranchInst {
    Call(u64),
    Jump(u64),
}

pub struct FuncAsm {
    insn_list: Instructions<'static>,
    string_list: Vec<Text<'static>>,
    name: String,
    range_cleanup: Option<(Range<usize>, usize)>,
    cs: &'static Capstone,
}

impl FuncAsm {
    pub fn new(function_name: String, elf: &Elf, cs: &'static Capstone) -> Self {
        let (code, addr) = elf.func_code(&function_name);
        let code = cs.disasm_all(code, addr).unwrap();

        Self {
            cs,
            name: function_name,
            string_list: code
                .iter()
                .map(|i| Self::inst_to_string(cs, elf, i))
                .collect(),
            insn_list: code,
            range_cleanup: None,
        }
    }

    fn format_insn(i: &Insn) -> Vec<Span<'static>> {
        let res = format!("{0: <30x}", i.address());
        let mut text = vec![Span::from(res)];

        if let Some(mnemonic) = i.mnemonic() {
            let style = Style::default().fg(Color::Cyan);

            text.push(Span::styled(format!("{:6} ", mnemonic), style));

            if let Some(op_str) = i.op_str() {
                let style = Style::default().fg(Color::Magenta);
                text.push(Span::styled(op_str.to_string(), style));
            }
        }

        text
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

    fn inst_to_string(c: &Capstone, elf: &Elf, inst: &Insn) -> Text<'static> {
        let detail = c.insn_detail(inst);

        if let Ok(d) = detail {
            let group = d.groups();
            let mut call_name = None;

            for i in group {
                match i {
                    InsnGroupId(CALL_INST) => {
                        for op in d.arch_detail().operands() {
                            if let arch::ArchOperand::X86Operand(op) = op {
                                if let arch::x86::X86OperandType::Imm(x) = op.op_type {
                                    call_name = elf.function_name_by_addr(x as u64);

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
                let mut text = Self::format_insn(inst);

                text.push(Span::from(format!("      <{}>", call)));
                Text::from(Line::from(text))
            } else {
                Text::from(Line::from(Self::format_insn(inst)))
            }
        } else {
            Text::from(Line::from(Self::format_insn(inst)))
        }
    }

    fn cleanup_jump(&mut self) {
        if let Some(range) = self.range_cleanup.take() {
            for i in range.0 {
                let text = self.string_list[i].lines[0].spans[0].content.to_mut();

                for j in range.1 + 5..text.len() {
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

    fn draw_jump(&mut self, state: &ListState) {
        let idx = state.selected().unwrap();

        if let Some(inst) = self.is_branch_inst(&self.insn_list.as_ref()[idx]) {
            match inst {
                BranchInst::Jump(addr) => {
                    // Check that jump belongs to current function
                    if self.insn_list.as_ref().iter().last().unwrap().address() < addr {
                        return;
                    }

                    let self_addr = self.insn_list[idx].address();
                    let text = unsafe {
                        self.string_list[idx].lines[0].spans[0]
                            .content
                            .to_mut()
                            .as_bytes_mut()
                    };
                    let addr_offset = text.iter().position(|c| *c == b':').unwrap();

                    if self.insn_list[idx].address() == addr {
                        for j in addr_offset + 5..text.len() {
                            if text[j] != b' ' {
                                break;
                            }

                            text[j] = b'-';
                        }
                    }

                    for i in if addr < self_addr {
                        Either::Left((idx - 1..=0).rev())
                    } else {
                        Either::Right(idx + 1..self.string_list.len())
                    } {
                        let text = unsafe {
                            self.string_list[i].lines[0].spans[0]
                                .content
                                .to_mut()
                                .as_bytes_mut()
                        };

                        if self.insn_list[i].address() == addr {
                            for j in addr_offset + 5..text.len() {
                                if j == text.len() - 1 {
                                    text[j] = b'>';
                                    self.range_cleanup = Some(if addr < self_addr {
                                        (
                                            Range {
                                                start: i,
                                                end: idx + 1,
                                            },
                                            addr_offset,
                                        )
                                    } else {
                                        (
                                            Range {
                                                start: idx + 1,
                                                end: i + 1,
                                            },
                                            addr_offset,
                                        )
                                    });
                                } else {
                                    text[j] = b'-';
                                }
                            }

                            break;
                        }

                        text[addr_offset + 5] = b'|';
                    }
                }
                _ => {}
            }
        }
    }
}

impl ScreenItem for FuncAsm {
    fn title(&self) -> String {
        format!("Disassembly of {}", self.name)
    }

    fn draw(&self) -> List {
        let list = List::new(
            self.string_list
                .clone()
                .into_iter()
                .map(ListItem::new)
                .collect::<Vec<ListItem>>(),
        )
        .style(Style::default().fg(Color::White))
        .highlight_style(Style::default().bg(Color::Cyan));

        list
    }

    fn list_size(&self) -> usize {
        self.insn_list.len()
    }

    fn cursor_move(&mut self, state: &ListState) {
        self.cleanup_jump();
        self.draw_jump(state);
    }

    fn go_in(
        &mut self,
        elf: &Elf,
        cs: &'static Capstone,
        state: &mut ListState,
    ) -> Option<ItemType> {
        let idx = state.selected().unwrap();
        self.cleanup_jump();

        if let Some(inst) = self.is_branch_inst(&self.insn_list.as_ref()[idx]) {
            match inst {
                BranchInst::Call(addr) => {
                    let call_name = elf.function_name_by_addr(addr).unwrap();
                    Some(ItemType::FunctionDisas(FuncAsm::new(call_name, elf, cs)))
                }
                BranchInst::Jump(addr) => {
                    let self_addr = self.insn_list[idx].address();

                    log_info!("Trying to find {:x}", addr);

                    if addr < self_addr {
                        for i in (idx..=0).rev() {
                            if self.insn_list[i].address() == addr {
                                state.select(Some(i));
                                break;
                            }
                        }
                    } else {
                        for i in idx..self.insn_list.len() {
                            if self.insn_list[i].address() == addr {
                                state.select(Some(i));
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
