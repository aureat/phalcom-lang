use phalcom_modules::DeclarationId;
use phalcom_modules::identity::ModuleId;
use phalcom_semantic::checker::context::CheckerControl;
use phalcom_semantic::checker::inference::{
    ConstraintOrigin, InferenceFailureReason, InferenceOutcome, InferenceProofState, InferenceRelation, InferenceSession, InferenceSupport, InferenceTerm,
    InferenceTupleElement,
};
use phalcom_semantic::db::{CancellationToken, QueryBudget};
use phalcom_semantic::types::evidence::{DynamicReason, EvidenceOrigin, TypeKnowledge, UnknownReason};
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
    let param_map = session.instantiate_generic_signature(&gen_sig, &store);
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

#[test]
fn conflicting_constraint_retains_real_origin_and_terms() {
    let mut store = TypeStore::new();
    let hier = MapTypeHierarchy::new();
    let int_ty = store.nominal(test_decl("Int"));
    let string_ty = store.nominal(test_decl("String"));
    let mut session = InferenceSession::new();
    let var = session.fresh_variable(KindId::TYPE);

    session.add_constraint(
        InferenceRelation::Equivalent(InferenceTerm::Var(var), InferenceTerm::Canonical(int_ty)),
        ConstraintOrigin::Explicit,
        None,
    );
    session.add_constraint(
        InferenceRelation::Equivalent(InferenceTerm::Var(var), InferenceTerm::Canonical(string_ty)),
        ConstraintOrigin::Explicit,
        None,
    );

    let outcome = session.solve(&mut store, &hier);
    let InferenceOutcome::Conflicting(conflict) = outcome else {
        panic!("expected a real conflicting constraint");
    };
    assert_eq!(conflict.constraint_index, Some(1));
    assert_eq!(conflict.origin, Some(ConstraintOrigin::Explicit));
    assert!(matches!(conflict.failure, InferenceFailureReason::StructuralMismatch { .. }));
}

#[test]
fn conflicting_bounds_retain_failed_upper_bound_origin() {
    let mut store = TypeStore::new();
    let mut hierarchy = MapTypeHierarchy::new();
    let int_decl = test_decl("Int");
    let number_decl = test_decl("Number");
    let string_decl = test_decl("String");
    hierarchy.insert(int_decl.clone(), number_decl.clone());
    let int_ty = store.nominal(int_decl);
    let number_ty = store.nominal(number_decl);
    let string_ty = store.nominal(string_decl);
    let mut session = InferenceSession::new();
    let variable = session.fresh_variable(KindId::TYPE);
    let first_expression =
        phalcom_semantic::identity::ExpressionId::new(phalcom_semantic::identity::BodyId(1), phalcom_semantic::identity::LocalExpressionId(1));
    let failed_expression =
        phalcom_semantic::identity::ExpressionId::new(phalcom_semantic::identity::BodyId(1), phalcom_semantic::identity::LocalExpressionId(2));

    session.add_constraint(
        InferenceRelation::Subtype(InferenceTerm::Canonical(int_ty), InferenceTerm::Var(variable)),
        ConstraintOrigin::Explicit,
        None,
    );
    session.add_constraint(
        InferenceRelation::Subtype(InferenceTerm::Var(variable), InferenceTerm::Canonical(number_ty)),
        ConstraintOrigin::ExpectedResult { expression: first_expression },
        None,
    );
    session.add_constraint(
        InferenceRelation::Subtype(InferenceTerm::Var(variable), InferenceTerm::Canonical(string_ty)),
        ConstraintOrigin::ExpectedResult { expression: failed_expression },
        None,
    );

    let InferenceOutcome::Conflicting(conflict) = session.solve(&mut store, &hierarchy) else {
        panic!("expected final conflicting-bound reconciliation");
    };
    assert_eq!(conflict.constraint_index, Some(2));
    assert_eq!(conflict.origin, Some(ConstraintOrigin::ExpectedResult { expression: failed_expression }));
}

#[test]
fn generic_support_tracks_value_evidence_and_ignores_plain_context() {
    let mut store = TypeStore::new();
    let hier = MapTypeHierarchy::new();
    let int_ty = store.nominal(test_decl("Int"));
    let mut session = InferenceSession::new();
    let var = session.fresh_variable(KindId::TYPE);

    session.add_constraint_with_support(
        InferenceRelation::Subtype(InferenceTerm::Canonical(int_ty), InferenceTerm::Var(var)),
        ConstraintOrigin::Argument {
            call: phalcom_semantic::identity::ExpressionId::new(phalcom_semantic::identity::BodyId(1), phalcom_semantic::identity::LocalExpressionId(1)),
            argument: phalcom_semantic::identity::ExpressionId::new(phalcom_semantic::identity::BodyId(1), phalcom_semantic::identity::LocalExpressionId(2)),
            parameter_index: 0,
        },
        None,
        InferenceSupport::Assumed,
    );

    let outcome = session.solve(&mut store, &hier);
    let InferenceOutcome::Solved(solution) = outcome else {
        panic!("assumed value evidence should still solve generic variable");
    };
    assert_eq!(solution.support.get(&var), Some(&InferenceSupport::Assumed));
    assert_eq!(session.term_support(&InferenceTerm::Var(var)), Some(InferenceSupport::Assumed));
}

