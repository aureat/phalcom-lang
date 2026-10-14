use phalcom_semantic::checker::proof_predicate::ProofPredicateGoal;
use phalcom_semantic::db::{CancellationToken, QueryBudget};
use phalcom_semantic::field_key::FieldKey;
use phalcom_semantic::identity::{DeclarationId, ModuleId};
use phalcom_semantic::match_semantics::BranchProofEnvironment;
use phalcom_semantic::static_domain::StaticValueDomain;
use phalcom_semantic::static_eval::application::{
    CanonicalStaticProofFact, StaticSpecializationArgument, StaticSpecializationEnvironmentError, StaticSpecializationEnvironmentKey,
};
use phalcom_semantic::static_eval::residual::{
    StaticEvalTerm, StaticResidualApplication, StaticResidualData, StaticResidualParameter, StaticResidualStore, StaticResidualStoreError,
};
use phalcom_semantic::types::constraint::TypeConstraint;
use phalcom_semantic::types::id::{KindId, TypeId, TypeParameterId};
use phalcom_semantic::types::outcome::BudgetKind;
use phalcom_semantic::types::parameter::{TypeParameterData, TypeParameterOwner};
use phalcom_semantic::types::store::TypeStore;

#[test]
fn test_same_node_interns_once() {
    let mut store = StaticResidualStore::new();
    let mut budget = QueryBudget::default();

    let decl = DeclarationId::new(ModuleId::universe_root(), "Foo".into());
    let param = StaticResidualParameter {
        owner: decl.clone(),
        index: 0,
        domain: StaticValueDomain::Kind(KindId::TYPE),
    };

    let ref1 = store.parameter(param.clone(), &mut budget).expect("intern parameter 1");
    let ref2 = store.parameter(param, &mut budget).expect("intern parameter 2");

    assert_eq!(ref1, ref2, "interning equivalent residual must yield identical StaticResidualRef");
    assert_eq!(budget.residual_nodes_created, 1, "memo hit must not charge additional residual nodes");
}

#[test]
fn test_different_parameters_do_not_collide() {
    let mut store = StaticResidualStore::new();
    let mut budget = QueryBudget::default();

    let decl1 = DeclarationId::new(ModuleId::universe_root(), "Foo".into());
    let decl2 = DeclarationId::new(ModuleId::universe_root(), "Bar".into());

    let param1 = StaticResidualParameter {
        owner: decl1.clone(),
        index: 0,
        domain: StaticValueDomain::Kind(KindId::TYPE),
    };
    let param2 = StaticResidualParameter {
        owner: decl1,
        index: 1,
        domain: StaticValueDomain::Kind(KindId::TYPE),
    };
    let param3 = StaticResidualParameter {
        owner: decl2,
        index: 0,
        domain: StaticValueDomain::Kind(KindId::TYPE),
    };

    let ref1 = store.parameter(param1, &mut budget).expect("ref1");
    let ref2 = store.parameter(param2, &mut budget).expect("ref2");
    let ref3 = store.parameter(param3, &mut budget).expect("ref3");

    assert_ne!(ref1, ref2);
    assert_ne!(ref1, ref3);
    assert_ne!(ref2, ref3);
    assert_eq!(budget.residual_nodes_created, 3);
}

#[test]
fn test_cross_store_refs_rejected() {
    let mut store_a = StaticResidualStore::new();
    let mut store_b = StaticResidualStore::new();
    let mut budget = QueryBudget::default();

    let decl = DeclarationId::new(ModuleId::universe_root(), "Foo".into());
    let param = StaticResidualParameter {
        owner: decl,
        index: 0,
        domain: StaticValueDomain::Kind(KindId::TYPE),
    };

    let ref_a = store_a.parameter(param, &mut budget).expect("ref_a");
    let err = store_b.get(ref_a).unwrap_err();
    assert_eq!(err, StaticResidualStoreError::ForeignStore);

    let nested_err = store_b
        .tuple(Box::new([StaticEvalTerm::Residual(ref_a)]), TypeId(100), &mut budget)
        .unwrap_err();
    assert_eq!(nested_err, StaticResidualStoreError::ForeignStore);
}

