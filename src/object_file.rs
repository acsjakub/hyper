use std::collections::HashMap;
use std::fmt;
use std::fs::File;
use std::io::{BufRead, BufReader};

use crate::error::{LinkError, LoadError};
use crate::relocation::Relocation;
use crate::segment::Segment;
use crate::symbol::Symbol;

#[derive(Debug, PartialEq)]
struct ObjectFile {
    segments: HashMap<String, Segment>,
    symbols: Vec<Symbol>,
    rels: Vec<Relocation>,
    data: Vec<u8>,
    rt_seg_alloc_map: HashMap<u64, u64>,
}

impl ObjectFile {
    const STARTING_ADDRESS: u64 = 0x1000;
    const SEGMENT_ALIGNMENT: u64 = 0x1000;

    fn from_file(input_path: &str) -> Result<Self, LoadError> {
        let file = File::open(input_path).unwrap();
        let mut reader = BufReader::new(file);

        let mut line = String::new();
        reader.read_line(&mut line).unwrap();
        if line.trim() != "LINK" {
            return Err(LoadError::from("Invalid Magic, expected: 'LINK'"));
        }
        line = String::new();
        reader.read_line(&mut line).unwrap();
        let nums: Vec<u64> = line
            .trim()
            .split(' ')
            .map(str::parse)
            .collect::<Result<Vec<u64>, _>>()
            .unwrap();
        if nums.len() != 3 {
            return Err(LoadError::from("Expected three numbers on line 2"));
        }
        let mut segments = HashMap::new();
        let mut symbols = Vec::new();
        let mut rels = Vec::new();

        // todo this better
        for i in 0..nums[0] {
            line = String::new();
            reader.read_line(&mut line).unwrap();
            let segment = Segment::from_line(line, i).unwrap();
            segments.insert(segment.name.clone(), segment);
        }

        for i in 0..nums[1] {
            line = String::new();
            reader.read_line(&mut line).unwrap();
            symbols.push(Symbol::from_line(line, i).unwrap());
        }

        for _ in 0..nums[2] {
            line = String::new();
            reader.read_line(&mut line).unwrap();
            rels.push(Relocation::from_line(line));
        }

        Ok(Self {
            segments: segments,
            symbols: symbols,
            rels: rels,
            data: Vec::new(),
            rt_seg_alloc_map: HashMap::new(),
        })
    }

    fn merge_segments(obj_files: &mut Vec<ObjectFile>, seg_sizes: &HashMap<String, usize>) -> HashMap<String, Segment> {
        let mut result_segments = HashMap::new();
        let mut next_segment_address: u64 = Self::STARTING_ADDRESS;
        let mut next_free_id: u64 = 1;

        // create segments for the resulting file by
        // merging segments from input files with the same name

        for file in obj_files {
            for (name, segment) in &file.segments {
                let r_segment: &mut Segment = match result_segments.get_mut(name) {
                    Some(segment) => segment,
                    None => {
                        result_segments.insert(
                            name.into(),
                            Segment::from(
                                next_free_id,
                                name,
                                next_segment_address,
                                0,
                                &segment.flags,
                            ),
                        );
                        let seg_len = *seg_sizes.get(name.into()).unwrap() as u64;
                        next_segment_address =
                            seg_len + (Self::SEGMENT_ALIGNMENT - (seg_len % Self::SEGMENT_ALIGNMENT));
                        next_free_id += 1;
                        result_segments.get_mut(name).unwrap()
                    }
                };
                file.rt_seg_alloc_map
                    .insert(segment.id, r_segment.address + r_segment.len as u64);
                r_segment.len += segment.len;
            }
        }
        result_segments
    }

    fn add_common_block(obj_files: &Vec<ObjectFile>, result_segments: &mut HashMap<String, Segment>) {

        // for exercise 4.2
        // adding common blocks
        // go through symbol table and identify non-zero undefined symbols
        // (they have 0 as a segment number and U as a type)
        // add space of appropriate size to the .bss segment
        //
        //

        let mut common_block_size = 0;
        for file in obj_files {
            for symbol in &file.symbols {
                if symbol.typ == String::from("U") && symbol.value > common_block_size {
                    common_block_size = symbol.value;
                }
            }
        }

        let total_len = result_segments.iter().map(|(_,s)| s.len).sum::<usize>() as u64;
        let next_id = result_segments.len() as u64 + 1;
        result_segments
            .entry(".bss".into())
            .or_insert(Segment {
                id: next_id,
                name: String::from(".bss"),
                address: total_len,
                len: 0,
                flags: String::from("RW"),
            })
            .len += common_block_size as usize;
    }