#[test]
fn binding_inference_variable_checks_canonical_kind() {
    use phalcom_semantic::types::parameter::{TypeParameterData, TypeParameterOwner};

    let mut store = TypeStore::new();
    let owner = TypeParameterOwner::Declaration(test_decl("HigherKinded"));
    let arrow_kind = store.arrow_kind(vec![KindId::TYPE].into_boxed_slice(), KindId::TYPE);
    let parameter = store.intern_type_parameter(TypeParameterData::new(owner, 0, "F", arrow_kind));
    let higher_kind_form = store.parameter_form(parameter);
    let int_ty = store.nominal(test_decl("Int"));
    let mut session = InferenceSession::new();
    let var = session.fresh_variable(arrow_kind);

    let failure = session.bind(var, int_ty, &store).expect_err("kind mismatch must remain explicit");
    assert_eq!(
        failure,
        InferenceFailureReason::KindMismatch {
            var,
            expected: arrow_kind,
            actual: store.kind_of(int_ty),
        }
    );
    assert_ne!(store.kind_of(higher_kind_form), KindId::TYPE);
}

#[test]
fn inference_proof_state_meet_preserves_unavailable_reasons() {
    let unknown = InferenceProofState::Unknown(UnknownReason::UnresolvedName("missing".into()));
    assert_eq!(InferenceProofState::Established.meet(unknown.clone()), unknown);
    assert_eq!(unknown.clone().meet(InferenceProofState::Dynamic(DynamicReason::ExplicitEscape)), unknown,);
    assert_eq!(
        InferenceProofState::Established.meet(InferenceProofState::Assumed),
        InferenceProofState::Assumed,
    );
}

#[test]
fn aliases_and_compound_returns_meet_proof_states() {
    let mut store = TypeStore::new();
    let hierarchy = MapTypeHierarchy::new();
    let int_ty = store.nominal(test_decl("Int"));
    let mut session = InferenceSession::new();
    let established = session.fresh_variable(KindId::TYPE);
    let assumed = session.fresh_variable(KindId::TYPE);
    let established_term = InferenceTerm::Var(established);
    let assumed_term = InferenceTerm::Var(assumed);

    session.record_required_premise(
        &established_term,
        ConstraintOrigin::Explicit,
        &TypeKnowledge::established(int_ty, EvidenceOrigin::Syntax),
        None,
    );
    session.record_required_premise(
        &assumed_term,
        ConstraintOrigin::Explicit,
        &TypeKnowledge::assumed(int_ty, EvidenceOrigin::Syntax),
        None,
    );
    session.add_constraint(
        InferenceRelation::Equivalent(established_term.clone(), assumed_term.clone()),
        ConstraintOrigin::Explicit,
        None,
    );
    session.add_constraint(
        InferenceRelation::Equivalent(established_term.clone(), InferenceTerm::Canonical(int_ty)),
        ConstraintOrigin::Explicit,
        None,
    );

    assert!(session.solve(&mut store, &hierarchy).is_solved());
    assert_eq!(session.proof_state_for_term(&established_term), InferenceProofState::Assumed,);

    let compound = InferenceTerm::Tuple(
        vec![
            InferenceTupleElement {
                label: None,
                term: established_term,
            },
            InferenceTupleElement {
                label: None,
                term: InferenceTerm::Var(session.fresh_variable(KindId::TYPE)),
            },
        ]
        .into_boxed_slice(),
    );
    assert_eq!(
        session.proof_state_for_term(&compound),
        InferenceProofState::Unknown(UnknownReason::UnderconstrainedTypeVariable),
    );
}

#[test]
fn return_proof_remembers_unknown_required_premise_after_substitution_solves() {
    let mut store = TypeStore::new();
    let hier = MapTypeHierarchy::new();
    let int_ty = store.nominal(test_decl("Int"));
    let mut session = InferenceSession::new();
    let variable = session.fresh_variable(KindId::TYPE);
    let term = InferenceTerm::Var(variable);

    session.record_required_premise(
        &term,
        ConstraintOrigin::Explicit,
        &TypeKnowledge::established(int_ty, EvidenceOrigin::Syntax),
        None,
    );
    session.add_constraint_with_support(
        InferenceRelation::Subtype(InferenceTerm::Canonical(int_ty), term.clone()),
        ConstraintOrigin::Explicit,
        None,
        InferenceSupport::Established,
    );
    session.record_required_premise(
        &term,
        ConstraintOrigin::Explicit,
        &TypeKnowledge::Unknown(UnknownReason::UnresolvedName("missing".into())),
        None,
    );

    assert!(session.solve(&mut store, &hier).is_solved());
    assert_eq!(
        session.proof_state_for_term(&term),
        InferenceProofState::Unknown(UnknownReason::UnresolvedName("missing".into())),
    );
}

