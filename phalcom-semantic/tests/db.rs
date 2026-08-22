use phalcom_modules::ModuleId;
use phalcom_semantic::db::{
    BudgetKind, CancellationToken, DependencyEdge, DependencyIndex, DependencyRecorder, ProductFingerprint, QueryBudget, QueryKey, QueryOutcome,
    QueryScheduler, QueryValue, SemanticDb,
};

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
    assert!(matches!(cancellation.check(), Err(_)));
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
