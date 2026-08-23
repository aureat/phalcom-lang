use phalcom_modules::ModuleId;
use phalcom_semantic::db::{
    BudgetKind, CancellationToken, DependencyIndex, DependencyRecorder, ProductFingerprint, QueryBudget, QueryKey, QueryOutcome, QueryScheduler, QueryValue,
    SemanticDb,
};
use std::sync::Arc;

fn module() -> ModuleId {
    ModuleId::core()
}

#[test]
fn reverse_invalidation_reaches_full_dependency_closure_deterministically() {
    let leaf = QueryKey::ParsedModule(module());
    let middle = QueryKey::UnlinkedInterface(module());
    let root = QueryKey::LinkedInterface(module());
    let mut index = DependencyIndex::new();

    let mut middle_dependencies = DependencyRecorder::new(middle.clone());
    middle_dependencies.record(leaf.clone(), ProductFingerprint::new(11));
    index.replace_dependencies(middle.clone(), middle_dependencies.finish());

    let mut root_dependencies = DependencyRecorder::new(root.clone());
    root_dependencies.record(middle.clone(), ProductFingerprint::new(12));
    index.replace_dependencies(root.clone(), root_dependencies.finish());

    let closure = index.reverse_closure([leaf.clone()]);
    assert_eq!(closure.into_iter().collect::<Vec<_>>(), vec![leaf, middle, root]);
}

#[test]
fn cancellation_and_budget_are_distinct_terminal_outcomes() {
    let cancellation = CancellationToken::new();
    cancellation.cancel();
    assert!(cancellation.is_cancelled());
    assert!(cancellation.check().is_err());
    assert!(matches!(QueryOutcome::<()>::cancelled(), QueryOutcome::Cancelled));

    let mut budget = QueryBudget::new(1);
    assert!(budget.charge_step().is_ok());
    let report = budget.charge_step().expect_err("second step exceeds budget");
    assert_eq!(report.kind(), BudgetKind::Steps);
    assert!(matches!(QueryOutcome::<()>::budget_exceeded(report), QueryOutcome::BudgetExceeded(_)));
}

#[test]
fn scheduler_pops_unique_keys_in_canonical_order() {
    let parsed = QueryKey::ParsedModule(module());
    let linked = QueryKey::LinkedInterface(module());
    let shell = QueryKey::DeclarationShell(phalcom_semantic::DeclarationId::new(module(), "Shell".into()));
    let mut scheduler = QueryScheduler::new();

    scheduler.enqueue(linked.clone());
    scheduler.enqueue(parsed.clone());
    scheduler.enqueue(shell.clone());
    scheduler.enqueue(parsed.clone());

    assert_eq!(scheduler.len(), 3);
    assert_eq!(scheduler.pop_next(), Some(parsed));
    assert_eq!(scheduler.pop_next(), Some(linked));
    assert_eq!(scheduler.pop_next(), Some(shell));
    assert_eq!(scheduler.pop_next(), None);
}

#[test]
fn stale_revision_cannot_publish_a_ready_product() {
    let mut db = SemanticDb::new();
    let key = QueryKey::ParsedModule(module());
    let stale = db.revision();
    let current = db.begin_revision();

    let error = db
        .publish_ready(key.clone(), stale, ProductFingerprint::new(7), QueryValue::from_bytes([1, 2, 3]), [])
        .expect_err("old revision must be rejected");

    assert!(error.is_stale());
    assert_eq!(error.expected_revision(), current);
    assert!(db.query_state(&key).is_none());
}

#[test]
fn semantic_db_keeps_type_store_identity_across_revisions() {
    let mut db = SemanticDb::new();
    let store_id = db.store().id();
    let first_revision = db.revision();

    let second_revision = db.begin_revision();
    assert_ne!(first_revision, second_revision);
    assert_eq!(db.store().id(), store_id, "one semantic DB epoch keeps one TypeStoreId");

    db.begin_revision();
    assert_eq!(db.store().id(), store_id, "later revisions reuse the same type store");
}