#[test]
fn expected_only_selection_does_not_seed_generic_proof() {
    let mut store = TypeStore::new();
    let hier = MapTypeHierarchy::new();
    let int_ty = store.nominal(test_decl("Int"));
    let mut session = InferenceSession::new();
    let variable = session.fresh_variable(KindId::TYPE);
    let term = InferenceTerm::Var(variable);
    session.add_constraint(
        InferenceRelation::Subtype(term.clone(), InferenceTerm::Canonical(int_ty)),
        ConstraintOrigin::ExpectedResult {
            expression: phalcom_semantic::identity::ExpressionId::new(phalcom_semantic::identity::BodyId(1), phalcom_semantic::identity::LocalExpressionId(1)),
        },
        None,
    );

    assert!(session.solve(&mut store, &hier).is_solved());
    assert_eq!(
        session.proof_state_for_term(&term),
        InferenceProofState::Unknown(UnknownReason::UnderconstrainedTypeVariable),
    );
}

#[test]
fn variable_subtype_constraint_is_directed_and_permutation_stable() {
    let mut store = TypeStore::new();
    let mut hierarchy = MapTypeHierarchy::new();
    let int_decl = test_decl("Int");
    let number_decl = test_decl("Number");
    hierarchy.insert(int_decl.clone(), number_decl.clone());
    let int_ty = store.nominal(int_decl);
    let number_ty = store.nominal(number_decl);

    for permutation in [0_u8, 1_u8] {
        let mut session = InferenceSession::new();
        let sub = session.fresh_variable(KindId::TYPE);
        let sup = session.fresh_variable(KindId::TYPE);
        let directed = InferenceRelation::Subtype(InferenceTerm::Var(sub), InferenceTerm::Var(sup));
        let bind_sub = InferenceRelation::Equivalent(InferenceTerm::Var(sub), InferenceTerm::Canonical(int_ty));
        let bind_sup = InferenceRelation::Equivalent(InferenceTerm::Var(sup), InferenceTerm::Canonical(number_ty));
        if permutation == 0 {
            session.add_constraint(directed, ConstraintOrigin::Explicit, None);
            session.add_constraint(bind_sub, ConstraintOrigin::Explicit, None);
            session.add_constraint(bind_sup, ConstraintOrigin::Explicit, None);
        } else {
            session.add_constraint(bind_sub, ConstraintOrigin::Explicit, None);
            session.add_constraint(bind_sup, ConstraintOrigin::Explicit, None);
            session.add_constraint(directed, ConstraintOrigin::Explicit, None);
        }
        let outcome = session.solve(&mut store, &hierarchy);
        let InferenceOutcome::Solved(solution) = outcome else {
            panic!("directed subtype relation should accept Int <: Number");
        };
        assert_eq!(solution.substitutions.get(&sub), Some(&int_ty));
        assert_eq!(solution.substitutions.get(&sup), Some(&number_ty));
    }
}

#[test]
fn inference_solver_observes_cancellation_and_budget() {
    let mut store = TypeStore::new();
    let hierarchy = MapTypeHierarchy::new();
    let token = CancellationToken::new();
    token.cancel();
    let control = CheckerControl::new(QueryBudget::default(), &token);
    let mut cancelled = InferenceSession::new();
    let variable = cancelled.fresh_variable(KindId::TYPE);
    cancelled.add_constraint(
        InferenceRelation::Equivalent(InferenceTerm::Var(variable), InferenceTerm::Var(variable)),
        ConstraintOrigin::Explicit,
        None,
    );
    assert!(matches!(
        cancelled.solve_with_control(&mut store, &hierarchy, &control),
        InferenceOutcome::Cancelled
    ));

    let mut budgeted = InferenceSession::new();
    let variable = budgeted.fresh_variable(KindId::TYPE);
    budgeted.add_constraint(
        InferenceRelation::Equivalent(InferenceTerm::Var(variable), InferenceTerm::Var(variable)),
        ConstraintOrigin::Explicit,
        None,
    );
    let control = CheckerControl::new(QueryBudget::new(0), &CancellationToken::new());
    assert!(matches!(
        budgeted.solve_with_control(&mut store, &hierarchy, &control),
        InferenceOutcome::BudgetExceeded(_)
    ));
}
