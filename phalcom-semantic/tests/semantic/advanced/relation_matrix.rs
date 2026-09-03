use phalcom_common::selector::Selector;
use phalcom_modules::identity::ModuleId;
use phalcom_semantic::identity::{DeclarationId, DispatchSide, VariantId};
use phalcom_semantic::types::family::{FamilyMemberType, FamilyOperationShape};
use phalcom_semantic::types::id::KindId;
use phalcom_semantic::types::parameter::{SelfRole, SelfTypeTerm, TypeParameterData, TypeParameterOwner};
use phalcom_semantic::types::relation::{MapTypeHierarchy, is_subtype};
use phalcom_semantic::types::row::{RecordRowField, RecordRowTail};
use phalcom_semantic::types::store::{CallableParameterType, CallableType, TupleTypeElement, TypeData, TypeStore};
use phalcom_semantic::types::type_lambda::ScopedTypeData;

fn decl(name: &str) -> DeclarationId {
    DeclarationId::new(ModuleId::universe_root(), name.into())
}

#[test]
fn every_proper_matrix_form_has_explicit_relation_behavior() {
    let mut store = TypeStore::new();
    let mut hierarchy = MapTypeHierarchy::new();
    let int = store.nominal(decl("Int"));
    let string = store.nominal(decl("String"));
    let object = store.nominal(decl("Object"));
    hierarchy.insert(decl("Int"), decl("Object"));
    let class_object = store.class_object_type(decl("Int"));
    let tuple = store.tuple(vec![TupleTypeElement { label: None, ty: int }, TupleTypeElement { label: None, ty: string }].into_boxed_slice());
    let record = store.record(Box::new([RecordRowField { name: "value".into(), ty: int }]));
    let callable = store.callable(CallableType {
        parameters: Box::new([CallableParameterType::new(int)]),
        return_type: string,
    });
    let family = store
        .family_type([FamilyMemberType::value(FamilyOperationShape::getter(), int)])
        .expect("valid family shape");
    let parameter = store.intern_type_parameter(TypeParameterData::new(TypeParameterOwner::Declaration(decl("Owner")), 0, "T", KindId::TYPE));
    let parameter_form = store.parameter_form(parameter);
    let self_form = store.self_type(SelfTypeTerm {
        owner: decl("Owner"),
        side: DispatchSide::Instance,
        role: SelfRole::InstanceType,
    });
    let union = store.union(&[int, string]);
    let never = store.never();
    let unit = store.unit();

    assert!(is_subtype(&mut store, &hierarchy, never, int));
    assert!(is_subtype(&mut store, &hierarchy, unit, unit));
    assert!(is_subtype(&mut store, &hierarchy, int, object));
    assert!(!is_subtype(&mut store, &hierarchy, class_object, int));
    assert!(is_subtype(&mut store, &hierarchy, tuple, tuple));
    assert!(is_subtype(&mut store, &hierarchy, record, record));
    assert!(is_subtype(&mut store, &hierarchy, callable, callable));
    assert!(is_subtype(&mut store, &hierarchy, family, family));
    assert!(is_subtype(&mut store, &hierarchy, parameter_form, parameter_form));
    assert!(is_subtype(&mut store, &hierarchy, self_form, self_form));
    assert!(is_subtype(&mut store, &hierarchy, int, union));
    assert!(!is_subtype(&mut store, &hierarchy, string, int));

    assert!(matches!(store.get(tuple), TypeData::Tuple(_)));
    assert!(matches!(store.get(record), TypeData::Record(_)));
    assert!(matches!(store.get(callable), TypeData::Callable(_)));
    assert!(matches!(store.get(family), TypeData::Family(_)));
    assert!(matches!(store.get(parameter_form), TypeData::Parameter(_)));
    assert!(matches!(store.get(self_form), TypeData::SelfType(_)));
}

#[test]
fn constructors_and_lambdas_are_not_runtime_value_subtypes() {
    let mut store = TypeStore::new();
    let hierarchy = MapTypeHierarchy::new();
    let object = store.nominal(decl("Object"));
    let constructor_kind = store.arrow_kind(Box::new([KindId::TYPE]), KindId::TYPE);
    let constructor = store.nominal_form(decl("Box"), constructor_kind);
    let unit = store.unit();
    let body = store.arena_mut().intern_scoped(ScopedTypeData::Free(unit));
    let lambda_id = store.arena_mut().intern_lambda(Box::new([KindId::TYPE]), body, KindId::TYPE, None);
    let lambda = store.type_lambda(lambda_id);
    let never = store.never();

    assert_ne!(store.kind_of(constructor), KindId::TYPE);
    assert_ne!(store.kind_of(lambda), KindId::TYPE);
    assert!(!is_subtype(&mut store, &hierarchy, constructor, object));
    assert!(!is_subtype(&mut store, &hierarchy, lambda, object));
    assert!(!is_subtype(&mut store, &hierarchy, never, lambda));
}

#[test]
fn exact_cases_and_open_records_follow_canonical_relation() {
    let mut store = TypeStore::new();
    let hierarchy = MapTypeHierarchy::new();
    let color = decl("Color");
    let color_type = store.nominal(color.clone());
    let red = VariantId::new(color, Selector::getter("Red").expect("valid getter selector"));
    let red_case = store.exact_case_type(&red, color_type).expect("valid exact case");
    assert!(is_subtype(&mut store, &hierarchy, red_case, color_type));

    let owner = TypeParameterOwner::Declaration(decl("RowOwner"));
    let row_parameter = store.intern_type_parameter(TypeParameterData::new(owner, 0, "R", KindId::RECORD_ROW));
    let int = store.nominal(decl("Int"));
    let open = store
        .record_row_type_checked(vec![RecordRowField { name: "value".into(), ty: int }], RecordRowTail::Parameter(row_parameter))
        .expect("valid open row");
    let required = store.record(Box::new([RecordRowField { name: "value".into(), ty: int }]));

    assert!(is_subtype(&mut store, &hierarchy, open, required));
    assert!(!is_subtype(&mut store, &hierarchy, required, open));
    let missing = store.record(Box::new([RecordRowField { name: "other".into(), ty: int }]));
    assert!(!is_subtype(&mut store, &hierarchy, missing, required));
}

#[test]
fn callable_relation_preserves_parameter_shape_and_polarity() {
    let mut store = TypeStore::new();
    let hierarchy = MapTypeHierarchy::new();
    let int = store.nominal(decl("Int"));
    let string = store.nominal(decl("String"));
    let int_to_string = store.callable(CallableType {
        parameters: Box::new([CallableParameterType::new(int)]),
        return_type: string,
    });
    let string_to_string = store.callable(CallableType {
        parameters: Box::new([CallableParameterType::new(string)]),
        return_type: string,
    });
    let labeled = store.callable(CallableType {
        parameters: Box::new([CallableParameterType::new(int).with_label("value")]),
        return_type: string,
    });

    assert!(is_subtype(&mut store, &hierarchy, int_to_string, int_to_string));
    assert!(!is_subtype(&mut store, &hierarchy, int_to_string, string_to_string));
    assert!(!is_subtype(&mut store, &hierarchy, int_to_string, labeled));
}
