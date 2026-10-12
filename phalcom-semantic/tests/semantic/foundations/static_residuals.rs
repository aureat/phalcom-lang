use phalcom_semantic::checker::proof_predicate::ProofPredicateGoal;
use phalcom_semantic::db::QueryBudget;
use phalcom_semantic::field_key::FieldKey;
use phalcom_semantic::identity::{DeclarationId, ModuleId};
use phalcom_semantic::static_domain::StaticValueDomain;
use phalcom_semantic::static_eval::application::StaticSpecializationEnvironmentKey;
use phalcom_semantic::static_eval::residual::{
    StaticEvalTerm, StaticResidualApplication, StaticResidualData, StaticResidualParameter, StaticResidualStore,
    StaticResidualStoreError,
};
use phalcom_semantic::types::id::{KindId, TypeId};
use phalcom_semantic::types::outcome::BudgetKind;

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

    assert_eq!(hash_a, hash_b, "fingerprints must match despite inverted field insertion order and store lineages");
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

    let r1 = store
        .proof(g1, TypeId(1), StaticSpecializationEnvironmentKey::default(), &mut budget)
        .unwrap();
    let r2 = store
        .proof(g2, TypeId(1), StaticSpecializationEnvironmentKey::default(), &mut budget)
        .unwrap();

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
}
