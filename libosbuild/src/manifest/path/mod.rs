use std::fmt;
use std::ops;

pub mod test;

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
