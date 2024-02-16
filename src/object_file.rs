use std::fmt;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::collections::HashMap;

use crate::relocation::Relocation;
use crate::segment::Segment;
use crate::symbol::Symbol;

#[derive(Debug, PartialEq)]
struct LoadError {
    msg: String
}

impl LoadError {
    fn from(msg: &str) -> Self {
        Self {
            msg: String::from(msg)
        }
    }
}

#[derive(Debug, PartialEq)]
struct LinkError;


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
            symbols.push(Symbol::from_line(line, i));
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

    fn link(obj_files: Vec<ObjectFile>) {//Result<ObjectFile, LinkError> {
        let result: Self;
        let mut segments: Vec<Segment> = Vec::new();
        let mut seg_total_sizes: HashMap<String, usize> = HashMap::new();
        for file in obj_files {
            for segment in file.segments {
                *seg_total_sizes.entry(segment.1.name).or_insert(0) += segment.1.len;
            }
        }
        println!("{:?}", seg_total_sizes);
        // think about this, maybe it makes much more sense for the segments to be stored in a
        // hashmap, keyed with the name, rather than vector
        // for each segment found in obj_files' segments, let's check if there already is a segment with
        // such name,if yes, then we want to append to that segment
        // but this would mean we have to shift segments, we can also just remember the starting offsets
        // or we can sort the segments in the vector and in second pass merge the neighbours with same name
        //
        // for  now, we'll ignore the flags and address of the segment, just merge it
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
            write!(f, "{}", symbol)?;
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
        assert_eq!(object_file, Err(LoadError::from("Invalid Magic, expected: 'LINK'")))
    }

    #[test]
    fn test_object_file_too_few_counts() {
        let content = "LINK\n\
                       1 1\n";
        let object_file = create_object_file(content, "testfile2");
        assert_eq!(object_file, Err(LoadError::from("Expected three numbers on line 2")))
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
        let result = ObjectFile::link(linkable);


    }
}
