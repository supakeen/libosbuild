use std::fmt;
use std::ops;

pub enum Part {
    Name(String),
    Index(usize),
}

pub struct Path(pub Vec<Part>);

impl Path {
    pub fn new(path: Vec<Part>) -> Self {
        Self { 0: path }
    }
}

impl ops::Deref for Path {
    type Target = Vec<Part>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl fmt::Display for Path {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.is_empty() {
            write!(f, ".")
        } else {
            self.iter().fold(Ok(()), |result, part| match part {
                Part::Name(path) => {
                    if path.contains(' ') {
                        result.and_then(|_| write!(f, ".'{}'", path))
                    } else {
                        result.and_then(|_| write!(f, ".{}", path))
                    }
                }
                Part::Index(path) => result.and_then(|_| write!(f, "[{}]", path)),
            })
        }
    }
}

#[cfg(test)]
pub mod test {
    use super::*;

    #[test]
    fn fmt_path() {
        let test0 = Path(vec![Part::Name("foo".to_string())]);
        assert_eq!(format!("{}", test0), ".foo".to_string());

        let test1 = Path(vec![
            Part::Name("foo".to_string()),
            Part::Name("bar".to_string()),
        ]);

        assert_eq!(format!("{}", test1), ".foo.bar".to_string());

        let test2 = Path(vec![
            Part::Name("foo".to_string()),
            Part::Name("bar".to_string()),
            Part::Index(1337),
        ]);

        assert_eq!(format!("{}", test2), ".foo.bar[1337]".to_string());

        let test3 = Path(vec![
            Part::Name("foo".to_string()),
            Part::Index(42),
            Part::Name("bar".to_string()),
            Part::Index(1337),
        ]);

        assert_eq!(format!("{}", test3), ".foo[42].bar[1337]".to_string());
    }

    #[test]
    fn fmt_path_quoted() {
        let test0 = Path(vec![
            Part::Name("f oo".to_string()),
            Part::Index(42),
            Part::Name("ba r".to_string()),
            Part::Index(1337),
        ]);

        assert_eq!(format!("{}", test0), ".'f oo'[42].'ba r'[1337]".to_string());
    }

    #[test]
    fn fmt_path_double_index() {
        // XXX is this even legal? If it was it's at least supposed to be `.[42][1337]`?,
        // XXX verify with Python side.
        let test0 = Path(vec![Part::Index(42), Part::Index(1337)]);

        assert_eq!(format!("{}", test0), "[42][1337]".to_string());

        let test1 = Path(vec![
            Part::Index(42),
            Part::Name("bar".to_string()),
            Part::Index(1337),
        ]);

        assert_eq!(format!("{}", test1), "[42].bar[1337]".to_string());
    }
}
