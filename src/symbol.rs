use std::fmt;

#[derive(Debug, PartialEq)]
pub struct Symbol {
    id: u64,
    name: String,
    value: u64,
    seg_id: u64,
    typ: String,
}

impl Symbol {
    pub fn from_line(line: String, id: u64) -> Self {
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

#[cfg(test)]
mod test {
    use super::*;

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
}
