use crate::manifest::*;

pub struct ManifestDescription {}

impl ManifestDescription {
    fn load(&self) {}
    fn load_assembler(&self) {}
    fn load_build(&self) {}
    fn load_pipeline(&self) {}
    fn load_source(&self) {}
    fn load_stage(&self) {}
}

fn describe(manifest: Manifest, with_id: bool) -> Result<Manifest, ManifestError> {
    Ok(Manifest {
        version: Version::V1,
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
