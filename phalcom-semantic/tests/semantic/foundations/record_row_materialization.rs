use phalcom_modules::identity::ModuleId;
use phalcom_semantic::identity::DeclarationId;
use phalcom_semantic::types::id::KindId;
use phalcom_semantic::types::instantiation::{GenericInstantiation, RowMaterializationMode, TypeMaterializationError, materialize_type};
use phalcom_semantic::types::parameter::{TypeParameterData, TypeParameterOwner};
use phalcom_semantic::types::row::{RecordRowField, RecordRowTail};
use phalcom_semantic::types::store::{RecordTypeField, TypeData, TypeStore};

fn owner(name: &str) -> DeclarationId {
    DeclarationId::new(ModuleId::universe_root(), name.into())
}

fn row_parameter(store: &mut TypeStore, name: &str, index: u32) -> phalcom_semantic::TypeParameterId {
    store.intern_type_parameter(TypeParameterData::new(
        TypeParameterOwner::Declaration(owner("Rows")),
        index,
        name,
        KindId::RECORD_ROW,
    ))
}

#[test]
fn materialization_substitutes_row_tail_into_canonical_record() {
    let mut store = TypeStore::new();
    let row_parameter = row_parameter(&mut store, "R", 0);
    let int = store.nominal(owner("Int"));
    let string = store.nominal(owner("String"));
    let remainder = store
        .record_row_checked(
            vec![RecordRowField {
                name: "name".into(),
                ty: string,
            }],
            RecordRowTail::Closed,
        )
        .unwrap();
    let open = store
        .record_row_type_checked(vec![RecordTypeField { name: "value".into(), ty: int }], RecordRowTail::Parameter(row_parameter))
        .unwrap();
    let mut instantiation = GenericInstantiation::default();
    instantiation.bind_row(row_parameter, remainder);

    let materialized = materialize_type(&mut store, open, &instantiation, RowMaterializationMode::RequireSolvedTail).unwrap();
    let TypeData::Record(row_id) = store.get(materialized) else {
        panic!("expected canonical Record")
    };
    let row = store.record_row(*row_id);
    assert_eq!(row.tail, RecordRowTail::Closed);
    assert_eq!(row.fields.iter().map(|field| field.name.as_ref()).collect::<Vec<_>>(), vec!["name", "value"]);
}

#[test]
fn materialization_preserves_or_rejects_unbound_stable_tail_by_mode() {
    let mut store = TypeStore::new();
    let row_parameter = row_parameter(&mut store, "R", 0);
    let open = store.record_row_type_checked(Vec::new(), RecordRowTail::Parameter(row_parameter)).unwrap();
    let instantiation = GenericInstantiation::default();

    let preserved = materialize_type(&mut store, open, &instantiation, RowMaterializationMode::PreserveUnboundStableTail).unwrap();
    assert_eq!(preserved, open);
    assert!(matches!(
        materialize_type(&mut store, open, &instantiation, RowMaterializationMode::RequireSolvedTail),
        Err(TypeMaterializationError::UnresolvedRowParameter(parameter)) if parameter == row_parameter
    ));
}

#[test]
fn materialization_rejects_duplicate_and_recursive_row_substitution() {
    let mut store = TypeStore::new();
    let row_parameter = row_parameter(&mut store, "R", 0);
    let int = store.nominal(owner("Int"));
    let string = store.nominal(owner("String"));
    let open = store
        .record_row_type_checked(vec![RecordTypeField { name: "value".into(), ty: int }], RecordRowTail::Parameter(row_parameter))
        .unwrap();
    let duplicate_remainder = store
        .record_row_checked(
            vec![RecordRowField {
                name: "value".into(),
                ty: string,
            }],
            RecordRowTail::Closed,
        )
        .unwrap();
    let mut duplicate = GenericInstantiation::default();
    duplicate.bind_row(row_parameter, duplicate_remainder);
    assert!(matches!(
        materialize_type(&mut store, open, &duplicate, RowMaterializationMode::RequireSolvedTail),
        Err(TypeMaterializationError::RecordRow(phalcom_semantic::types::row::RecordRowFormationError::DuplicateField(field))) if field.as_ref() == "value"
    ));

    let recursive_remainder = store.record_row_checked(Vec::new(), RecordRowTail::Parameter(row_parameter)).unwrap();
    let mut recursive = GenericInstantiation::default();
    recursive.bind_row(row_parameter, recursive_remainder);
    assert!(matches!(
        materialize_type(&mut store, open, &recursive, RowMaterializationMode::RequireSolvedTail),
        Err(TypeMaterializationError::RecursiveRowSubstitution(parameter)) if parameter == row_parameter
    ));
}
