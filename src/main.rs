use std::fmt;
use std::fs::{read_to_string, File};
use std::io::{BufRead, BufReader};

struct Segment {
    id: u64,
    name: String,
    address: u64,
    len: usize,
    flags: String,
}

impl Segment {
    fn from_line(line: String, id: u64) -> Self {
        let mut it = line.split(' ');
        Self {
            id: id,
            name: it.next().unwrap().into(),
            address: u64::from_str_radix(it.next().unwrap(), 16).unwrap(),
            len: it.next().unwrap().parse::<u64>().unwrap() as usize,
            flags: it.next().unwrap().into(),
        }
    }
}

impl fmt::Display for Segment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {:x} {} {}",
            self.name, self.address, self.len, self.flags
        )
    }
}

struct Symbol {
    id: u64,
    name: String,
    value: u64,
    seg_id: u64,
    typ: String,
}

impl Symbol {
    fn from_line(line: String, id: u64) -> Self {
        let mut it = line.split(' ');
        Self {
            id: id,
            name: it.next().unwrap().into(),
            value: u64::from_str_radix(it.next().unwrap(), 16).unwrap(),
            seg_id: it.next().unwrap().parse::<u64>().unwrap(),
            typ: it.next().unwrap().into(),
        }
    }
}

impl fmt::Display for Symbol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {:x} {} {}",
            self.name, self.value, self.seg_id, self.typ
        )
    }
}

struct Relocation {
    loc: u64,
    seg_id: u64,
    reference: u64,
    typ: String,
}

impl Relocation {
    fn from_line(line: String) -> Self {
        let mut it = line.split(' ');
        Self {
            loc: it.next().unwrap().parse::<u64>().unwrap(),
            seg_id: it.next().unwrap().parse::<u64>().unwrap(),
            reference: it.next().unwrap().parse::<u64>().unwrap(),
            typ: it.next().unwrap().into(),
        }
    }
}

impl fmt::Display for Relocation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {} {} {}",
            self.loc, self.seg_id, self.reference, self.typ
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_relocation_from_line() {
        let rel_line = String::from("1 2 3 RD");
        let rel = Relocation::from_line(rel_line);
        assert_eq!(rel.loc, 1);
        assert_eq!(rel.seg_id, 2);
        assert_eq!(rel.reference, 3);
        assert_eq!(rel.typ, "RD");
    }

    #[test]
    fn test_relocation_display() {
        let rel_line = String::from("1 2 3 RD");
        let rel = Relocation::from_line(rel_line.clone());
        assert_eq!(format!("{}", rel), rel_line);
    }

    #[test]
    fn test_symbol_from_line() {
        let symbol_line = String::from("name deadbeef 2 RD");
        let symbol = Symbol::from_line(symbol_line, 1);
        assert_eq!(symbol.id, 1);
        assert_eq!(symbol.name, "name");
        assert_eq!(symbol.value, 0xdeadbeef);
        assert_eq!(symbol.seg_id, 2);
        assert_eq!(symbol.typ, "RD");
    }

    #[test]
    fn test_symbol_display() {
        let symbol_line = String::from("name deadbeef 2 RD");
        let symbol = Symbol::from_line(symbol_line.clone(), 1);
        assert_eq!(format!("{}", symbol), symbol_line);
    }

    #[test]
    fn test_segment_from_line() {
        let segment_line = String::from(".seg 1337dead 4000 RW");
        let segment = Segment::from_line(segment_line, 3);
        assert_eq!(segment.id, 3);
        assert_eq!(segment.name, ".seg");
        assert_eq!(segment.address, 0x1337dead);
        assert_eq!(segment.len, 4000);
        assert_eq!(segment.flags, "RW");
    }

    #[test]
    fn test_segment_display() {
        let segment_line = String::from(".seg 1337dead 4000 RW");
        let segment = Segment::from_line(segment_line.clone(), 3);
        assert_eq!(format!("{}", segment), segment_line);
    }

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

fn main() {
    let content = read_to_string("testfile").unwrap();
    println!("{}", content);
}
