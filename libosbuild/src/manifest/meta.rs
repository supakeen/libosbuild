use crate::manifest::validate::{ValidationError, ValidationResult};

struct Schema {
    name: Option<String>,
    data: Option<String>,
}

impl Schema {
    // XXX ValidationError is a struct
    pub fn check(self) -> ValidationResult {
        let mut result = ValidationResult::new(self.name.unwrap());

        if self.data.is_none() {
            result.fail("could not find schema information".to_string());
        }

        result
    }

    pub fn validate(self, target: Schema) -> ValidationResult {
        ValidationResult::new(self.name.unwrap())
    }
}
