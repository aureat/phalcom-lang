use phalcom_modules::identity::ModuleId;
use phalcom_semantic::checker::{GenericApplicationSession, InferenceRecordTail, InferenceTerm};
use phalcom_semantic::identity::{AssociatedStaticDomainDeclarationId, DeclarationId};
use phalcom_semantic::types::id::{KindId, TypeId, TypeParameterId};
use phalcom_semantic::types::outcome::{CancellationToken, QueryBudget};
use phalcom_semantic::types::parameter::{GenericSignature, TypeParameterData, TypeParameterOwner};
use phalcom_semantic::types::row::{RecordRowField, RecordRowTail};
use phalcom_semantic::types::store::{RecordTypeField, StaticArgument, TypeData, TypeStore};

fn declaration(name: &str) -> DeclarationId {
    DeclarationId::new(ModuleId::universe_root(), name.into())
}

fn row_parameter(store: &mut TypeStore) -> TypeParameterId {
    store.intern_type_parameter(TypeParameterData::new(
        TypeParameterOwner::Declaration(declaration("Probe")),
        0,
        "R",
        KindId::RECORD_ROW,
    ))
}

fn prefixed_row(store: &mut TypeStore, parameter: TypeParameterId) -> phalcom_semantic::types::id::RecordRowId {
    let int = store.nominal(declaration("Int"));
    store
        .record_row_checked(vec![RecordRowField::required("known".into(), int)], RecordRowTail::Parameter(parameter))
        .expect("valid prefixed row")
}

fn assert_nested_lack_rejects_duplicate_extension(store: &mut TypeStore, parameter: TypeParameterId, nested: TypeId) {
    let signature = GenericSignature::new(TypeParameterOwner::Declaration(declaration("Probe")), Box::new([parameter]));
    let mut session = GenericApplicationSession::new(&signature, store);
    session.constrain_signature_type_lacks(nested, store).expect("nested stable lack");

    let formal = store
        .record_row_type_checked(Vec::new(), RecordRowTail::Parameter(parameter))
        .expect("formal open row");
    let int = store.nominal(declaration("Int"));
    let actual = store
        .record_row_type_checked(vec![RecordTypeField::required("known".into(), int)], RecordRowTail::Closed)
        .expect("actual row with duplicate field");
    let InferenceTerm::Record(formal_record) = session.type_term(formal, store) else {
        panic!("expected formal row inference term")
    };
    session
        .constrain_known_record_argument(actual, &formal_record, store)
        .expect("known record constraint");
    let mut budget = QueryBudget::default();
    let cancellation = CancellationToken::new();
    assert!(matches!(
        session.solve_rows(store, &mut budget, &cancellation),
        phalcom_semantic::types::row_solver::RecordRowSolveResult::Rejected(
            phalcom_semantic::types::row_solver::RecordRowFailure::LacksViolation { field, .. }
        ) if field.as_ref() == "known"
    ));
}

#[test]
fn row_and_type_inference_domains_allocate_distinct_variables() {
    let mut store = TypeStore::new();
    let owner = TypeParameterOwner::Declaration(declaration("Probe"));
    let type_parameter = store.intern_type_parameter(TypeParameterData::new(owner.clone(), 0, "T", KindId::TYPE));
    let row_parameter = store.intern_type_parameter(TypeParameterData::new(owner, 1, "R", KindId::RECORD_ROW));
    let signature = GenericSignature::new(TypeParameterOwner::Declaration(declaration("Probe")), Box::new([type_parameter, row_parameter]));
    let session = GenericApplicationSession::new(&signature, &store);
    let type_binding = session.parameter_bindings.get(&type_parameter).expect("type binding");
    let row_binding = session.parameter_bindings.get(&row_parameter).expect("row binding");
    assert!(matches!(type_binding, phalcom_semantic::checker::GenericInferenceBinding::Type(_)));
    assert!(matches!(row_binding, phalcom_semantic::checker::GenericInferenceBinding::RecordRow(_)));
    assert_eq!(session.type_terms().len(), 1);
    assert_eq!(session.row_terms().len(), 1);
}

#[test]
fn record_argument_decomposition_solves_remainder_without_publishing_row_variable() {
    let mut store = TypeStore::new();
    let owner = TypeParameterOwner::Declaration(declaration("Probe"));
    let row_parameter = store.intern_type_parameter(TypeParameterData::new(owner.clone(), 0, "R", KindId::RECORD_ROW));
    let signature = GenericSignature::new(owner, Box::new([row_parameter]));
    let mut session = GenericApplicationSession::new(&signature, &store);
    let int = store.nominal(declaration("Int"));
    let string = store.nominal(declaration("String"));
    let formal = store
        .record_row_type_checked(vec![RecordTypeField::required("item".into(), int)], RecordRowTail::Parameter(row_parameter))
        .unwrap();
    session.constrain_signature_type_lacks(formal, &store).unwrap();
    let actual = store
        .record_row_type_checked(
            vec![RecordTypeField::required("item".into(), int), RecordTypeField::required("name".into(), string)],
            RecordRowTail::Closed,
        )
        .unwrap();
    let formal_term = session.type_term(formal, &store);
    let InferenceTerm::Record(formal_record) = formal_term else {
        panic!("expected record inference term")
    };
    let ordinary = session
        .constrain_known_record_argument(actual, &formal_record, &store)
        .unwrap()
        .expect("record argument");
    assert_eq!(ordinary.len(), 1);
    assert!(matches!(ordinary[0], (InferenceTerm::Canonical(actual), InferenceTerm::Canonical(expected)) if actual == int && expected == int));
    let mut budget = QueryBudget::default();
    let cancellation = CancellationToken::new();
    let solution = session.solve_rows(&store, &mut budget, &cancellation);
    let phalcom_semantic::types::row_solver::RecordRowSolveResult::Solved(solution) = solution else {
        panic!("expected solved remainder")
    };
    let variable = match session.row_terms()[&row_parameter] {
        phalcom_semantic::types::row_solver::RecordRowVarId(value) => phalcom_semantic::types::row_solver::RecordRowVarId(value),
    };
    let remainder = solution.zonk_variable_to_canonical(variable, &mut store).unwrap();
    let data = store.record_row(remainder);
    assert_eq!(data.fields.as_ref(), &[RecordRowField::required("name".into(), string)]);
    assert_eq!(data.tail, RecordRowTail::Closed);
    assert!(!store.contains_parameter_type(row_parameter));
    assert!(!matches!(store.get(formal), TypeData::Parameter(_)));
    assert!(!matches!(formal_record.tail, InferenceRecordTail::Parameter(_)));
}

