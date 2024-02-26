use std::fmt;
use std::num::ParseIntError;

#[derive(Debug, PartialEq)]
pub struct Symbol {
    id: u64,
    name: String,
    pub value: u64,
    seg_id: u64,
    pub typ: String,
}

#[derive(Debug, PartialEq)]
pub struct ParseError {
    msg: String,
}

impl ParseError {
    pub fn from_parseint(_err: ParseIntError) -> Self {
        Self {
            msg: String::from("Could not parse int"),
        }
    }
}

impl Symbol {
    fn address_from_str(addr: &str) -> Result<u64, ParseError> {
        Ok(u64::from_str_radix(addr, 16).map_err(ParseError::from_parseint)?)
    }

    pub fn from_line(line: String, id: u64) -> Self {
        let mut it = line.split(' ');
        let fields: Vec<&str> = line.split_whitespace().collect();
        match fields.len() {
            4.. => {
                Self {
                    id: id,
                    name: fields[0].into(),
                    value: Self::address_from_str(fields[1].into()).unwrap(),
                    seg_id: fields[2].parse::<u64>().unwrap(),
                    typ: fields[3].into(),
                }
            },
            _ => panic!("wrong numbeer of fields for symbol record")
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
