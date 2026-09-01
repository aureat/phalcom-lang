use phalcom_modules::identity::ModuleId;
use phalcom_semantic::identity::DeclarationId;
use phalcom_semantic::types::application::TypeApplicationError;
use phalcom_semantic::types::id::KindId;
use phalcom_semantic::types::kind::KindData;
use phalcom_semantic::types::store::TypeStore;

fn test_decl(name: &str) -> DeclarationId {
    DeclarationId::new(ModuleId::universe_root(), name.into())
}

#[test]
fn kind_application_returns_residual_arrow_kind() {
    let mut store = TypeStore::new();
    let map_kind = store.arrow_kind(vec![KindId::TYPE, KindId::TYPE].into_boxed_slice(), KindId::TYPE);

    let residual = store.apply_kind(map_kind, &[KindId::TYPE]).unwrap();

    assert_eq!(
        store.get_kind(residual),
        &KindData::Arrow {
            parameters: vec![KindId::TYPE].into_boxed_slice(),
            result: KindId::TYPE,
        }
    );
}

#[test]
fn nested_type_application_canonicalizes() {
    let mut store = TypeStore::new();
    let map_decl = test_decl("Map");
    let string_decl = test_decl("String");
    let int_decl = test_decl("Int");

    let map_kind = store.arrow_kind(vec![KindId::TYPE, KindId::TYPE].into_boxed_slice(), KindId::TYPE);
    let map = store.nominal_form(map_decl, map_kind);
    let string = store.nominal_type(string_decl);
    let int = store.nominal_type(int_decl);

    let partially = store.apply_type_form(map, &[string]).unwrap();
    let nested = store.apply_type_form(partially, &[int]).unwrap();
    let direct = store.apply_type_form(map, &[string, int]).unwrap();

    assert_eq!(nested, direct);
    assert_eq!(store.kind_of(direct), KindId::TYPE);
}

#[test]
fn wrong_kind_parameter_fails() {
    let mut store = TypeStore::new();
    let list_decl = test_decl("List");
    let map_decl = test_decl("Map");

    let list_kind = store.arrow_kind(vec![KindId::TYPE].into_boxed_slice(), KindId::TYPE);
    let map_kind = store.arrow_kind(vec![KindId::TYPE, KindId::TYPE].into_boxed_slice(), KindId::TYPE);

    let list = store.nominal_form(list_decl, list_kind);
    let map = store.nominal_form(map_decl, map_kind);

    let err = store.apply_type_form(list, &[map]).unwrap_err();
    assert!(matches!(
        err,
        TypeApplicationError::ArgumentKindMismatch {
            index: 0,
            expected: KindId::TYPE,
            actual,
        } if actual == map_kind
    ));
}

#[test]
fn too_many_arguments_fails() {
    let mut store = TypeStore::new();
    let list_decl = test_decl("List");
    let int_decl = test_decl("Int");

    let list_kind = store.arrow_kind(vec![KindId::TYPE].into_boxed_slice(), KindId::TYPE);
    let list = store.nominal_form(list_decl, list_kind);
    let int = store.nominal_type(int_decl);

    let err = store.apply_type_form(list, &[int, int]).unwrap_err();
    assert_eq!(err, TypeApplicationError::TooManyArguments { supplied: 2, accepted: 1 });
}

#[test]
fn applying_proper_int_as_constructor_fails() {
    let mut store = TypeStore::new();
    let int_decl = test_decl("Int");
    let int = store.nominal_type(int_decl);

    let err = store.apply_type_form(int, &[int]).unwrap_err();
    assert_eq!(
        err,
        TypeApplicationError::NotAConstructor {
            origin: int,
            kind: KindId::TYPE,
        }
    );
}

#[test]
fn class_object_distinct_from_nominal() {
    let mut store = TypeStore::new();
    let int_decl = test_decl("Int");

    let nominal_int = store.nominal_type(int_decl.clone());
    let class_obj_int = store.class_object_type(int_decl);

    assert_ne!(nominal_int, class_obj_int);
    assert_eq!(store.kind_of(nominal_int), KindId::TYPE);
    assert_eq!(store.kind_of(class_obj_int), KindId::TYPE);
    assert!(store.is_proper_type(nominal_int));
    assert!(store.is_proper_type(class_obj_int));
}

#[test]
fn every_interned_type_has_explicit_kind() {
    let store = TypeStore::new();
    assert_eq!(store.kind_of(store.never()), KindId::TYPE);
    assert_eq!(store.kind_of(store.unit()), KindId::TYPE);
    assert!(store.is_proper_type(store.never()));
    assert!(store.is_proper_type(store.unit()));
}
