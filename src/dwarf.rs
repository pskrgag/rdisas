use object::{File, Object, ObjectSection};
use std::collections::HashMap;
use std::path::PathBuf;
use std::{borrow, path};

pub struct DwarfParser {
    obj: File<'static>,
    lines: HashMap<u64, (PathBuf, usize)>,
}

impl DwarfParser {
    pub fn new(data: &'static [u8]) -> Option<Self> {
        let obj = File::parse(data).ok()?;
        let mut map = HashMap::new();

        let load_section = |id: gimli::SectionId| -> Result<borrow::Cow<[u8]>, gimli::Error> {
            match obj.section_by_name(id.name()) {
                Some(ref section) => Ok(section
                    .uncompressed_data()
                    .unwrap_or(borrow::Cow::Borrowed(&[][..]))),
                None => Ok(borrow::Cow::Borrowed(&[][..])),
            }
        };
        let endian = if obj.is_little_endian() {
            gimli::RunTimeEndian::Little
        } else {
            gimli::RunTimeEndian::Big
        };

        let dwarf_cow = gimli::Dwarf::load(&load_section).ok()?;

        // Borrow a `Cow<[u8]>` to create an `EndianSlice`.
        let borrow_section: &dyn for<'a> Fn(
            &'a borrow::Cow<[u8]>,
        )
            -> gimli::EndianSlice<'a, gimli::RunTimeEndian> =
            &|section| gimli::EndianSlice::new(&*section, endian);

        // Create `EndianSlice`s for all of the sections.
        let dwarf = dwarf_cow.borrow(&borrow_section);

        // Iterate over the compilation units.
        let mut iter = dwarf.units();

        while let Some(header) = iter.next().ok()? {
            println!(
                "Line number info for unit at <.debug_info+0x{:x}>",
                header.offset().as_debug_info_offset().unwrap().0
            );
            let unit = dwarf.unit(header).ok()?;

            // Get the line program for the compilation unit.
            if let Some(program) = unit.line_program.clone() {
                let comp_dir = if let Some(ref dir) = unit.comp_dir {
                    path::PathBuf::from(dir.to_string_lossy().into_owned())
                } else {
                    path::PathBuf::new()
                };

                // Iterate over the line program rows.
                let mut rows = program.rows();
                while let Some((header, row)) = rows.next_row().ok()? {
                    if !row.end_sequence() {
                        // Determine the path. Real applications should cache this for performance.
                        let mut path = path::PathBuf::new();
                        if let Some(file) = row.file(header) {
                            path = comp_dir.clone();

                            // The directory index 0 is defined to correspond to the compilation unit directory.
                            if file.directory_index() != 0 {
                                if let Some(dir) = file.directory(header) {
                                    path.push(
                                        dwarf
                                            .attr_string(&unit, dir)
                                            .ok()?
                                            .to_string_lossy()
                                            .as_ref(),
                                    );
                                }
                            }

                            path.push(
                                dwarf
                                    .attr_string(&unit, file.path_name())
                                    .ok()?
                                    .to_string_lossy()
                                    .as_ref(),
                            );
                        }

                        // Determine line/column. DWARF line/column is never 0, so we use that
                        // but other applications may want to display this differently.
                        let line = match row.line() {
                            Some(line) => line.get(),
                            None => 0,
                        };

                        map.insert(row.address(), (path, line as usize));
                    }
                }
            }
        }

        Some(Self { obj, lines: map })
    }
}
