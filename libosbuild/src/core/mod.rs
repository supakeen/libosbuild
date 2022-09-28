use crate::manifest::description::validation;
use crate::manifest::path as manifest_path;

pub struct Schema {
    name: Option<String>,
    data: Option<String>,
}

impl Schema {
    pub fn new(name: Option<String>, data: Option<String>) -> Self {
        Self { name, data }
    }

    pub fn is_valid(self) -> bool {
        let mut result = validation::Result::new();

        if self.data.is_none() {
            result.add_error(validation::Error {
                message: "could not find schema information".to_string(),
                path: manifest_path::Path(vec![]),
            });
        }

        result.into()
    }
}

#[cfg(test)]
mod test;
