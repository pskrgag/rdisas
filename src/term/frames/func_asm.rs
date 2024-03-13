use super::{ItemType, ScreenItem};
use crate::dwarf::FunctionDebugInfo;
use crate::elf::{Arch, Elf, Function};
use capstone::arch;
use capstone::Capstone;
use capstone::InsnGroupId;
use capstone::InsnGroupType;
use capstone::{Insn, Instructions};
use itertools::Either;
use std::fs::File;
use std::io::{prelude::*, BufReader};
use std::ops::Range;
use tui::{
    style::{Color, Style},
    text::{Line, Span, Text},
    widgets::{List, ListItem, ListState, Paragraph},
};

const CALL_INST: u8 = InsnGroupType::CS_GRP_CALL as u8;
const JUMP_INST: u8 = InsnGroupType::CS_GRP_JUMP as u8;

const COLORS: usize = 4;
const BASE: u8 = 60;
const SKIP: u8 = 10;

enum BranchInst {
    Call(u64),
    Jump(u64),
}

lazy_static::lazy_static! {
    static ref STYLE_ARRAY: [Style; COLORS] = [
        Style::default().bg(Color::Rgb(BASE, BASE, BASE)),
        Style::default().bg(Color::Rgb(BASE - SKIP * 1, BASE - SKIP * 1, BASE - SKIP * 1)),
        Style::default().bg(Color::Rgb(BASE - SKIP * 2, BASE - SKIP * 2, BASE - SKIP * 2)),
        Style::default().bg(Color::Rgb(BASE - SKIP * 3, BASE - SKIP * 3 , BASE - SKIP * 3)),
    ];

    static ref STYLE_SELECTED: Style = Style::default().bg(Color::Blue);
}

pub struct FuncAsm {
    insn_list: Instructions<'static>,
    string_list: Vec<Text<'static>>,
    name: String,
    range_cleanup: Option<(Range<usize>, usize)>,
    cs: &'static Capstone,
    arch: Arch,
    debug_info: Option<(Vec<Line<'static>>, usize)>,
    elf_debug_info: Option<FunctionDebugInfo>,
    marked: (Vec<usize>, Vec<usize>),
}

impl FuncAsm {
    pub fn new(f: Function, elf: &Elf, cs: &'static Capstone) -> Self {
        let (code, addr) = elf.func_code(f.addr());
        let code = cs.disasm_all(code, addr).unwrap();
        let elf_debug_info = elf.function_debug_info(&f);
        let debug_info = Self::debug_frame(&elf_debug_info);

        Self {
            cs,
            arch: elf.arch(),
            name: (*f.name()).clone(),
            string_list: code
                .iter()
                .map(|i| Self::inst_to_string(cs, elf, i, elf.arch()))
                .collect(),
            insn_list: code,
            range_cleanup: None,
            debug_info,
            elf_debug_info,
            marked: (Vec::new(), Vec::new()),
        }
    }

    fn debug_frame(d: &Option<FunctionDebugInfo>) -> Option<(Vec<Line<'static>>, usize)> {
        let di = d.as_ref()?;
        let mut v = Vec::new();
        // let (start, end) = di.line_range();

        if let Some(file) = File::open(di.file_name()).ok() {
            let reader = BufReader::new(file);
            // let mut size = end - start;

            // FIXME: Any idea how to get slice out of Lines?
            for line in reader.lines().skip(0) {
                let l = Line::from(line.ok()?);

                // l.patch_style(STYLE_ARRAY[i % COLORS]);
                v.push(l);

                // size -= 1;
                // if size == 0 {
                //     break;
                // }
            }
        }

        if v.len() != 0 {
            Some((v, 0))
        } else {
            None
        }
    }

