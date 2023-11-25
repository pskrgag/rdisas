use memmap::MmapOptions;
use std::env;

mod app;
mod elf;
mod term;
mod dwarf;

#[macro_use]
extern crate log;
extern crate capstone;

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

    let mut e = match elf::Elf::new(Box::leak(Box::new(mmap_data))) {
        Some(e) => Some(e),
        None => {
            error!("Failed to create elf");
            None
        }
    }
    .unwrap();

    let mut app = app::App::new(e).unwrap();
    let mut tui = term::tui::Tui::new().unwrap();

    tui.draw(&mut app);

    loop {
        let e = tui.next_event(app.state());

        if app.proccess_event(e) {
            break;
        }

        tui.draw(&mut app);
    }
}
