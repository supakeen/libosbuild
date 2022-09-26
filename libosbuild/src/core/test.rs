use crate::core::*;

#[test]
fn validation_error_path() {
    let test0 = ValidationError {
        message: String::new(),
        path: vec![ValidationPath::Name("foo".to_string())],
    };

    assert_eq!(test0.id(), ".foo".to_string());

    let test1 = ValidationError {
        message: String::new(),
        path: vec![
            ValidationPath::Name("foo".to_string()),
            ValidationPath::Name("bar".to_string()),
        ],
    };

    assert_eq!(test1.id(), ".foo.bar".to_string());

    let test2 = ValidationError {
        message: String::new(),
        path: vec![
            ValidationPath::Name("foo".to_string()),
            ValidationPath::Name("bar".to_string()),
            ValidationPath::Index(1337),
        ],
    };

    assert_eq!(test2.id(), ".foo.bar[1337]".to_string());

    let test3 = ValidationError {
        message: String::new(),
        path: vec![
            ValidationPath::Name("foo".to_string()),
            ValidationPath::Index(42),
            ValidationPath::Name("bar".to_string()),
            ValidationPath::Index(1337),
        ],
    };

    assert_eq!(test3.id(), ".foo[42].bar[1337]".to_string());
}

#[test]
fn validation_error_path_quoted() {
    let test3 = ValidationError {
        message: String::new(),
        path: vec![
            ValidationPath::Name("f oo".to_string()),
            ValidationPath::Index(42),
            ValidationPath::Name("ba r".to_string()),
            ValidationPath::Index(1337),
        ],
    };

    assert_eq!(test3.id(), ".'f oo'[42].'ba r'[1337]".to_string());
}

#[test]
fn validation_error_path_double_index() {
    // XXX is this even legal? If it was it's at least supposed to be `.[42][1337]`?,
    // XXX verify with Python side.
    let test0 = ValidationError {
        message: String::new(),
        path: vec![ValidationPath::Index(42), ValidationPath::Index(1337)],
    };

    assert_eq!(test0.id(), "[42][1337]".to_string());
}

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