#[test]
fn test_body_query_execution_and_invalidation() {
    use phalcom_common::range::SourceRange;
    use phalcom_common::selector::Selector;
    use phalcom_semantic::declarations::DeclarationTypeTable;
    use phalcom_semantic::identity::{CallableId, DeclarationId, DispatchSide};
    use phalcom_semantic::types::annotation::SimpleTypeResolver;
    use phalcom_semantic::types::relation::MapTypeHierarchy;
    use phalcom_semantic::types::store::TypeStore;

    let mut db = SemanticDb::new();
    let mut store = TypeStore::new();
    let hierarchy = MapTypeHierarchy::new();
    let resolver = SimpleTypeResolver::new();
    let decls = DeclarationTypeTable::new();
    let module = ModuleId::core();
    let cancel = CancellationToken::new();
    let budget = QueryBudget::default();

    let cid1 = CallableId::new(
        DeclarationId::new(module.clone(), "C1".into()),
        Selector::getter("m1").unwrap(),
        DispatchSide::Instance,
    );
    let cid2 = CallableId::new(
        DeclarationId::new(module.clone(), "C2".into()),
        Selector::getter("m2").unwrap(),
        DispatchSide::Instance,
    );

    let outcome1 = phalcom_semantic::db::query_callable_body(
        &mut db,
        cid1.clone(),
        &[],
        SourceRange { start: 0, end: 10 },
        &mut store,
        &hierarchy,
        &resolver,
        &decls,
        module.clone(),
        budget,
        &cancel,
    );
    assert!(outcome1.is_ready());

    let outcome1_cached = phalcom_semantic::db::query_callable_body(
        &mut db,
        cid1.clone(),
        &[],
        SourceRange { start: 0, end: 10 },
        &mut store,
        &hierarchy,
        &resolver,
        &decls,
        module.clone(),
        budget,
        &cancel,
    );
    match (&outcome1, &outcome1_cached) {
        (QueryOutcome::Ready(first), QueryOutcome::Ready(second)) => assert!(Arc::ptr_eq(first, second), "cache hit must return typed product"),
        _ => panic!("expected two ready callable products"),
    }
    let key1 = QueryKey::CallableBody(cid1.clone());
    assert_eq!(db.query_state(&key1).unwrap().is_ready(), true);
    assert_eq!(db.query_state(&key1).unwrap().revision(), Some(db.revision()));
    assert_eq!(db.query_state(&key1).unwrap().as_ready_value().unwrap().as_bytes(), b"callable-body");
    let first_input_fingerprint = db.query_state(&key1).unwrap().fingerprint().expect("ready callable has input fingerprint");
    assert_ne!(first_input_fingerprint.raw(), 0);
    assert!(db.product(&key1).and_then(|product| product.as_callable_body()).is_some());

    let changed_body = phalcom_ast::parse_source("1", 0).expect("changed callable body parses");
    let changed = phalcom_semantic::db::query_callable_body(
        &mut db,
        cid1.clone(),
        &changed_body.statements,
        SourceRange { start: 0, end: 10 },
        &mut store,
        &hierarchy,
        &resolver,
        &decls,
        module.clone(),
        budget,
        &cancel,
    );
    match (&outcome1, &changed) {
        (QueryOutcome::Ready(first), QueryOutcome::Ready(second)) => {
            assert!(!Arc::ptr_eq(first, second), "changed callable body must not reuse old typed product");
        }
        _ => panic!("expected changed body to produce a ready callable product"),
    }
    let changed_input_fingerprint = db.query_state(&key1).unwrap().fingerprint().expect("changed body has input fingerprint");
    assert_ne!(first_input_fingerprint, changed_input_fingerprint);

    let failed_body = phalcom_ast::parse_source("2", 0).expect("failed callable body parses");
    let failed_cancel = CancellationToken::new();
    failed_cancel.cancel();
    let failed = phalcom_semantic::db::query_callable_body(
        &mut db,
        cid1.clone(),
        &failed_body.statements,
        SourceRange { start: 0, end: 10 },
        &mut store,
        &hierarchy,
        &resolver,
        &decls,
        module.clone(),
        budget,
        &failed_cancel,
    );
    assert!(matches!(failed, QueryOutcome::Cancelled));
    assert!(db.product(&key1).is_none(), "cancelled generation must not appear current-ready");
    let last_good = db
        .last_known_good_product(&key1)
        .and_then(|product| product.as_callable_body())
        .expect("cancelled refresh retains last-known-good callable");
    let changed_arc = match &changed {
        QueryOutcome::Ready(product) => product,
        _ => panic!("changed body must have produced a ready product"),
    };
    assert!(Arc::ptr_eq(last_good, changed_arc), "last-known-good product must be prior ready result");

    let outcome2 = phalcom_semantic::db::query_callable_body(
        &mut db,
        cid2.clone(),
        &[],
        SourceRange { start: 0, end: 10 },
        &mut store,
        &hierarchy,
        &resolver,
        &decls,
        module,
        budget,
        &cancel,
    );
    assert!(outcome2.is_ready());

    let key1 = QueryKey::CallableBody(cid1);
    let key2 = QueryKey::CallableBody(cid2);

    assert!(matches!(db.query_state(&key1), Some(phalcom_semantic::db::QueryState::Cancelled { .. })));
    assert!(db.query_state(&key2).unwrap().is_ready());

    // Invalidate key1 only
    db.invalidate([key1.clone()]);
    assert!(db.query_state(&key1).is_none());
    assert!(db.query_state(&key2).unwrap().is_ready());
}