    fn format_insn(i: &Insn) -> Vec<Span<'static>> {
        let res = format!("0x{0: <30x}", i.address());
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
                        // I wanna generate it using macros one day
                        match self.arch {
                            Arch::X86_64 | Arch::X86 => {
                                if let arch::ArchOperand::X86Operand(op) = op {
                                    if let arch::x86::X86OperandType::Imm(x) = op.op_type {
                                        return Some(BranchInst::Call(x as u64));
                                    }
                                }
                            }
                            Arch::Arm => {
                                if let arch::ArchOperand::ArmOperand(op) = op {
                                    if let arch::arm::ArmOperandType::Imm(x) = op.op_type {
                                        return Some(BranchInst::Call(x as u64));
                                    }
                                }
                            }
                            Arch::Arm64 => {
                                if let arch::ArchOperand::Arm64Operand(op) = op {
                                    if let arch::arm64::Arm64OperandType::Imm(x) = op.op_type {
                                        return Some(BranchInst::Call(x as u64));
                                    }
                                }
                            }
                            _ => todo!(),
                        }
                    }
                }
                InsnGroupId(JUMP_INST) => {
                    for op in detail.arch_detail().operands() {
                        match self.arch {
                            Arch::X86_64 | Arch::X86 => {
                                if let arch::ArchOperand::X86Operand(op) = op {
                                    if let arch::x86::X86OperandType::Imm(x) = op.op_type {
                                        return Some(BranchInst::Jump(x as u64));
                                    }
                                }
                            }
                            Arch::Arm => {
                                if let arch::ArchOperand::ArmOperand(op) = op {
                                    if let arch::arm::ArmOperandType::Imm(x) = op.op_type {
                                        return Some(BranchInst::Jump(x as u64));
                                    }
                                }
                            }
                            Arch::Arm64 => {
                                if let arch::ArchOperand::Arm64Operand(op) = op {
                                    if let arch::arm64::Arm64OperandType::Imm(x) = op.op_type {
                                        return Some(BranchInst::Jump(x as u64));
                                    }
                                }
                            }
                            _ => todo!(),
                        }
                    }
                }
                _ => {}
            }
        }

        None
    }

    fn inst_to_string(c: &Capstone, elf: &Elf, inst: &Insn, arch: Arch) -> Text<'static> {
        let detail = c.insn_detail(inst);
        let line = if let Ok(d) = detail {
            let group = d.groups();
            let mut call_name = None;

            for i in group {
                match i {
                    InsnGroupId(CALL_INST) => {
                        for op in d.arch_detail().operands() {
                            match arch {
                                Arch::X86_64 | Arch::X86 => {
                                    if let arch::ArchOperand::X86Operand(op) = op {
                                        if let arch::x86::X86OperandType::Imm(x) = op.op_type {
                                            call_name = elf.function_by_addr(x as u64);
                                        }
                                    }
                                }
                                Arch::Arm => {
                                    if let arch::ArchOperand::ArmOperand(op) = op {
                                        if let arch::arm::ArmOperandType::Imm(x) = op.op_type {
                                            call_name = elf.function_by_addr(x as u64);
                                        }
                                    }
                                }
                                Arch::Arm64 => {
                                    if let arch::ArchOperand::Arm64Operand(op) = op {
                                        if let arch::arm64::Arm64OperandType::Imm(x) = op.op_type {
                                            call_name = elf.function_by_addr(x as u64);
                                        }
                                    }
                                }
                                _ => todo!(),
                            }
                        }
                    }
                    InsnGroupId(JUMP_INST) => {
                        // log_info!("Found jump inst at addr {:x}", inst.address());
                    }
                    _ => {}
                }
            }

            if let Some(call) = call_name {
                let mut text = Self::format_insn(inst);

                text.push(Span::from(format!("      <{}>", call.name())));
                Line::from(text)
            } else {
                Line::from(Self::format_insn(inst))
            }
        } else {
            Line::from(Self::format_insn(inst))
        };

        Text::from(line)
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
                    let addr_offset = text.iter().position(|c| *c == b' ').unwrap();

                    if self.insn_list[idx].address() != addr {
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
                                                start: idx,
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

    fn clean_debug(&mut self) {
        if let Some(d) = &mut self.debug_info {
            for debug in &self.marked.0 {
                d.0[*debug].patch_style(Style::default().bg(Color::Reset));
            }

            self.marked.0.clear();

            for inst in &self.marked.1 {
                self.string_list[*inst].patch_style(Style::default().bg(Color::Reset));
            }

            self.marked.1.clear();
        }
    }

    fn color_debug(&mut self, state: &ListState) -> Option<()> {
        let di = self.elf_debug_info.as_ref()?;
        let addr = self.insn_list[state.selected().unwrap()].address();

        let line_orig = di.line_by_addr(addr)?;
        let line = line_orig - self.debug_info.as_ref()?.1;

        if let Some(d) = &mut self.debug_info {
            if d.0.len() > line {
                d.0[line].patch_style(STYLE_ARRAY[0]);
                self.marked.0.push(line);
            }

            let addrs = di.line_to_addrs(*line_orig);
            for i in addrs.unwrap() {
                for (cnt, j) in &mut self.insn_list.iter().enumerate() {
                    if i.contains(&j.address()) {
                        self.string_list[cnt].patch_style(STYLE_ARRAY[0]);
                        self.marked.1.push(cnt);
                    } else if j.address() == addr {
                        std::fs::write("addr.log", format!("{}", addr));
                    }
                }
            }
        }

        Some(())
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
        .highlight_style(Style::default().bg(Color::DarkGray));

        list
    }

    fn list_size(&self) -> usize {
        self.insn_list.len()
    }

    fn cursor_move(&mut self, state: &ListState) {
        self.clean_debug();
        self.color_debug(state);
        self.cleanup_jump();
        self.draw_jump(state);
    }

    fn second_frame(&self) -> Option<Paragraph> {
        Some(Paragraph::new(self.debug_info.as_ref()?.0.clone()))
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
                    let call_name = elf.function_by_addr(addr)?;
                    Some(ItemType::FunctionDisas(FuncAsm::new(call_name, elf, cs)))
                }
                BranchInst::Jump(addr) => {
                    let self_addr = self.insn_list[idx].address();

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
