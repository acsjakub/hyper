use std::collections::HashMap;
use std::fmt;
use std::fs::File;
use std::io::{BufRead, BufReader};

use crate::relocation::Relocation;
use crate::segment::Segment;
use crate::symbol::Symbol;
use crate::error::{LoadError, LinkError};

#[derive(Debug, PartialEq)]
struct ObjectFile {
    segments: HashMap<String, Segment>,
    symbols: Vec<Symbol>,
    rels: Vec<Relocation>,
    data: Vec<u8>,
}

impl ObjectFile {
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
        })
    }

    fn link(obj_files: Vec<ObjectFile>) -> Result<ObjectFile, LinkError> {
        let mut result_segments = HashMap::new();
        let mut segs_grouped_by_name: HashMap<String, Vec<&Segment>> = HashMap::new();

        for file in &obj_files {
            for (_, segment) in &file.segments {
                segs_grouped_by_name
                    .entry(segment.name.clone())
                    .or_insert(Vec::new())
                    .push(segment);
            }
        }
        let mut next_free_address: u64 = 0x1000;
        let mut next_free_id: u64 = 1;

        for (name, segments) in segs_grouped_by_name {
            let seg = Segment {
                id: next_free_id,
                name: name.clone(),
                address: next_free_address,
                len: segments.iter().map(|s| s.len).sum(),
                flags: String::from(&segments[0].flags), //this assumes segments with the same name have same name
            };
            next_free_address += seg.len as u64 + (0x1000 - (seg.len as u64 % 0x1000));
            result_segments.insert(name, seg);
            next_free_id += 1;
        }

        // for exercise 4.2
        // adding common blocks
        // go through symbol table and identify non-zero undefined symbols
        // (they have 0 as a segment number and U as a type)
        // add space of appropriate size to the .bss segment

        let mut common_block_size = 0;
        for file in &obj_files {
            for symbol in &file.symbols {
                if symbol.typ == String::from("U") && symbol.value > common_block_size {
                    common_block_size = symbol.value;
                }
            }
        }
        result_segments.entry(".bss".into()).or_insert(Segment {
            id: next_free_id,
            name: String::from(".bss"),
            address: next_free_address,
            len: 0,
            flags: String::from("RW"),

        }).len += common_block_size as usize;
        Ok(Self {
            segments: result_segments,
            symbols: Vec::new(),
            rels: Vec::new(),
            data: Vec::new(),
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
        let  objfile2 = create_object_file(content, "/tmp/test_objfile2").unwrap();
        let linkable = vec![objfile1, objfile2];
        let result = ObjectFile::link(linkable).unwrap();
        assert_eq!(result.segments[".bss"].len, 200 + 0x300);



    }
}
