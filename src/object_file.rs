use std::fmt;
use std::fs::File;
use std::io::{BufRead, BufReader};

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
struct ObjectFile {
    segments: Vec<Segment>,
    symbols: Vec<Symbol>,
    rels: Vec<Relocation>,
    data: Vec<u8>,
}

impl ObjectFile {
    // todo: make this return Result with correct type
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
        let mut segments = Vec::new();
        let mut symbols = Vec::new();
        let mut rels = Vec::new();

        // todo this better
        for i in 0..nums[0] {
            line = String::new();
            reader.read_line(&mut line).unwrap();
            segments.push(Segment::from_line(line, i).unwrap());
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
            write!(f, "{}", segment)?;
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
}
