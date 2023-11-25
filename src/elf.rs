use crate::dwarf::{DwarfParser, FunctionDebugInfo};
use elf::endian::AnyEndian;
use elf::section::{SectionHeader, SectionHeaderTable};
use elf::symbol::Symbol;
use elf::ElfBytes;
use std::collections::HashMap;

// Name and symbol
pub struct Function(String, Symbol);
type FunctionMap = HashMap<u64, Function>;

// TODO: extend maybe?
#[derive(PartialEq, Eq)]
pub enum Arch {
    X86,
    X86_64,
    Arm64,
    Arm,
    Riscv,
    Mips,
}

pub struct Elf {
    data: ElfBytes<'static, AnyEndian>,
    sections: SectionHeaderTable<'static, AnyEndian>,
    functions: FunctionMap,
    debug_info: Option<DwarfParser>,
}

impl Function {
    pub fn new(name: String, sym: Symbol) -> Self {
        Self(name, sym)
    }

    pub fn name(&self) -> &String {
        &self.0
    }

    pub fn addr(&self) -> u64 {
        self.1.st_value
    }

    pub fn size(&self) -> usize {
        self.1.st_size as usize
    }
}

impl Clone for Function {
    fn clone(&self) -> Self {
        Self::new(self.0.clone(), self.1.clone())
    }
}

impl Elf {
    pub fn new(raw_data: &'static [u8]) -> Option<Self> {
        const ELF_SYM_STT_FUNC: u8 = 2;

        let data = match ElfBytes::<AnyEndian>::minimal_parse(raw_data) {
            Ok(o) => Some(o),
            Err(e) => {
                error!("Failed to parse file {}", e);
                None
            }
        }?;

        let (symtab, strtab) = data.symbol_table().ok()??;

        // If compiler does not set size for function, simply look up next label
        // in the same section
        let proccess_st_size = |mut sym: Symbol| {
            let mut next_sym: Option<Symbol> = None;

            if sym.st_size == 0 {
                for j in symtab.iter() {
                    if j.st_symtype() == ELF_SYM_STT_FUNC
                        && sym.st_shndx == sym.st_shndx
                        && j.st_value > sym.st_value
                    {
                        if let Some(s) = next_sym.as_ref() {
                            if j.st_value < s.st_value {
                                next_sym = Some(j)
                            }
                        } else {
                            next_sym = Some(j);
                        }
                    }
                }
                if let Some(s) = next_sym.as_ref() {
                    sym.st_size = s.st_value - sym.st_value;
                }
            }
            sym
        };

        Some(Self {
            functions: symtab
                .iter()
                .filter(|s| s.st_symtype() == ELF_SYM_STT_FUNC)
                .map(|sym| {
                    (
                        sym.st_value,
                        Function::new(
                            strtab
                                .get(sym.st_name as usize)
                                .unwrap_or("unknown")
                                .to_owned(),
                            proccess_st_size(sym),
                        ),
                    )
                })
                .collect(),
            sections: data.section_headers()?,
            data,
            debug_info: DwarfParser::new(raw_data),
        })
    }

    pub fn arch(&self) -> Arch {
        match self.data.ehdr.e_machine {
            0x3e => Arch::X86_64,
            0x03 => Arch::X86,
            0xb7 => Arch::Arm64,
            0x28 => Arch::Arm,
            0xF3 => Arch::Riscv,
            0x08 => Arch::Mips,
            _ => panic!("Consider using some other arch... =)"),
        }
    }

    pub fn function_by_addr(&self, addr: u64) -> Option<Function> {
        Some(self.functions.get(&addr)?.clone())
    }

    pub fn function_names(&self) -> Vec<Function> {
        self.functions.iter().map(|x| (*x.1).clone()).collect()
    }

    pub fn func_code(&self, addr: u64) -> (&[u8], u64) {
        match self.data.ehdr.e_type {
            elf::abi::ET_REL => self.func_code_reloc(addr),
            elf::abi::ET_DYN | elf::abi::ET_EXEC => self.func_code_exe(addr),
            _ => unreachable!(),
        }
    }

    fn func_code_reloc(&self, addr: u64) -> (&[u8], u64) {
        let func = self.functions.get(&addr).unwrap();

        return (
            &self
                .data
                .section_data(&self.sections.get(func.1.st_shndx as usize).unwrap())
                .unwrap()
                .0[func.1.st_value as usize..func.1.st_size as usize],
            func.1.st_value,
        );
    }

    fn func_code_exe(&self, addr: u64) -> (&[u8], u64) {
        let func = self.functions.get(&addr).unwrap();
        let sym = &func.1;
        let target_section = &self.sections.get(sym.st_shndx as usize).unwrap();

        let start = (sym.st_value - target_section.sh_addr) as usize;
        let end = start + sym.st_size as usize;

        let section_data = &self.data.section_data(target_section).unwrap().0;
        // crate::log_info!("start {} end {} len {}", start, end, section_data.len());
        if end > section_data.len() {
            (&section_data[0..0], sym.st_value)
        } else if start != end {
            (&section_data[start..end], sym.st_value)
        } else {
            (&section_data[start..], sym.st_value)
        }
    }

    pub fn function_debug_info(&self, f: Function) -> Option<FunctionDebugInfo> {
        let dw = self.debug_info.as_ref()?;

        dw.function_data(&f)
    }

    #[cfg(debug_assertions)]
    #[allow(dead_code)]
    fn section_name(&self, s: &SectionHeader) {
        let sh = self
            .data
            .section_header_by_name(".shstrtab")
            .unwrap()
            .unwrap();
        println!(
            "section {:?}",
            std::str::from_utf8(&self.data.section_data(&sh).unwrap().0[s.sh_name as usize..])
        );
    }
}
