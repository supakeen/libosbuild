/// Version 1 of the manifest description This is the first version of the osbuild manifest description,
/// that has a main pipeline that consists of zero or more stages to create a tree and optionally one assembler that assembles
/// the created tree into an artefact. The pipeline can have any number of nested build pipelines. A sources section is used
/// to fetch resources.
pub mod v1;

/// Version 2 of manifest descriptions, this version is current.
pub mod v2;

#[derive(Debug)]
pub enum ManifestDescriptionError {}
