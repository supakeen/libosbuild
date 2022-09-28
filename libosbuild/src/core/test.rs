use crate::core::*;

#[test]
fn validation_result_no_error_valid() {
    let result = ValidationResult::new(String::new());
    let valid: bool = result.into();

    assert_eq!(valid, true);
}

#[test]
fn validation_result_error_invalid() {
    let mut result = ValidationResult::new(String::new());
    result.fail("booboo".to_string());
    let valid: bool = result.into();

    assert_eq!(valid, false);
}

#[test]
fn schema_without_data_is_invalid() {
    let schema = Schema {
        name: Some("name".to_string()),
        data: None,
    };
    let valid = schema.is_valid();

    assert_eq!(valid, false);
}

#[test]
fn schema_with_data_is_valid() {
    let schema = Schema {
        name: Some("name".to_string()),
        data: Some("data".to_string()),
    };
    let valid = schema.is_valid();

    assert_eq!(valid, true);
}
