use phalcom_modules::DeclarationId;
use phalcom_modules::identity::ModuleId;
use phalcom_semantic::checker::inference::{ConstraintOrigin, InferenceOutcome, InferenceRelation, InferenceSession, InferenceTerm};
use phalcom_semantic::types::id::KindId;
use phalcom_semantic::types::relation::MapTypeHierarchy;
use phalcom_semantic::types::store::TypeStore;

fn test_decl(name: &str) -> DeclarationId {
    let module = ModuleId::core();
    DeclarationId::new(module, name.into())
}

#[test]
fn test_fresh_variable_does_not_grow_type_store() {
    let store = TypeStore::new();
    let initial_count = store.type_count();

    let mut session = InferenceSession::new();
    let v1 = session.fresh_variable(KindId::TYPE);
    let v2 = session.fresh_variable(KindId::TYPE);

    assert_ne!(v1, v2);
    // Law: solver variable is not in TypeStore
    assert_eq!(store.type_count(), initial_count);
}

#[test]
fn test_unification_and_occurs_check() {
    let mut store = TypeStore::new();
    let hier = MapTypeHierarchy::new();
    let int_decl = test_decl("Int");
    let list_decl = test_decl("List");
    let int_ty = store.nominal(int_decl);

    let list_kind = store.arrow_kind(vec![KindId::TYPE].into_boxed_slice(), KindId::TYPE);
    let list_form = store.nominal_form(list_decl, list_kind);
    let list_int_ty = store.list_of(list_form, int_ty).unwrap();

    let mut session = InferenceSession::new();
    let var = session.fresh_variable(KindId::TYPE);

    // ?T in List<?T> == List<Int>
    let list_var_term = InferenceTerm::Applied {
        origin: Box::new(InferenceTerm::Canonical(list_form)),
        arguments: Box::new([InferenceTerm::Var(var)]),
    };
    let list_int_term = InferenceTerm::Canonical(list_int_ty);

    session.add_constraint(InferenceRelation::Equivalent(list_var_term, list_int_term), ConstraintOrigin::Explicit, None);

    let outcome = session.solve(&mut store, &hier);
    assert!(outcome.is_solved());
    if let InferenceOutcome::Solved(sol) = outcome {
        assert_eq!(sol.substitutions.get(&var), Some(&int_ty));
    }

    // Materialize
    let materialized = session.materialize(&InferenceTerm::Var(var), &mut store).unwrap();
    assert_eq!(materialized, int_ty);
}

#[test]
fn test_occurs_check_rejects_recursive_term() {
    let mut store = TypeStore::new();
    let hier = MapTypeHierarchy::new();
    let list_decl = test_decl("List");
    let list_kind = store.arrow_kind(vec![KindId::TYPE].into_boxed_slice(), KindId::TYPE);
    let list_form = store.nominal_form(list_decl, list_kind);

    let mut session = InferenceSession::new();
    let var = session.fresh_variable(KindId::TYPE);

    // ?T == List<?T> -> occurs check failure
    let list_var_term = InferenceTerm::Applied {
        origin: Box::new(InferenceTerm::Canonical(list_form)),
        arguments: Box::new([InferenceTerm::Var(var)]),
    };

    session.add_constraint(
        InferenceRelation::Equivalent(InferenceTerm::Var(var), list_var_term),
        ConstraintOrigin::Explicit,
        None,
    );

    let outcome = session.solve(&mut store, &hier);
    assert!(!outcome.is_solved());
}

#[test]
fn test_underconstrained_variable_detected() {
    let mut store = TypeStore::new();
    let hier = MapTypeHierarchy::new();

    let mut session = InferenceSession::new();
    let _var = session.fresh_variable(KindId::TYPE);

    // No constraints added on var
    let outcome = session.solve(&mut store, &hier);
    assert!(matches!(outcome, InferenceOutcome::Underconstrained(_)));
}
