#![feature(slice_pattern)]

use std::env;
use memmap::MmapOptions;

mod disas;
mod elf;
mod term;

#[macro_use]
extern crate log;

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
    }.unwrap();

    let mmap_data = match unsafe { MmapOptions::new().map(&file) } {
        Ok(f) => Some(f),
        Err(e) => {
            error!("Failed to map {}: {}", args[1], e);
            None
        }
    }.unwrap();

    let e = match elf::Elf::new(&*mmap_data) {
        Some(e) => Some(e),
        None => {
            error!("Failed to create elf");
            None
        }
    }.unwrap();

    let mut d = disas::Disas::new(args[1].clone(), e).unwrap();

    d.exec();
}