#[test]
fn test_hash_independent_of_allocation_order() {
    let mut store_a = StaticResidualStore::new();
    let mut store_b = StaticResidualStore::new();
    let mut budget = QueryBudget::default();

    let decl = DeclarationId::new(ModuleId::universe_root(), "Foo".into());

    let p0 = StaticResidualParameter {
        owner: decl.clone(),
        index: 0,
        domain: StaticValueDomain::Kind(KindId::TYPE),
    };
    let p1 = StaticResidualParameter {
        owner: decl.clone(),
        index: 1,
        domain: StaticValueDomain::Kind(KindId::TYPE),
    };

    // Store A interns p0 then p1
    let r0_a = store_a.parameter(p0.clone(), &mut budget).unwrap();
    let r1_a = store_a.parameter(p1.clone(), &mut budget).unwrap();

    // Store B interns p1 then p0
    let r1_b = store_b.parameter(p1, &mut budget).unwrap();
    let r0_b = store_b.parameter(p0, &mut budget).unwrap();

    // Now build a Record in both stores with fields in different order
    let rec_a = store_a
        .record(
            vec![
                (FieldKey::symbol("x"), StaticEvalTerm::Residual(r0_a)),
                (FieldKey::symbol("y"), StaticEvalTerm::Residual(r1_a)),
            ],
            TypeId(100),
            &mut budget,
        )
        .unwrap();

    let rec_b = store_b
        .record(
            vec![
                (FieldKey::symbol("y"), StaticEvalTerm::Residual(r1_b)),
                (FieldKey::symbol("x"), StaticEvalTerm::Residual(r0_b)),
            ],
            TypeId(100),
            &mut budget,
        )
        .unwrap();

    let hash_a = store_a.content_fingerprint(rec_a).expect("fingerprint a");
    let hash_b = store_b.content_fingerprint(rec_b).expect("fingerprint b");

    assert_eq!(
        hash_a, hash_b,
        "fingerprints must match despite inverted field insertion order for identical IDs within compatible lineages"
    );
}

#[test]
fn test_equivalent_proof_fingerprint_is_commutative() {
    let mut store = StaticResidualStore::new();
    let mut budget = QueryBudget::default();

    let g1 = ProofPredicateGoal::Equivalent {
        left: TypeId(42),
        right: TypeId(99),
    };
    let g2 = ProofPredicateGoal::Equivalent {
        left: TypeId(99),
        right: TypeId(42),
    };

    let r1 = store.proof(g1, TypeId(1), StaticSpecializationEnvironmentKey::default(), &mut budget).unwrap();
    let r2 = store.proof(g2, TypeId(1), StaticSpecializationEnvironmentKey::default(), &mut budget).unwrap();

    assert_eq!(r1, r2, "symmetric Equivalent goals normalize to one residual node");

    let h1 = store.content_fingerprint(r1).unwrap();
    let h2 = store.content_fingerprint(r2).unwrap();

    assert_eq!(h1, h2, "Equivalent goal fingerprints must be order-independent");
}

#[test]
fn test_residual_node_growth_budget_exhaustion() {
    let mut store = StaticResidualStore::new();
    let mut budget = QueryBudget::default();
    budget.max_residual_nodes = 2;

    let decl = DeclarationId::new(ModuleId::universe_root(), "Foo".into());
    let p0 = StaticResidualParameter {
        owner: decl.clone(),
        index: 0,
        domain: StaticValueDomain::Kind(KindId::TYPE),
    };
    let p1 = StaticResidualParameter {
        owner: decl.clone(),
        index: 1,
        domain: StaticValueDomain::Kind(KindId::TYPE),
    };
    let p2 = StaticResidualParameter {
        owner: decl,
        index: 2,
        domain: StaticValueDomain::Kind(KindId::TYPE),
    };

    assert!(store.parameter(p0.clone(), &mut budget).is_ok());
    assert!(store.parameter(p1, &mut budget).is_ok());
    // Re-interning p0 is free
    assert!(store.parameter(p0, &mut budget).is_ok());

    // Creating 3rd new node exceeds budget
    let StaticResidualStoreError::BudgetExceeded(err) = store.parameter(p2, &mut budget).unwrap_err() else {
        panic!("expected residual-node budget exhaustion");
    };
    assert_eq!(err.kind(), BudgetKind::ResidualNodes);
    assert_eq!(err.limit(), 2);
    assert_eq!(err.used(), 3);
}

