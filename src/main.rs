use std::fmt;
use std::fs::{read_to_string, File};
use std::io::{BufRead, BufReader, Write};
use crate::segment::Segment;

mod relocation;
mod segment;
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_object_file_load() {
        let object = "LINK\n\
                      1 1 1\n\
                      .seg 1337dead 4000 RW\n\
                      sym deadbeef 2 RD\n\
                      1 2 3 RD\n";
        let path = "testfile";
        let mut output = File::create(path).unwrap();
        write!(output, "{}", object);
        let object_file = ObjectFile::from_file(path);
        assert_eq!(format!("{}", object_file), object);
    }
}

struct ObjectFile {
    segments: Vec<Segment>,
    symbols: Vec<Symbol>,
    rels: Vec<Relocation>,
    data: Vec<u8>,
}

// todo implement consuming one line read for symbol, for segment, for relocation

impl ObjectFile {
    // todo: make this return Result with correct type
    fn from_file(input_path: &str) -> Self {
        let file = File::open(input_path).unwrap();
        let mut reader = BufReader::new(file);

        let mut line = String::new();
        reader.read_line(&mut line).unwrap();
        if line.trim() != "LINK" {
            panic!("Invalid Magic, expected: 'LINK'")
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
            panic!("Expected three numbers on line 2")
        }
        let mut segments = Vec::new();
        let mut symbols = Vec::new();
        let mut rels = Vec::new();

        // todo this better
        for i in 0..nums[0] {
            line = String::new();
            reader.read_line(&mut line).unwrap();
            segments.push(Segment::from_line(line, i));
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

        Self {
            segments: segments,
            symbols: symbols,
            rels: rels,
            data: Vec::new(),
        }
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
        for symbol in &self.symbols{
            write!(f, "{}", symbol)?;
        }
        for rel in &self.rels {
            write!(f, "{}", rel)?;
        }
        Ok(())
    }
}
mod symbol;

fn main() {
    let content = read_to_string("testfile").unwrap();
    println!("{}", content);
}
