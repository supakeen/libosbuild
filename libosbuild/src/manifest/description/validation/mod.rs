use crate::manifest::path as manifest_path;

#[cfg(test)]
pub mod test;

/// Describes a single failed validation. Consists of a `message` describing the error and a `path`
/// that points to the thing that caused the error.
pub struct Error {
    pub message: String,
    pub path: manifest_path::Path,
}

impl Error {
    /// Calculate the id of a Error, this is a dotted and subscripted string that points
    /// to the element in the Manifest that triggered the error message.
    pub fn id(self) -> String {
        format!("{}", self.path)
    }
}

pub struct Result {
    errors: Vec<Error>,
}

impl Result {
    pub fn new() -> Self {
        Self { errors: vec![] }
    }

    /// Add a `Error` to the set of errors
    pub fn add_error(&mut self, error: Error) {
        self.errors.push(error);
    }
}

impl From<Result> for bool {
    fn from(object: Result) -> bool {
        object.errors.is_empty()
    }
}
