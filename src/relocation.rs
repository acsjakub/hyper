use std::fmt;

#[derive(Debug, PartialEq)]
pub struct Relocation {
    loc: u64,
    seg_id: u64,
    reference: u64,
    typ: String,
}

impl Relocation {
    pub fn from_line(line: String) -> Self {
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
mod test {
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
}
