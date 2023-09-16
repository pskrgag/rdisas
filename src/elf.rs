use elf::endian::AnyEndian;
use elf::file::*;
use elf::section::{SectionHeader, SectionHeaderTable};
use elf::symbol::Symbol;
use elf::ElfBytes;
use std::collections::HashMap;

// Name and section index
type Function = (String, u64);
type FunctionMap = HashMap<u64, Function>;

pub struct Functions<'a> {
    list: Vec<(&'a str, Symbol)>,
}

impl<'a> Functions<'a> {
    pub fn new(list: Vec<(&'a str, Symbol)>) -> Self {
        Self { list }
    }

    // refs didn't work.... I am too dumb at rust
    pub fn names(&self) -> Vec<String> {
        self.list.iter().map(|x| x.0.to_owned()).collect()
    }
}

pub struct Elf {
    data: ElfBytes<'static, AnyEndian>,
    sections: Option<SectionHeaderTable<'static, AnyEndian>>,
    functions: FunctionMap,
}

impl Elf {
    pub fn new(data: &'static [u8]) -> Option<Self> {
        const ELF_SYM_STT_FUNC: u8 = 2;

        let data = match ElfBytes::<AnyEndian>::minimal_parse(data) {
            Ok(o) => Some(o),
            Err(e) => {
                error!("Failed to parse file {}", e);
                None
            }
        }?;

        Self::check_header(&data)?;
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
        })
    }

    pub fn load_sections(&mut self) -> Option<()> {
        self.sections = Some(self.data.section_headers()?);
        Some(())
    }

    fn check_header(e: &ElfBytes<AnyEndian>) -> Option<()> {
        let hdr = e.ehdr;

        match hdr.class {
            Class::ELF64 => Some(()),
            _ => {
                error!("Elf header class is not 64bit");
                None
            }
        }?;

        match hdr.e_machine {
            183 | 62 => Some(()),
            other => {
                error!("Does not support {}", other);
                None
            }
        }?;

        Some(())
    }

    pub fn function_name_by_addr(&self, addr: u64) -> Option<String> {
        Some(self.functions.get(&addr)?.0.clone())
    }

    pub fn function_names(&self) -> Option<Functions> {
        const ELF_SYM_STT_FUNC: u8 = 2;

        // TODO: optimize that shit!!!!!
        if let Ok(Some((symtab, strtab))) = self.data.symbol_table() {
            Some(Functions::new(
                symtab
                    .iter()
                    .map(|sym| (strtab.get(sym.st_name as usize).unwrap_or("unknown"), sym))
                    .filter(|s| s.1.st_symtype() == ELF_SYM_STT_FUNC)
                    .collect::<Vec<(&str, Symbol)>>(),
            ))
        } else {
            None
        }
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
            if i.st_symtype() == ELF_SYM_STT_FUNC {
                if strtab.get(i.st_name as usize).unwrap() == name {
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
        }

        todo!();
    }

    fn func_code_exe(&self, name: &String) -> (&[u8], u64) {
        const ELF_SYM_STT_FUNC: u8 = 2;

        let (symtab, strtab) = self
            .data
            .symbol_table()
            .expect("Failed to get symbol table")
            .unwrap();

        for i in symtab {
            if i.st_symtype() == ELF_SYM_STT_FUNC && strtab.get(i.st_name as usize).unwrap() == name
            {
                let target_section = &self.sections.unwrap().get(i.st_shndx as usize).unwrap();

                let start = (i.st_value - target_section.sh_addr) as usize;
                let end = start + i.st_size as usize;

                crate::log_info!("Found {} at addr {}", name, i.st_value);

                return (
                    &self.data.section_data(target_section).unwrap().0[start..end],
                    i.st_value,
                );
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
