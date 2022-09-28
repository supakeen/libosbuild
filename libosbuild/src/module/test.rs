use crate::module::*;

#[test]
fn registry_by_name() {
    let module = Module::new(Kind::Stage, "/bin/sh").unwrap();

    let registry = Registry {
        modules: vec![module],
    };

    let option = registry.by_name("sh");

    assert!(option.is_some());
}

#[test]
fn registry_by_name_no_result() {
    let module = Module::new(Kind::Stage, "/bin/sh").unwrap();
    let registry = Registry::new(vec![module]);

    let option = registry.by_name("foo");

    assert!(option.is_none());
}

#[test]
fn registry_by_kind() {
    let module = Module::new(Kind::Stage, "/bin/sh").unwrap();
    let registry = Registry::new(vec![module]);

    let option = registry.by_kind(Kind::Stage);

    assert!(option.is_some());
}

#[test]
fn registry_by_kind_no_result() {
    let module = Module::new(Kind::Stage, "/bin/sh").unwrap();

    let registry = Registry {
        modules: vec![module],
    };

    let option = registry.by_kind(Kind::Runner);

    assert!(option.is_none());
}

#[test]
fn registry_by_kind_multiple_result() {
    let module0 = Module::new(Kind::Stage, "/bin/sh").unwrap();
    let module1 = Module::new(Kind::Stage, "/bin/sh").unwrap();
    let registry = Registry::new(vec![module0, module1]);

    let option = registry.by_kind(Kind::Stage);

    assert!(option.is_some());
    assert_eq!(option.unwrap().len(), 2);
}

#[test]
fn module_get_schema() {
    let module = Module::new(Kind::Stage, "/usr/bin/ls").unwrap();

    let mut schema = module.get_schema();
    assert!(schema.is_ok());

    schema = module.get_schema();
    assert!(schema.is_ok());
}

#[test]
fn module_get_schema_unparseable_path() {
    assert!(Module::new(Kind::Stage, "").is_err());
}
