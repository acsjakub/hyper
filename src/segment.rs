use std::fmt;

pub struct Segment {
    id: u64,
    name: String,
    address: u64,
    len: usize,
    flags: String,
}

impl Segment {
    pub fn from_line(line: String, id: u64) -> Self {
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

#[cfg(test)]
mod test {
    use super::*;

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
}
