use crate::manifest::*;

pub struct ManifestDescription {}

impl ManifestDescription {
    fn load(&self) {}
    fn load_device(&self) {}
    fn load_input(&self) {}
    fn load_mount(&self) {}
    fn load_pipeline(&self) {}
    fn load_stage(&self) {}
}

pub struct DeviceDescription {}

pub struct InputDescription {}

pub struct MountDescription {}

pub struct StageDescription {}

pub struct PipelineDescription {}

fn describe(manifest: Manifest, with_id: bool) -> Result<Manifest, ManifestError> {
    Ok(Manifest {
        version: Version::V2,
    })
}

pub struct Validator {
    manifest: Manifest,
}

impl Validator {
    fn validate_module(&self) {}
    fn validate_pipeline(&self) {}
    fn validate_stage(&self) {}
    fn validate_stage_modules(&self) {}
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn dummy() {
        assert_eq!(1, 1);
    }
}
