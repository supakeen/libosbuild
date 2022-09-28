use crate::manifest::path as manifest_path;

/// Describes a single failed validation. Consists of a `message` describing the error and a `path`
/// that points to the thing that caused the error.
pub struct ValidationError {
    message: String,
    path: manifest_path::Path,
}

impl ValidationError {
    /// Calculate the id of a ValidationError, this is a dotted and subscripted string that points
    /// to the element in the Manifest that triggered the error message.
    pub fn id(self) -> String {
        format!("{}", self.path)
    }
}

pub struct ValidationResult {
    origin: Option<String>,
    errors: Vec<ValidationError>,
}

impl ValidationResult {
    pub fn new(origin: String) -> Self {
        Self {
            origin: Some(origin),
            errors: vec![],
        }
    }

    /// Add a `ValidationError` to the set of errors
    pub fn add(&mut self, error: ValidationError) {
        self.errors.push(error);
    }

    /// Add a new `ValidationError` with `message` as message
    pub fn fail(&mut self, message: String) {
        self.add(ValidationError {
            message,
            path: manifest_path::Path(Vec::new()),
        });
    }

    /// Merge all errors of `result` into this `ValidationResult` adjusting their paths by
    /// pre-pending the optionally supplied `path`
    pub fn merge(&mut self, result: ValidationResult, path: Vec<manifest_path::Path>) {
        for error in result.errors {
            self.add(error);
        }
    }
}

impl From<ValidationResult> for bool {
    fn from(object: ValidationResult) -> bool {
        object.errors.is_empty()
    }
}

struct Schema {
    name: Option<String>,
    data: Option<String>,
}

impl Schema {
    // XXX ValidationError is a struct
    pub fn is_valid(self) -> bool {
        let mut result = ValidationResult::new(self.name.unwrap());

        if self.data.is_none() {
            result.fail("could not find schema information".to_string());
        }

        result.into()
    }

    pub fn validate(self, target: Schema) -> ValidationResult {
        ValidationResult::new(self.name.unwrap())
    }
}

struct Module {
    name: String,
    r#type: String,
    path: String,
}

struct Format {}

struct Index {
    path: String,
}

#[cfg(test)]
mod test;
