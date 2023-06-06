use elf::ElfBytes;
use elf::endian::AnyEndian;
use elf::file::*;
use elf::symbol::Symbol;
use elf::section::SectionHeaderTable;

pub struct Functions<'a> {
    list: Vec<(&'a str, Symbol)>,
}

impl<'a> Functions<'a> {
    pub fn new(list: Vec<(&'a str, Symbol)>) -> Self {
        Self { list: list }
    }

    // refs didn't work.... I am too dumbat rust
    pub fn names(&self) -> Vec<String> {
        self.list.iter().map(|x| x.0.to_owned()).collect()
    }
}

pub struct Elf<'a> {
   data: ElfBytes::<'a, AnyEndian>,
   sections: Option<SectionHeaderTable<'a, AnyEndian>>,
}

impl<'a> Elf<'a> {
    pub fn new(data: &'a [u8]) -> Option<Self> {
        let data = match ElfBytes::<AnyEndian>::minimal_parse(data) {
            Ok(o) => Some(o),
            Err(e) => {
                error!("Failed to parse file {}", e);
                None
            }
        }?;

        Self::check_header(&data)?;

        Some(Self {data: data, sections: None} )
    }

    pub fn get_raw_section(&self, idx: usize) -> Option<&'a [u8]> {
        let s = self.sections.unwrap().get(idx).ok()?;

        Some(self.data.section_data(&s).ok()?.0)
    }

    pub fn load_sections(&mut self) -> Option<()> {
        self.sections = Some(self.data.section_headers()?);
        Some(())
    }

    pub fn check_header(e: &ElfBytes<AnyEndian>) -> Option<()> {
        let hdr = e.ehdr;

        match hdr.class {
            Class::ELF64 => Some(()),
            _ => {
                error!("Elf header class is not 64bit");
                None
            }
        }?;

        match hdr.e_machine {
            183 => Some(()),
            other => {
                error!("Elf file is not for EM_ARM {}", other);
                None
            }
        }?;

        Some(())
    }

    pub fn function_names(&self) -> Option<Functions> {
        const ELF_SYM_STT_FUNC: u8 = 2;

        if let Ok(Some((symtab, strtab))) = self.data.symbol_table() {
            Some(Functions::new(symtab
                .iter()
                .map(|sym| (strtab.get(sym.st_name as usize).unwrap_or("unknown"), sym))
                .filter(|s| s.1.st_symtype() == ELF_SYM_STT_FUNC)
                .collect::<Vec<(&str, Symbol)>>()))
        } else {
            None
        }
    }
}
