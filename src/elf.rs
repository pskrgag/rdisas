use elf::endian::AnyEndian;
use elf::section::{SectionHeader, SectionHeaderTable};
use elf::symbol::Symbol;
use elf::ElfBytes;
use std::collections::HashMap;
use crate::dwarf::DwarfParser;

// Name and section index
type Function = (String, u64);
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
    sections: Option<SectionHeaderTable<'static, AnyEndian>>,
    functions: FunctionMap,
    debug_info: DwarfParser,
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

        Some(Self {
            functions: symtab
                .iter()
                .filter(|s| s.st_symtype() == ELF_SYM_STT_FUNC)
                .map(|sym| {
                    (
                        sym.st_value,
                        (
                            strtab
                                .get(sym.st_name as usize)
                                .unwrap_or("unknown")
                                .to_owned(),
                            sym.st_shndx as u64,
                        ),
                    )
                })
                .collect(),
            data,
            sections: None,
            debug_info: DwarfParser::new(raw_data)?,
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
            _ => panic!("How did I end up here?"),
        }
    }

    pub fn load_sections(&mut self) -> Option<()> {
        self.sections = Some(self.data.section_headers()?);
        Some(())
    }

    pub fn function_name_by_addr(&self, addr: u64) -> Option<String> {
        Some(self.functions.get(&addr)?.0.clone())
    }

    pub fn function_names(&self) -> Vec<String> {
        self.functions.iter().map(|x| x.1.0.clone()).collect()
    }

    pub fn func_code(&self, name: &String) -> (&[u8], u64) {
        match self.data.ehdr.e_type {
            elf::abi::ET_REL => self.func_code_reloc(name),
            elf::abi::ET_DYN | elf::abi::ET_EXEC => self.func_code_exe(name),
            _ => unreachable!(),
        }
    }

    fn func_code_reloc(&self, name: &String) -> (&[u8], u64) {
        const ELF_SYM_STT_FUNC: u8 = 2;

        let (symtab, strtab) = self
            .data
            .symbol_table()
            .expect("Failed to get symbol table")
            .unwrap();

        for i in symtab {
            if i.st_symtype() == ELF_SYM_STT_FUNC && strtab.get(i.st_name as usize).unwrap() == name
            {
                return (
                    &self
                        .data
                        .section_data(&self.sections.unwrap().get(i.st_shndx as usize).unwrap())
                        .unwrap()
                        .0[i.st_value as usize..i.st_size as usize],
                    i.st_value,
                );
            }
        }

        panic!("Something gone wrong....");
    }

    fn func_code_exe(&self, name: &String) -> (&[u8], u64) {
        const ELF_SYM_STT_FUNC: u8 = 2;

        let (symtab, strtab) = self
            .data
            .symbol_table()
            .expect("Failed to get symbol table")
            .unwrap();

        for i in symtab.iter() {
            if i.st_symtype() == ELF_SYM_STT_FUNC && strtab.get(i.st_name as usize).unwrap() == name
            {
                let target_section = &self.sections.unwrap().get(i.st_shndx as usize).unwrap();

                let start = (i.st_value - target_section.sh_addr) as usize;
                let mut end = start + i.st_size as usize;

                // If compiler does not set size for function, simply look up next label
                // in the same section
                if start == end {
                    let mut next_sym: Option<Symbol> = None;

                    for j in symtab {
                        if j.st_symtype() == ELF_SYM_STT_FUNC
                            && i.st_shndx == j.st_shndx
                            && j.st_value > i.st_value
                        {
                            if let Some(s) = next_sym.as_ref() {
                                if j.st_value < s.st_value {
                                    next_sym = Some(j)
                                }
                            } else {
                                next_sym = Some(j);
                            }
                        }

                        if let Some(s) = next_sym.as_ref() {
                            end = (s.st_value - target_section.sh_addr) as usize;
                        }
                    }
                }

                if start != end {
                    return (
                        &self.data.section_data(target_section).unwrap().0[start..end],
                        i.st_value,
                    );
                } else {
                    return (
                        &self.data.section_data(target_section).unwrap().0[start..],
                        i.st_value,
                    );
                }
            }
        }

        todo!()
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
