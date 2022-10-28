pub mod description;
pub mod path;

#[derive(Debug)]
pub enum ManifestError {
}

pub enum Version {
    V1,
    V2,
}

pub struct Manifest {}

#[cfg(test)]
mod test {
    #[test]
    fn dummy() {
        assert_eq!(1, 1);
    }
}
