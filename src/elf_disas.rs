use capstone::prelude::*;
use capstone::prelude::*;

pub struct CapstoneWrapper {
    c: Capstone,
}

impl CapstoneWrapper {
    pub fn new_aarch64() -> Option<Self> {
        Some(Self {
            c: Capstone::new()
                .arm64()
                .mode(arch::arm64::ArchMode::Arm)
                .build()
                .ok()?,
        })
    }

    pub fn new_x86() -> Option<Self> {
        Some(Self {
            c: Capstone::new()
                .x86()
                .mode(arch::x86::ArchMode::Mode64)
                .build()
                .ok()?,
        })
    }

    pub fn disas_all(&self, code: &[u8], addr: u64) -> Vec<String> {
        let insts = self.c.disasm_all(code, addr).unwrap();
        let mut res = Vec::new();

        for i in insts.as_ref() {
            res.push(i.to_string());
        }

        res
    }
}