    // creates new segment allocation map and stores
    // information about new locations for each segment
    // in the rt_seg_alloc_map structure of each respective file
    fn allocate_segments(
        obj_files: &mut Vec<ObjectFile>,
    ) -> Result<HashMap<String, Segment>, LinkError> {
        let mut seg_sizes = HashMap::new();

        for file in obj_files.iter() {
            for (_, segment) in &file.segments {
                *seg_sizes.entry(segment.name.clone()).or_insert(0) += segment.len
            }
        }
        println!("{:?}", seg_sizes);

        let mut result_segments = Self::merge_segments(obj_files, &seg_sizes);
        Self::add_common_block(obj_files, &mut result_segments);

        Ok(result_segments)
    }

    fn link(obj_files: Vec<ObjectFile>) -> Result<ObjectFile, LinkError> {
        let mut objects = obj_files;
        // create global symbol table
        // what needs to be in the per object symbol table?
        Ok(Self {
            segments: Self::allocate_segments(&mut objects)?,
            symbols: Vec::new(),
            rels: Vec::new(),
            data: Vec::new(),
            rt_seg_alloc_map: HashMap::new(),
        })
    }
}

impl fmt::Display for ObjectFile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "LINK")?;
        writeln!(
            f,
            "{} {} {}",
            self.segments.len(),
            self.symbols.len(),
            self.rels.len()
        )?;
        for segment in &self.segments {
            writeln!(f, "{}", segment.1)?;
        }
        for symbol in &self.symbols {
            writeln!(f, "{}", symbol)?;
        }
        for rel in &self.rels {
            write!(f, "{}", rel)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn create_object_file(content: &str, filename: &str) -> Result<ObjectFile, LoadError> {
        let mut output = File::create(filename).unwrap();
        write!(output, "{}", content).unwrap();
        ObjectFile::from_file(filename)
    }

    #[test]
    fn test_object_file_load() {
        let content = "LINK\n\
                      1 1 1\n\
                      .seg 1337dead 4000 RW\n\
                      sym deadbeef 2 RD\n\
                      1 2 3 RD\n";
        let object_file = create_object_file(content, "testfile").unwrap();
        assert_eq!(format!("{}", object_file), content);
    }

    #[test]
    fn test_object_file_wrong_magic() {
        let content = "LNK\n";
        let object_file = create_object_file(content, "testfile1");
        assert_eq!(
            object_file,
            Err(LoadError::from("Invalid Magic, expected: 'LINK'"))
        )
    }

    #[test]
    fn test_object_file_too_few_counts() {
        let content = "LINK\n\
                       1 1\n";
        let object_file = create_object_file(content, "testfile2");
        assert_eq!(
            object_file,
            Err(LoadError::from("Expected three numbers on line 2"))
        )
    }

    #[test]
    fn test_two_obj_files_linkage() {
        let content = "LINK\n\
                       3 0 0\n\
                       .text 1000 2000 R\n\
                       .data 3000 1000 RW\n\
                       .bss  4000 200 RW\n";
        let objfile1 = create_object_file(content, "objfile1").unwrap();
        let objfile2 = create_object_file(content, "objfile2").unwrap();
        let linkable = vec![objfile1, objfile2];
        ObjectFile::link(linkable).unwrap();
    }

    #[test]
    fn test_basic_segment_allocation() {
        let content = "LINK\n\
                       3 0 0\n\
                       .text 1000 2000 R\n\
                       .data 3000 1000 RW\n\
                       .bss  4000 200 RW\n";
        let objfile1 = create_object_file(content, "objfile1").unwrap();
        let content = "LINK\n\
                       3 0 0\n\
                       .text 1000 6000 R\n\
                       .data 3000 500 RW\n\
                       .bss  4000 2000 RW\n";

        let objfile2 = create_object_file(content, "objfile2").unwrap();
        let linkable = vec![objfile1, objfile2];
        let result = ObjectFile::link(linkable).unwrap();
        assert_eq!(result.segments[".bss"].len, 2200);
        assert_eq!(result.segments[".data"].len, 1500);
        assert_eq!(result.segments[".text"].len, 8000);
    }

    #[test]
    fn test_common_segment_allocation() {
        let content = "LINK\n\
                       2 3 0\n\
                       .text 1000 4000 R\n\
                       .data 5000 1000 RW\n\
                       symbol_undef 300 0 U\n\
                       symbol_def cafebabe 1 D\n\
                       symbol_zero 0 0 U\n";
        let objfile1 = create_object_file(content, "/tmp/test_objfile1").unwrap();
        let content = "LINK\n\
                       3 2 0\n\
                       .text 1000 5000 R\n\
                       .data 6000 2000 RW\n\
                       .bss  8000 200  RW\n\
                       symbol_next_undef 100 0 U\n\
                       symbol_known 1442dead 1 D\n";
        let objfile2 = create_object_file(content, "/tmp/test_objfile2").unwrap();
        let linkable = vec![objfile1, objfile2];
        let result = ObjectFile::link(linkable).unwrap();
        assert_eq!(result.segments[".bss"].len, 200 + 0x300);
    }
}
