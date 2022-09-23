/// Version 1 of manifest descriptions, this version is *DEPRECATED*.
pub mod v1 {
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
}

/// Version 2 of manifest descriptions, this version is current.
pub mod v2 {
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
}