#[test]
fn residual_payload_work_is_charged_before_interning_a_large_node() {
    let mut store = StaticResidualStore::new();
    let leaf = store
        .parameter(
            StaticResidualParameter {
                owner: DeclarationId::new(ModuleId::universe_root(), "Input".into()),
                index: 0,
                domain: StaticValueDomain::Kind(KindId::TYPE),
            },
            &mut QueryBudget::default(),
        )
        .unwrap();
    let mut budget = QueryBudget::new(0);
    let result = store.pack(
        vec![StaticEvalTerm::Residual(leaf); 2048].into_boxed_slice(),
        Box::new([]),
        Box::new([]),
        TypeId(42),
        &mut budget,
    );
    let report = match result {
        Err(StaticResidualStoreError::BudgetExceeded(report)) => report,
        other => panic!("a large residual payload must exhaust the work budget: {other:?}"),
    };
    assert_eq!(report.kind(), BudgetKind::Steps);
    assert_eq!(budget.residual_nodes_created, 0, "the rejected payload must not allocate a residual node");
}

#[test]
fn test_residual_result_domains() {
    let decl = DeclarationId::new(ModuleId::universe_root(), "Foo".into());
    let param = StaticResidualParameter {
        owner: decl.clone(),
        index: 0,
        domain: StaticValueDomain::Kind(KindId::TYPE),
    };
    let data_param = StaticResidualData::Parameter { parameter: param };
    assert_eq!(data_param.result_domain(), StaticValueDomain::Kind(KindId::TYPE));

    let apply = StaticResidualData::Apply {
        application: StaticResidualApplication {
            definition: decl,
            arguments: Box::new([]),
            environment: StaticSpecializationEnvironmentKey::default(),
        },
        result_domain: StaticValueDomain::ValueType(TypeId(42)),
    };
    assert_eq!(apply.result_domain(), StaticValueDomain::ValueType(TypeId(42)));

    let pack = StaticResidualData::Pack {
        positionals: Box::new([]),
        labels: Box::new([]),
        labeled_values: Box::new([]),
        result_type: TypeId(43),
    };
    assert_eq!(pack.result_domain(), StaticValueDomain::ValueType(TypeId(43)));
}

#[test]
fn pack_residual_rejects_mismatched_or_duplicate_symbol_lanes() {
    let mut store = StaticResidualStore::new();
    let mut budget = QueryBudget::default();
    let malformed = store.pack(Box::new([]), Box::new([Box::<str>::from("x")]), Box::new([]), TypeId(42), &mut budget);
    assert_eq!(malformed, Err(StaticResidualStoreError::InvalidPackLanes));
    let duplicate = store.pack(
        Box::new([]),
        Box::new([Box::<str>::from("x"), Box::<str>::from("x")]),
        Box::new([StaticEvalTerm::Kinded(TypeId(1)), StaticEvalTerm::Kinded(TypeId(2))]),
        TypeId(42),
        &mut budget,
    );
    assert_eq!(duplicate, Err(StaticResidualStoreError::DuplicatePackLabel("x".into())));
    assert_eq!(budget.residual_nodes_created, 0);
}

