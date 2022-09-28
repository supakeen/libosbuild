use crate::core::Schema;
use crate::manifest::description::validation;
use crate::manifest::path;

#[test]
fn validation_result_no_error_valid() {
    let result = validation::Result::new();
    let valid: bool = result.into();

    assert_eq!(valid, true);
}

#[test]
fn validation_result_error_invalid() {
    let mut result = validation::Result::new();
    result.add_error(validation::Error {
        message: "booboo".to_string(),
        path: path::Path(vec![]),
    });
    let valid: bool = result.into();

    assert_eq!(valid, false);
}

#[test]
fn schema_without_data_is_invalid() {
    let schema = Schema::new(Some("name".to_string()), None);
    let valid = schema.is_valid();

    assert_eq!(valid, false);
}

#[test]
fn schema_with_data_is_valid() {
    let schema = Schema::new(Some("name".to_string()), Some("data".to_string()));
    let valid = schema.is_valid();

    assert_eq!(valid, true);
}
