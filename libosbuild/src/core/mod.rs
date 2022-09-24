pub enum ValidationPath {
    Name(String),
    Index(usize),
}

/// Describes a single failed validation. Consists of a `message` describing the error and a `path`
/// that points to the thing that caused the error.
pub struct ValidationError {
    message: String,
    path: Vec<ValidationPath>,
}

impl ValidationError {
    pub fn id(self) -> String {
        if self.path.is_empty() {
            ".".to_string()
        } else {
            let mut result = String::new();

            for part in self.path.into_iter() {
                match part {
                    ValidationPath::Name(path) => {
                        if path.contains(' ') {
                            result = format!("{}.'{}'", result, path);
                        } else {
                            result = format!("{}.{}", result, path);
                        }
                    }
                    ValidationPath::Index(path) => {
                        result = format!("{}[{}]", result, path);
                    }
                }
            }

            result
        }
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
            path: Vec::new(),
        });
    }

    /// Merge all errors of `result` into this `ValidationResult` adjusting their paths by
    /// pre-pending the optionally supplied `path`
    pub fn merge(&mut self, result: ValidationResult, path: Vec<ValidationPath>) {
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
