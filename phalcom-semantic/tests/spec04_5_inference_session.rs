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

#[test]
fn test_variable_to_variable_aliasing_solves() {
    let mut store = TypeStore::new();
    let hier = MapTypeHierarchy::new();
    let int_decl = test_decl("Int");
    let int_ty = store.nominal(int_decl);

    let mut session = InferenceSession::new();
    let v1 = session.fresh_variable(KindId::TYPE);
    let v2 = session.fresh_variable(KindId::TYPE);

    // ?v1 == ?v2
    session.add_constraint(
        InferenceRelation::Equivalent(InferenceTerm::Var(v1), InferenceTerm::Var(v2)),
        ConstraintOrigin::Explicit,
        None,
    );
    // ?v2 == Int
    session.add_constraint(
        InferenceRelation::Equivalent(InferenceTerm::Var(v2), InferenceTerm::Canonical(int_ty)),
        ConstraintOrigin::Explicit,
        None,
    );

    let outcome = session.solve(&mut store, &hier);
    assert!(outcome.is_solved());
    if let InferenceOutcome::Solved(sol) = outcome {
        assert_eq!(sol.substitutions.get(&v1), Some(&int_ty));
        assert_eq!(sol.substitutions.get(&v2), Some(&int_ty));
    }
}

#[test]
fn test_lower_bounds_join_infers_union() {
    let mut store = TypeStore::new();
    let hier = MapTypeHierarchy::new();
    let int_decl = test_decl("Int");
    let str_decl = test_decl("String");
    let int_ty = store.nominal(int_decl);
    let str_ty = store.nominal(str_decl);

    let mut session = InferenceSession::new();
    let v = session.fresh_variable(KindId::TYPE);

    // Int <: ?v
    session.add_constraint(
        InferenceRelation::Subtype(InferenceTerm::Canonical(int_ty), InferenceTerm::Var(v)),
        ConstraintOrigin::Explicit,
        None,
    );
    // String <: ?v
    session.add_constraint(
        InferenceRelation::Subtype(InferenceTerm::Canonical(str_ty), InferenceTerm::Var(v)),
        ConstraintOrigin::Explicit,
        None,
    );

    let outcome = session.solve(&mut store, &hier);
    assert!(outcome.is_solved());
    if let InferenceOutcome::Solved(sol) = outcome {
        let expected_union = store.union(&[int_ty, str_ty]);
        assert_eq!(sol.substitutions.get(&v), Some(&expected_union));
    }
}

#[test]
fn test_generic_signature_instantiation_and_callable_solving() {
    use phalcom_semantic::checker::inference::{InferenceCallable, InferenceCallableParameter};
    use phalcom_semantic::types::parameter::{GenericSignature, TypeParameterData, TypeParameterOwner};

    let mut store = TypeStore::new();
    let hier = MapTypeHierarchy::new();
    let int_decl = test_decl("Int");
    let int_ty = store.nominal(int_decl);

    let owner = TypeParameterOwner::Declaration(test_decl("Holder"));
    let param_data = TypeParameterData::new(owner.clone(), 0, "T", KindId::TYPE);
    let param_id = store.intern_type_parameter(param_data);
    let gen_sig = GenericSignature::new(owner, vec![param_id].into_boxed_slice());

    let mut session = InferenceSession::new();
    let param_map = session.instantiate_generic_signature(&gen_sig);
    let var_t = param_map.get(&param_id).unwrap().clone();

    // Callable: (t: ?T) -> ?T
    let callable_term = InferenceTerm::Callable(InferenceCallable {
        parameters: vec![InferenceCallableParameter {
            label: None,
            term: var_t.clone(),
            rest: false,
        }]
        .into_boxed_slice(),
        return_type: Box::new(var_t.clone()),
    });

    // Argument is Int => ?T == Int
    session.add_constraint(
        InferenceRelation::Subtype(InferenceTerm::Canonical(int_ty), var_t.clone()),
        ConstraintOrigin::Explicit,
        None,
    );

    let outcome = session.solve(&mut store, &hier);
    assert!(outcome.is_solved());

    let solved_callable = session.materialize(&callable_term, &mut store).unwrap();
    let expected_callable = store.callable(phalcom_semantic::types::store::CallableType {
        parameters: vec![phalcom_semantic::types::store::CallableParameterType {
            label: None,
            ty: int_ty,
            rest: false,
        }]
        .into_boxed_slice(),
        return_type: int_ty,
    });

    assert_eq!(solved_callable, expected_callable);
}

