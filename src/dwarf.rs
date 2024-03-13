use crate::elf::Function;
use object::{File, Object, ObjectSection};
use std::collections::HashMap;
use std::ops::Range;
use std::path::PathBuf;
use std::{borrow, path};

pub struct DwarfParser {
    obj: File<'static>,
    addr_to_line: HashMap<Range<u64>, (PathBuf, usize)>,
    line_to_addr: HashMap<usize, Vec<Range<u64>>>,
}

pub struct FunctionDebugInfo {
    path: PathBuf,
    addr_to_line: HashMap<Range<u64>, usize>, // addr -> line
    line_to_addrs: HashMap<usize, Vec<Range<u64>>>, // line -> addr
}

impl FunctionDebugInfo {
    pub(crate) fn new(
        data: (
            PathBuf,
            HashMap<Range<u64>, usize>,
            HashMap<usize, Vec<Range<u64>>>,
        ),
    ) -> Self {
        Self {
            path: data.0,
            addr_to_line: data.1,
            line_to_addrs: data.2,
        }
    }

    pub fn line_to_addrs(&self, line: usize) -> Option<&Vec<Range<u64>>> {
        self.line_to_addrs.get(&line)
    }

    pub fn line_by_addr(&self, addr: u64) -> Option<&usize> {
        Some(self.addr_to_line.iter().find(|x| x.0.contains(&addr))?.1)
    }

    pub fn file_name(&self) -> &PathBuf {
        &self.path
    }

    pub fn line_range(&self) -> (usize, usize) {
        let mut min: usize = usize::MAX;
        let mut max: usize = 0;

        for i in self.addr_to_line.iter() {
            min = usize::min(min, *i.1);
            max = usize::max(max, *i.1);
        }

        (min, max)
    }
}

impl DwarfParser {
    pub fn function_data(&self, f: &Function) -> Option<FunctionDebugInfo> {
        use std::fs::write;

        write(
            "test.log",
            format!("{:?}\n\n{:?}", self.line_to_addr, f.addr()),
        );
        let pb = self.addr_to_line.iter().find(|x| x.0.contains(&f.addr()))?;

        let addr_to_line: HashMap<_, _> = self
            .addr_to_line
            .iter()
            .filter(|x| x.0.start >= f.addr() && x.0.end <= f.addr() + f.size() as u64)
            .map(|x| (x.0.clone(), x.1 .1))
            .collect();

        Some(FunctionDebugInfo::new((
            pb.1 .0.clone(),
            addr_to_line,
            self.line_to_addr.clone(),
        )))
    }

    pub fn new(data: &'static [u8]) -> Option<Self> {
        let obj = File::parse(data).ok()?;
        let mut addr_to_line = HashMap::new();
        let mut line_to_addr: HashMap<usize, Vec<Range<u64>>> = HashMap::new();

        let mut tmp = Vec::new();

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

                        tmp.push(((path, line as usize), row.address()));
                    }
                }
            }
        }

        // TODO: refactor it one day, please....
        for i in tmp.as_slice().windows(2) {
            addr_to_line.insert(i[0].1..i[1].1, i[0].0.clone());

            if let Some(line) = line_to_addr.get_mut(&i[0].0 .1) {
                line.push(i[0].1..i[1].1);
            } else {
                line_to_addr.insert(i[0].0 .1, vec![(i[0].1..i[1].1)]);
            }
        }

        use std::fs::write;

        Some(Self {
            obj,
            addr_to_line,
            line_to_addr,
        })
    }
}
