use memmap::MmapOptions;
use std::env;

mod disas;
mod elf;
mod term;

#[macro_use]
extern crate log;
extern crate capstone;

#[macro_use]
extern crate lazy_static;

fn main() {
    env_logger::init();

    let args = env::args().collect::<Vec<String>>();

    if args.len() != 2 {
        error!("usage: {} <file>", args[0]);
        return;
    }

    let file = match std::fs::File::open(&args[1]) {
        Ok(f) => Some(f),
        Err(e) => {
            error!("Failed to open {}: {}", args[1], e);
            None
        }
    }
    .unwrap();

    let mmap_data = match unsafe { MmapOptions::new().map(&file) } {
        Ok(f) => Some(f),
        Err(e) => {
            error!("Failed to map {}: {}", args[1], e);
            None
        }
    }
    .unwrap();

    let e = match elf::Elf::new(Box::leak(Box::new(mmap_data))) {
        Some(e) => Some(e),
        None => {
            error!("Failed to create elf");
            None
        }
    }
    .unwrap();

    let d = disas::Disas::new(args[1].clone(), e).unwrap();

    d.exec();
}
