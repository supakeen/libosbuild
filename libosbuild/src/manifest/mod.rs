/// Manifests are described in JSON, this module provides functions and objects to parse those
/// JSON descriptions into manifests.
pub mod description;

#[derive(Debug)]
pub enum ManifestError {}

pub enum Version {
    V1,
    V2,
}

pub struct Manifest {
    version: Version,
}

pub struct Validator {
    manifest: Manifest,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn dummy() {
        assert_eq!(1, 1);
    }
}
