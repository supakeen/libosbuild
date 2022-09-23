/// Manifests are described in JSON, this module provides functions and objects to parse those
/// JSON descriptions into manifests.
pub mod description;
pub mod meta;
pub mod validate;

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
