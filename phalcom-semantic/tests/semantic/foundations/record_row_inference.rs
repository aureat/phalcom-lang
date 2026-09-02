use phalcom_modules::identity::ModuleId;
use phalcom_semantic::checker::{GenericApplicationSession, InferenceRecordTail, InferenceTerm};
use phalcom_semantic::identity::DeclarationId;
use phalcom_semantic::types::id::KindId;
use phalcom_semantic::types::outcome::{CancellationToken, QueryBudget};
use phalcom_semantic::types::parameter::{GenericSignature, TypeParameterData, TypeParameterOwner};
use phalcom_semantic::types::row::{RecordRowField, RecordRowTail};
use phalcom_semantic::types::store::{RecordTypeField, TypeData, TypeStore};

fn declaration(name: &str) -> DeclarationId {
    DeclarationId::new(ModuleId::universe_root(), name.into())
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
        .record_row_type_checked(vec![RecordTypeField { name: "item".into(), ty: int }], RecordRowTail::Parameter(row_parameter))
        .unwrap();
    let actual = store
        .record_row_type_checked(
            vec![
                RecordTypeField { name: "item".into(), ty: int },
                RecordTypeField {
                    name: "name".into(),
                    ty: string,
                },
            ],
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
    assert_eq!(
        data.fields.as_ref(),
        &[RecordRowField {
            name: "name".into(),
            ty: string,
        }]
    );
    assert_eq!(data.tail, RecordRowTail::Closed);
    assert!(!store.contains_parameter_type(row_parameter));
    assert!(!matches!(store.get(formal), TypeData::Parameter(_)));
    assert!(!matches!(formal_record.tail, InferenceRecordTail::Parameter(_)));
}
