use crate::error::ParseError;
use std::fmt;

#[derive(Debug, PartialEq)]
pub struct Segment {
    pub id: u64,
    pub name: String,
    pub address: u64,
    pub len: usize,
    pub flags: String,
}

impl Segment {
    fn address_from_str(addr: &str) -> Result<u64, ParseError> {
        Ok(u64::from_str_radix(addr, 16).map_err(ParseError::from_parseint)?)
    }
    pub fn from(id: u64, name: &str, address: u64, len: usize, flags: &str) -> Self {
        Self {
            id,
            name: name.to_string(),
            address,
            len,
            flags: flags.to_string(),
        }
    }

    pub fn from_line(line: String, id: u64) -> Result<Self, ParseError> {
        let fields: Vec<&str> = line.split_whitespace().collect();
        match fields.len() {
            4.. => Ok(Self {
                id: id,
                name: String::from(fields[0]),
                address: Self::address_from_str(fields[1])?,
                len: fields[2]
                    .parse::<u64>()
                    .map_err(ParseError::from_parseint)? as usize,
                flags: String::from(fields[3]),
            }),
            _ => Err(ParseError::from("did not match fields")),
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_segment_display() {
        let segment_line = String::from(".seg 1337dead 4000 RW");
        let segment = Segment::from_line(segment_line.clone(), 3).unwrap();
        assert_eq!(format!("{}", segment), segment_line);
    }

    #[test]
    fn test_valid_line() {
        let segment_line = String::from(".seg 1337dead 4000 RW");
        let segment = Segment::from_line(segment_line, 3).unwrap();
        assert_eq!(segment.id, 3);
        assert_eq!(segment.name, ".seg");
        assert_eq!(segment.address, 0x1337dead);
        assert_eq!(segment.len, 4000);
        assert_eq!(segment.flags, "RW");
    }

    #[test]
    fn test_short_line() {
        let segment_line = String::from(".seg 14325 4000");
        let _ = Segment::from_line(segment_line, 1);
    }

    #[test]
    fn test_invalid_address_line() {
        let segment_line = String::from(".seg g14325 4000 RD");
        let segment = Segment::from_line(segment_line, 1);
        assert_eq!(segment, Err(ParseError::from("Could not parse int")),);
    }
}