#[test]
fn cv_env_01_environment_normalization_laws() {
    let decl = DeclarationId::new(ModuleId::universe_root(), "Foo".into());
    let p0 = StaticResidualParameter {
        owner: decl.clone(),
        index: 0,
        domain: StaticValueDomain::Kind(KindId::TYPE),
    };
    let p1 = StaticResidualParameter {
        owner: decl,
        index: 1,
        domain: StaticValueDomain::Kind(KindId::TYPE),
    };

    let t0 = TypeParameterId(1);
    let t1 = TypeParameterId(2);

    let ty_a = TypeId(10);
    let ty_b = TypeId(20);

    // 1. Order independence
    let env_1 = StaticSpecializationEnvironmentKey::normalized(
        vec![
            (p0.clone(), StaticSpecializationArgument::Kinded(ty_a)),
            (p1.clone(), StaticSpecializationArgument::Kinded(ty_b)),
        ],
        vec![(t0, ty_a), (t1, ty_b)],
        vec![
            CanonicalStaticProofFact::Equal(ty_b, ty_a),
            CanonicalStaticProofFact::Subtype { lower: ty_a, upper: ty_b },
        ],
    )
    .unwrap();

    let env_2 = StaticSpecializationEnvironmentKey::normalized(
        vec![
            (p1.clone(), StaticSpecializationArgument::Kinded(ty_b)),
            (p0.clone(), StaticSpecializationArgument::Kinded(ty_a)),
        ],
        vec![(t1, ty_b), (t0, ty_a)],
        vec![
            CanonicalStaticProofFact::Subtype { lower: ty_a, upper: ty_b },
            CanonicalStaticProofFact::Equal(ty_a, ty_b),
        ],
    )
    .unwrap();

    assert_eq!(env_1, env_2, "normalization must be independent of input ordering");

    // 2. Duplicate coalescing
    let env_dup = StaticSpecializationEnvironmentKey::normalized(
        vec![
            (p0.clone(), StaticSpecializationArgument::Kinded(ty_a)),
            (p0.clone(), StaticSpecializationArgument::Kinded(ty_a)),
        ],
        vec![(t0, ty_a), (t0, ty_a)],
        vec![CanonicalStaticProofFact::Equal(ty_a, ty_b), CanonicalStaticProofFact::Equal(ty_b, ty_a)],
    )
    .unwrap();
    assert_eq!(env_dup.argument_bindings().len(), 1);
    assert_eq!(env_dup.type_bindings().len(), 1);
    assert_eq!(env_dup.constraints().len(), 1);

    // 3. Symmetric Equal canonicalization
    assert_eq!(env_dup.constraints()[0], CanonicalStaticProofFact::Equal(ty_a, ty_b));

    // 4. Directional Subtype preservation
    let env_sub_1 =
        StaticSpecializationEnvironmentKey::normalized(vec![], vec![], vec![CanonicalStaticProofFact::Subtype { lower: ty_a, upper: ty_b }]).unwrap();
    let env_sub_2 =
        StaticSpecializationEnvironmentKey::normalized(vec![], vec![], vec![CanonicalStaticProofFact::Subtype { lower: ty_b, upper: ty_a }]).unwrap();
    assert_ne!(env_sub_1, env_sub_2, "Subtype directionality must be preserved");

    // 5. Idempotence: N(N(E)) == N(E)
    let env_idemp = StaticSpecializationEnvironmentKey::normalized(
        env_1.argument_bindings().iter().cloned(),
        env_1.type_bindings().iter().copied(),
        env_1.constraints().iter().cloned(),
    )
    .unwrap();
    assert_eq!(env_1, env_idemp, "normalization must be idempotent");

    // 6. Conflicting same-layer bindings fail closed
    let conflict_arg = StaticSpecializationEnvironmentKey::normalized(
        vec![
            (p0.clone(), StaticSpecializationArgument::Kinded(ty_a)),
            (p0.clone(), StaticSpecializationArgument::Kinded(ty_b)),
        ],
        vec![],
        vec![],
    );
    assert_eq!(conflict_arg.unwrap_err(), StaticSpecializationEnvironmentError::ConflictingArgumentBinding);

    let conflict_ty = StaticSpecializationEnvironmentKey::normalized(vec![], vec![(t0, ty_a), (t0, ty_b)], vec![]);
    assert_eq!(conflict_ty.unwrap_err(), StaticSpecializationEnvironmentError::ConflictingTypeBinding);
}