#[test]
fn return_record_prefix_contributes_signature_lacks_to_row_inference() {
    let mut store = TypeStore::new();
    let owner = TypeParameterOwner::Declaration(declaration("Probe"));
    let row_parameter = store.intern_type_parameter(TypeParameterData::new(owner.clone(), 0, "R", KindId::RECORD_ROW));
    let signature = GenericSignature::new(owner, Box::new([row_parameter]));
    let mut session = GenericApplicationSession::new(&signature, &store);
    let string = store.nominal(declaration("String"));
    let formal = store
        .record_row_type_checked(vec![RecordTypeField::required("name".into(), string)], RecordRowTail::Parameter(row_parameter))
        .unwrap();
    let returned = store
        .record_row_type_checked(
            vec![
                RecordTypeField::required("name".into(), string),
                RecordTypeField::required("tag".into(), string),
            ],
            RecordRowTail::Parameter(row_parameter),
        )
        .unwrap();
    session.constrain_signature_type_lacks(formal, &store).unwrap();
    session.constrain_signature_type_lacks(returned, &store).unwrap();

    let actual = store
        .record_row_type_checked(
            vec![
                RecordTypeField::required("name".into(), string),
                RecordTypeField::required("tag".into(), string),
            ],
            RecordRowTail::Closed,
        )
        .unwrap();
    let InferenceTerm::Record(formal_record) = session.type_term(formal, &store) else {
        panic!("expected record inference term");
    };
    session
        .constrain_known_record_argument(actual, &formal_record, &store)
        .unwrap()
        .expect("record argument");

    let mut budget = QueryBudget::default();
    let cancellation = CancellationToken::new();
    let outcome = session.solve_rows(&store, &mut budget, &cancellation);
    assert!(matches!(
        outcome,
        phalcom_semantic::types::row_solver::RecordRowSolveResult::Rejected(
            phalcom_semantic::types::row_solver::RecordRowFailure::LacksViolation { field, .. }
        ) if field.as_ref() == "tag"
    ));
}

#[test]
fn heterogeneous_applied_row_argument_preserves_nested_stable_lacks() {
    let mut store = TypeStore::new();
    let parameter = row_parameter(&mut store);
    let row = prefixed_row(&mut store, parameter);
    let kind = store.arrow_kind(vec![KindId::RECORD_ROW].into_boxed_slice(), KindId::TYPE);
    let origin = store.nominal_form(declaration("Applied"), kind);
    let applied = store
        .apply_static_form(origin, &[StaticArgument::RecordRow(row)])
        .expect("heterogeneous row application");

    assert_nested_lack_rejects_duplicate_extension(&mut store, parameter, applied);
}

#[test]
fn static_data_row_argument_preserves_nested_stable_lacks() {
    let mut store = TypeStore::new();
    let parameter = row_parameter(&mut store);
    let row = prefixed_row(&mut store, parameter);
    let declaration = declaration("Shape");
    let domain = store.nominal_static_domain(declaration.clone());
    let constructor = store.intern_static_constructor(domain, declaration, "Row", vec![KindId::RECORD_ROW].into_boxed_slice());
    let static_data = store
        .static_data(domain, constructor, vec![StaticArgument::RecordRow(row)].into_boxed_slice())
        .expect("row-valued static constructor argument");

    assert_nested_lack_rejects_duplicate_extension(&mut store, parameter, static_data);
}

#[test]
fn associated_static_domain_owner_row_argument_preserves_nested_stable_lacks() {
    let mut store = TypeStore::new();
    let parameter = row_parameter(&mut store);
    let row = prefixed_row(&mut store, parameter);
    let owner = declaration("Container");
    let associated = AssociatedStaticDomainDeclarationId::new(owner.clone(), "State");
    let domain = store.associated_static_domain(associated, vec![StaticArgument::RecordRow(row)].into_boxed_slice());
    let constructor = store.intern_static_constructor(domain, owner, "Ready", Box::new([]));
    let static_data = store.static_data(domain, constructor, Box::new([])).expect("associated static case");

    assert_nested_lack_rejects_duplicate_extension(&mut store, parameter, static_data);
}
