use crate::manifest::path::*;

#[test]
fn fmt_path() {
    let test0 = Path(vec![Part::Name("foo".to_string())]);
    assert_eq!(format!("{}", test0), ".foo".to_string());

    let test1 = Path(vec![
        Part::Name("foo".to_string()),
        Part::Name("bar".to_string()),
    ]);

    assert_eq!(format!("{}", test1), ".foo.bar".to_string());

    let test2 = Path(vec![
        Part::Name("foo".to_string()),
        Part::Name("bar".to_string()),
        Part::Index(1337),
    ]);

    assert_eq!(format!("{}", test2), ".foo.bar[1337]".to_string());

    let test3 = Path(vec![
        Part::Name("foo".to_string()),
        Part::Index(42),
        Part::Name("bar".to_string()),
        Part::Index(1337),
    ]);

    assert_eq!(format!("{}", test3), ".foo[42].bar[1337]".to_string());
}

#[test]
fn fmt_path_quoted() {
    let test0 = Path(vec![
        Part::Name("f oo".to_string()),
        Part::Index(42),
        Part::Name("ba r".to_string()),
        Part::Index(1337),
    ]);

    assert_eq!(format!("{}", test0), ".'f oo'[42].'ba r'[1337]".to_string());
}

#[test]
fn fmt_path_double_index() {
    // XXX is this even legal? If it was it's at least supposed to be `.[42][1337]`?,
    let test0 = Path(vec![Part::Index(42), Part::Index(1337)]);

    assert_eq!(format!("{}", test0), "[42][1337]".to_string());

    let test1 = Path(vec![
        Part::Index(42),
        Part::Name("bar".to_string()),
        Part::Index(1337),
    ]);

    assert_eq!(format!("{}", test1), "[42].bar[1337]".to_string());

    let test2 = Path(vec![
        Part::Name("foo".to_string()),
        Part::Index(42),
        Part::Name("bar".to_string()),
        Part::Index(1337),
    ]);

    assert_eq!(format!("{}", test2), ".foo[42].bar[1337]".to_string());
}