#[test]
fn cv_env_04_branch_proof_nearest_binding_precedence() {
    let t0 = TypeParameterId(1);
    let ty_outer = TypeId(10);
    let ty_inner = TypeId(20);

    let mut outer_env = BranchProofEnvironment::default();
    outer_env.bindings.insert(t0, ty_outer);
    outer_env.equalities = Box::new([TypeConstraint::Equal(ty_outer, ty_inner)]);

    let mut inner_env = BranchProofEnvironment::default();
    inner_env.bindings.insert(t0, ty_inner);
    inner_env.equalities = Box::new([TypeConstraint::Subtype(ty_inner, ty_outer)]);

    // Active branch proofs stack has outer then inner (inner is nearest)
    let key = StaticSpecializationEnvironmentKey::from_branch_proofs([&outer_env, &inner_env]).unwrap();
    assert_eq!(key.type_bindings().len(), 1);
    assert_eq!(key.type_bindings()[0], (t0, ty_inner), "nearest active branch proof binding must win");
    assert_eq!(key.constraints().len(), 2, "constraints must be unioned");
}

#[test]
fn cv_sub_01_and_02_substitution_before_conflict_and_rejection() {
    let mut types = TypeStore::new();
    let mut budget = QueryBudget::default();
    let cancellation = CancellationToken::new();

    let owner = DeclarationId::new(ModuleId::universe_root(), "TestDecl".into());
    let p_u = types.intern_type_parameter(TypeParameterData::new(TypeParameterOwner::Declaration(owner.clone()), 0, "U", KindId::TYPE));
    let ty_u = types.parameter_form(p_u);

    let p_t = types.intern_type_parameter(TypeParameterData::new(TypeParameterOwner::Declaration(owner), 1, "T", KindId::TYPE));

    let boxed_decl = DeclarationId::new(ModuleId::universe_root(), "Boxed".into());
    let ctor_kind = types.arrow_kind(Box::new([KindId::TYPE]), KindId::TYPE);
    let boxed_origin = types.nominal_form(boxed_decl, ctor_kind);
    let boxed_u = types.apply_type_form(boxed_origin, &[ty_u]).unwrap();

    let ty_unit = types.unit();
    let boxed_unit = types.apply_type_form(boxed_origin, &[ty_unit]).unwrap();

    let other_decl = DeclarationId::new(ModuleId::universe_root(), "Other".into());
    let other_ty = types.nominal_type(other_decl);
    let boxed_other = types.apply_type_form(boxed_origin, &[other_ty]).unwrap();

    // Captured: T = Boxed<U>
    let captured = StaticSpecializationEnvironmentKey::normalized(vec![], vec![(p_t, boxed_u)], vec![]).unwrap();

    // Supplied 1: U = Unit, T = Boxed<Unit>
    let supplied_valid = StaticSpecializationEnvironmentKey::normalized(vec![], vec![(p_u, ty_unit), (p_t, boxed_unit)], vec![]).unwrap();

    let composed_valid = captured
        .compose_with_supplied(&supplied_valid, &mut types, &mut budget, &cancellation)
        .expect("substitution before conflict must succeed when substituted captured matches supplied");
    assert_eq!(composed_valid.type_bindings().len(), 2);

    // Supplied 2: U = Unit, T = Boxed<Other> (contradiction after U = Unit)
    let supplied_conflict = StaticSpecializationEnvironmentKey::normalized(vec![], vec![(p_u, ty_unit), (p_t, boxed_other)], vec![]).unwrap();

    let err = captured
        .compose_with_supplied(&supplied_conflict, &mut types, &mut budget, &cancellation)
        .unwrap_err();
    assert_eq!(err, StaticSpecializationEnvironmentError::ConflictingTypeBinding);
}

#[test]
fn specialization_environment_normalization() {
    cv_env_01_environment_normalization_laws();
}

#[test]
fn substitution_before_conflict() {
    cv_sub_01_and_02_substitution_before_conflict_and_rejection();
}
