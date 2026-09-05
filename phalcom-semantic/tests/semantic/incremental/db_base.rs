use phalcom_modules::ModuleId;
use phalcom_semantic::db::{
    BudgetKind, CancellationToken, DependencyIndex, DependencyRecorder, InputFingerprint, ProductFingerprint, QueryBudget, QueryKey, QueryOutcome,
    QueryScheduler, QueryValue, SemanticDb,
};
use std::sync::Arc;

fn module() -> ModuleId {
    ModuleId::universe_root()
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
fn declaration_shell_query_publishes_typed_product_and_reuses_it() {
    use phalcom_semantic::declarations::DeclarationTypeInfo;
    use phalcom_semantic::types::id::{KindId, TypeId};

    let declaration = phalcom_semantic::DeclarationId::new(module(), "Shell".into());
    let info = Arc::new(DeclarationTypeInfo {
        declaration: declaration.clone(),
        form: TypeId(1),
        class_object_type: TypeId(2),
        kind: KindId::TYPE,
        generic_signature: None,
        supertype_template: None,
    });
    let mut db = SemanticDb::new();

    let first = phalcom_semantic::db::query_declaration_shell(&mut db, Arc::new(phalcom_semantic::TypeDeclarationShell::Nominal((*info).clone())));
    let first_product = match first {
        QueryOutcome::Ready(product) => product,
        other => panic!("expected shell product, got {other:?}"),
    };
    let key = QueryKey::DeclarationShell(declaration.clone());
    assert!(db.product(&key).and_then(|product| product.as_declaration_shell()).is_some());

    let second = phalcom_semantic::db::query_declaration_shell(&mut db, Arc::new(phalcom_semantic::TypeDeclarationShell::Nominal((*info).clone())));
    match second {
        QueryOutcome::Ready(product) => assert!(Arc::ptr_eq(&first_product, &product)),
        other => panic!("expected cached shell product, got {other:?}"),
    }
}

#[test]
fn purge_module_removes_last_known_good_products_and_edges() {
    use phalcom_semantic::declarations::DeclarationTypeInfo;
    use phalcom_semantic::types::id::{KindId, TypeId};

    let module = module();
    let declaration = phalcom_semantic::DeclarationId::new(module.clone(), "Obsolete".into());
    let info = Arc::new(DeclarationTypeInfo {
        declaration: declaration.clone(),
        form: TypeId(1),
        class_object_type: TypeId(2),
        kind: KindId::TYPE,
        generic_signature: None,
        supertype_template: None,
    });
    let mut db = SemanticDb::new();
    let _ = phalcom_semantic::db::query_declaration_shell(
        &mut db,
        Arc::new(phalcom_semantic::TypeDeclarationShell::Nominal((*info).clone())),
    );
    let key = QueryKey::DeclarationShell(declaration);
    assert!(db.last_known_good_product(&key).is_some());

    assert!(db.purge_module(&module) > 0);
    assert!(db.product(&key).is_none());
    assert!(db.last_known_good_product(&key).is_none());
    assert!(db.query_state(&key).is_none());
}

#[test]
fn stale_revision_cannot_publish_a_ready_product() {
    let mut db = SemanticDb::new();
    let key = QueryKey::ParsedModule(module());
    let stale = db.revision();
    let current = db.begin_revision();

    let error = db
        .publish_ready(
            key.clone(),
            stale,
            InputFingerprint::new(7),
            ProductFingerprint::new(7),
            QueryValue::from_bytes([1, 2, 3]),
            [],
        )
        .expect_err("old revision must be rejected");

    assert!(error.is_stale());
    assert_eq!(error.expected_revision(), current);
    assert!(db.query_state(&key).is_none());
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
    let dispatch = phalcom_semantic::dispatch::SurfaceDispatchResolver::new();
    let module = ModuleId::universe_root();
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

    let outcome1 = phalcom_semantic::db::query_signatureless_callable_body(
        &mut db,
        phalcom_semantic::db::CallableBodyQuery {
            callable: cid1.clone(),
            body: &[],
            body_range: SourceRange { start: 0, end: 10 },
            store: &mut store,
            hierarchy: &hierarchy,
            resolver: &resolver,
            declarations: &decls,
            dispatch: &dispatch,
            module: module.clone(),
            budget,
            cancel: &cancel,
            formal_inputs: None,
        },
    );
    assert!(outcome1.is_ready());

    let outcome1_cached = phalcom_semantic::db::query_signatureless_callable_body(
        &mut db,
        phalcom_semantic::db::CallableBodyQuery {
            callable: cid1.clone(),
            body: &[],
            body_range: SourceRange { start: 0, end: 10 },
            store: &mut store,
            hierarchy: &hierarchy,
            resolver: &resolver,
            declarations: &decls,
            dispatch: &dispatch,
            module: module.clone(),
            budget,
            cancel: &cancel,
            formal_inputs: None,
        },
    );
    match (&outcome1, &outcome1_cached) {
        (QueryOutcome::Ready(first), QueryOutcome::Ready(second)) => assert!(Arc::ptr_eq(first, second), "cache hit must return typed product"),
        _ => panic!("expected two ready callable products"),
    }
    let key1 = QueryKey::CallableBody(cid1.clone());
    assert!(db.query_state(&key1).unwrap().is_ready());
    assert_eq!(db.query_state(&key1).unwrap().revision(), Some(db.revision()));
    assert_eq!(db.query_state(&key1).unwrap().as_ready_value().unwrap().as_bytes(), b"callable-body");
    let first_input_fingerprint = db
        .query_state(&key1)
        .unwrap()
        .input_fingerprint()
        .expect("ready callable has input fingerprint");
    assert_ne!(first_input_fingerprint.raw(), 0);
    assert!(db.product(&key1).and_then(|product| product.as_callable_body()).is_some());

    let changed_body = phalcom_ast::parse_source("...", 0).expect("changed callable body parses");
    let changed = phalcom_semantic::db::query_signatureless_callable_body(
        &mut db,
        phalcom_semantic::db::CallableBodyQuery {
            callable: cid1.clone(),
            body: &changed_body.statements,
            body_range: SourceRange { start: 0, end: 10 },
            store: &mut store,
            hierarchy: &hierarchy,
            resolver: &resolver,
            declarations: &decls,
            dispatch: &dispatch,
            module: module.clone(),
            budget,
            cancel: &cancel,
            formal_inputs: None,
        },
    );
    match (&outcome1, &changed) {
        (QueryOutcome::Ready(first), QueryOutcome::Ready(second)) => {
            assert!(!Arc::ptr_eq(first, second), "changed callable body must not reuse old typed product");
        }
        _ => panic!("expected changed body to produce a ready callable product"),
    }
    let changed_input_fingerprint = db.query_state(&key1).unwrap().input_fingerprint().expect("changed body has input fingerprint");
    assert_ne!(first_input_fingerprint, changed_input_fingerprint);

    let failed_body = phalcom_ast::parse_source("2", 0).expect("failed callable body parses");
    let failed_cancel = CancellationToken::new();
    failed_cancel.cancel();
    let failed = phalcom_semantic::db::query_signatureless_callable_body(
        &mut db,
        phalcom_semantic::db::CallableBodyQuery {
            callable: cid1.clone(),
            body: &failed_body.statements,
            body_range: SourceRange { start: 0, end: 10 },
            store: &mut store,
            hierarchy: &hierarchy,
            resolver: &resolver,
            declarations: &decls,
            dispatch: &dispatch,
            module: module.clone(),
            budget,
            cancel: &failed_cancel,
            formal_inputs: None,
        },
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

    let outcome2 = phalcom_semantic::db::query_signatureless_callable_body(
        &mut db,
        phalcom_semantic::db::CallableBodyQuery {
            callable: cid2.clone(),
            body: &[],
            body_range: SourceRange { start: 0, end: 10 },
            store: &mut store,
            hierarchy: &hierarchy,
            resolver: &resolver,
            declarations: &decls,
            dispatch: &dispatch,
            module,
            budget,
            cancel: &cancel,
            formal_inputs: None,
        },
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

#[test]
fn callable_body_query_fails_closed_when_consumed_signature_product_is_missing() {
    use phalcom_common::range::SourceRange;
    use phalcom_common::selector::Selector;
    use phalcom_semantic::declarations::DeclarationTypeTable;
    use phalcom_semantic::dispatch::{CallableSignature, SurfaceDispatchResolver};
    use phalcom_semantic::identity::{CallableId, DeclarationId, DispatchSide};
    use phalcom_semantic::surface::DeclarationSurface;
    use phalcom_semantic::types::annotation::SimpleTypeResolver;
    use phalcom_semantic::types::evidence::{EvidenceOrigin, TypeKnowledge};
    use phalcom_semantic::types::relation::MapTypeHierarchy;
    use phalcom_semantic::types::store::TypeStore;

    let mut db = SemanticDb::new();
    let mut store = TypeStore::new();
    let hierarchy = MapTypeHierarchy::new();
    let resolver = SimpleTypeResolver::new();
    let declarations = DeclarationTypeTable::new();
    let module = ModuleId::universe_root();
    let cancel = CancellationToken::new();
    let budget = QueryBudget::default();

    let owner = DeclarationId::new(module.clone(), "Owner".into());
    let selector = Selector::getter("value").unwrap();
    let callable = CallableId::new(owner.clone(), selector.clone(), DispatchSide::Instance);

    let signature = CallableSignature::new(selector, Vec::new(), TypeKnowledge::assumed(store.unit(), EvidenceOrigin::DeveloperAnnotation));
    let mut surface = DeclarationSurface::new(Some(owner.clone()));
    surface.add_callable(DispatchSide::Instance, signature);
    let mut dispatch = SurfaceDispatchResolver::new();
    dispatch.register_surface(owner, surface);

    let outcome = phalcom_semantic::db::query_callable_body(
        &mut db,
        phalcom_semantic::db::CallableBodyQuery {
            callable: callable.clone(),
            body: &[],
            body_range: SourceRange { start: 0, end: 10 },
            store: &mut store,
            hierarchy: &hierarchy,
            resolver: &resolver,
            declarations: &declarations,
            dispatch: &dispatch,
            module,
            budget,
            cancel: &cancel,
            formal_inputs: None,
        },
    );

    match outcome {
        QueryOutcome::Failed(message) => {
            assert!(
                message.contains("CallableSignature") && !message.contains("DeclarationSurface"),
                "failure identifies the missing canonical callable-signature prerequisite: {message}"
            );
        }
        other => panic!("missing required signature product must fail closed, got {other:?}"),
    }

    let body_key = QueryKey::CallableBody(callable);
    assert!(db.product(&body_key).is_none(), "failed dependency recording must not publish CallableBody");
}
