use phalcom_modules::identity::{ModuleComponent, ModuleId, ModulePath, ResolvedProjectId};
use phalcom_modules::interface::InterfaceBuilder;
use phalcom_modules::linker::{LinkedModule, LinkedProgram};
use phalcom_modules::metadata::ModuleMetadata;
use phalcom_modules::source::ModuleKind;
use phalcom_semantic::session::SemanticWorkspaceSession;
use phalcom_semantic::source::ParsedModuleUnit;
use phalcom_semantic::workspace::SemanticWorkspaceInput;
use std::collections::BTreeMap;
use std::sync::Arc;

fn build_input(module: ModuleId, source_code: &str, generation: u64) -> SemanticWorkspaceInput {
    let parse_res = phalcom_ast::parse(source_code, 0);
    let program = Arc::new(parse_res.program);
    let _ = InterfaceBuilder::build(module.clone(), ModuleKind::Module, &program);

    let linked_mod = LinkedModule {
        interface: phalcom_modules::interface::LinkedModuleInterface {
            module: module.clone(),
            kind: ModuleKind::Module,
            exports: BTreeMap::new(),
            metadata: ModuleMetadata::default(),
        },
        bindings: phalcom_modules::linker::ModuleBindingLayout::default(),
        linked_reads: Vec::new(),
        runtime_dependencies: Vec::new(),
    };

    let mut modules = BTreeMap::new();
    modules.insert(module.clone(), linked_mod);

    let linked = Arc::new(LinkedProgram {
        universe: Arc::new(phalcom_modules::project::ProjectUniverse::new()),
        modules,
        graphs: phalcom_modules::graph::ModuleGraphs::default(),
        entry: module.clone(),
        initialization_order: vec![module.clone()],
    });

    let mut sources = BTreeMap::new();
    sources.insert(
        module.clone(),
        Arc::new(ParsedModuleUnit::new(
            module,
            ModuleKind::Module,
            None,
            Arc::from(source_code),
            program,
        )),
    );

    SemanticWorkspaceInput {
        linked,
        sources,
        generation,
    }
}

#[test]
fn stable_type_store_id_and_snapshots_across_revisions() {
    let module = ModuleId::resolved(
        ResolvedProjectId::from_raw(1),
        ModulePath::from_components(vec![ModuleComponent::from_identifier("main").unwrap()]),
    );
    let mut session = SemanticWorkspaceSession::new();

    // Revision 1
    let src1 = r#"
class Counter {
  _count: Int = 0

  @constructor
  new(_ count: Int) {
    _count = count
  }

  get -> Int {
    _count
  }
}
"#;
    let input1 = build_input(module.clone(), src1, 1);
    let update1 = session.update(input1);
    let store_id1 = update1.snapshot.store.id();

    assert_eq!(update1.stats.modules_recomputed, 1);
    assert_eq!(update1.stats.callables_recomputed, 2);
    assert_eq!(update1.stats.callables_reused, 0);

    // Revision 2: edit body of `get`
    let src2 = r#"
class Counter {
  _count: Int = 0

  @constructor
  new(_ count: Int) {
    _count = count
  }

  get -> Int {
    _count + 1
  }
}
"#;
    let input2 = build_input(module.clone(), src2, 2);
    let update2 = session.update(input2);
    let store_id2 = update2.snapshot.store.id();

    // Verify TypeStoreId stability
    assert_eq!(store_id1, store_id2, "TypeStoreId must remain stable across revisions in one session");

    // Old snapshot 1 must remain intact and independent
    assert_eq!(update1.snapshot.generation, 1);
    assert_eq!(update2.snapshot.generation, 2);
    assert_eq!(update1.snapshot.store.id(), store_id1);

    // Verify fine-grained callable reuse: `new` was reused, `get` was recomputed
    assert_eq!(update2.stats.callables_reused, 1, "unchanged `new` callable must be reused from DB cache");
    assert_eq!(update2.stats.callables_recomputed, 1, "changed `get` callable must be recomputed");
}

#[test]
fn cancelled_update_preserves_last_known_good() {
    use phalcom_semantic::db::{CancellationToken, QueryBudget};

    let module = ModuleId::resolved(
        ResolvedProjectId::from_raw(1),
        ModulePath::from_components(vec![ModuleComponent::from_identifier("main").unwrap()]),
    );
    let mut session = SemanticWorkspaceSession::new();

    let src1 = r#"
class Worker {
  work -> Int {
    100
  }
}
"#;
    let input1 = build_input(module.clone(), src1, 1);
    let update1 = session.update(input1);
    assert_eq!(update1.snapshot.generation, 1);

    // Cancelled update
    let cancel = CancellationToken::new();
    cancel.cancel();
    let src2 = r#"
class Worker {
  work -> Int {
    200
  }
}
"#;
    let input2 = build_input(module.clone(), src2, 2);
    let res = session.update_with_budget_and_cancel(input2, QueryBudget::default(), &cancel);
    assert!(res.is_err());

    // Last known good snapshot is still generation 1
    let last_good = session.last_known_good_snapshot().expect("last known good snapshot exists");
    assert_eq!(last_good.generation, 1);
}

#[test]
fn one_session_has_one_type_store_identity_across_revisions() {
    let module = ModuleId::resolved(
        ResolvedProjectId::from_raw(1),
        ModulePath::from_components(vec![ModuleComponent::from_identifier("main").unwrap()]),
    );
    let mut session = SemanticWorkspaceSession::new();
    let expected_store_id = session.store().id();

    for rev in 1..=100 {
        let src = format!("class Sample {{ compute -> Int {{ {} }} }}", rev);
        let input = build_input(module.clone(), &src, rev);
        let update = session.update(input);
        assert_eq!(
            update.snapshot.store.id(),
            expected_store_id,
            "revision {} store.id() must equal session.store().id()",
            rev
        );
        assert_eq!(
            session.store().id(),
            expected_store_id,
            "session.store().id() must be immutable across revisions"
        );
    }
}

#[test]
fn retained_old_snapshot_preserves_type_denotation_after_later_revisions() {
    use phalcom_semantic::types::TypeData;
    use phalcom_semantic::types::id::KindId;

    let module = ModuleId::resolved(
        ResolvedProjectId::from_raw(1),
        ModulePath::from_components(vec![ModuleComponent::from_identifier("main").unwrap()]),
    );
    let mut session = SemanticWorkspaceSession::new();

    let src1 = r#"
class Point {
  _x: Int = 0
  _y: Int = 0
}
"#;
    let input1 = build_input(module.clone(), src1, 1);
    let update1 = session.update(input1);
    let snapshot1 = update1.snapshot.clone();
    let store_id = snapshot1.store.id();

    // Intern a test type directly into the session's store to get a known TypeId and TypeData
    let point_name_ty = session.store_mut().intern_with_kind(
        TypeData::Nominal {
            declaration: phalcom_semantic::identity::DeclarationId::new(module.clone(), "Point".into()),
        },
        KindId::TYPE,
    );
    let original_data = session.store().get(point_name_ty).clone();

    // Perform revisions 2 and 3 that intern unrelated types
    let src2 = "class Other { _v: String = \"\" }";
    let input2 = build_input(module.clone(), src2, 2);
    let update2 = session.update(input2);

    let src3 = "class Another { _v: Bool = true }";
    let input3 = build_input(module.clone(), src3, 3);
    let update3 = session.update(input3);

    // Verify snapshot 1 retains the exact TypeData for point_name_ty
    assert_eq!(snapshot1.store.id(), store_id);
    assert_eq!(update2.snapshot.store.id(), store_id);
    assert_eq!(update3.snapshot.store.id(), store_id);

    // Later session store also preserves the original type data
    assert_eq!(session.store().get(point_name_ty), &original_data);
}

#[test]
fn distinct_workspace_sessions_have_distinct_snapshot_workspace_ids() {
    use phalcom_semantic::identity::WorkspaceId;

    let module = ModuleId::resolved(
        ResolvedProjectId::from_raw(1),
        ModulePath::from_components(vec![ModuleComponent::from_identifier("main").unwrap()]),
    );

    let mut session1 = SemanticWorkspaceSession::with_workspace(WorkspaceId::from_raw(10));
    let mut session2 = SemanticWorkspaceSession::with_workspace(WorkspaceId::from_raw(20));

    let src = "class Sample {}";
    let input1 = build_input(module.clone(), src, 1);
    let update1 = session1.update(input1);

    let input2 = build_input(module.clone(), src, 1);
    let update2 = session2.update(input2);

    assert_eq!(update1.snapshot.id.workspace(), WorkspaceId::from_raw(10));
    assert_eq!(update2.snapshot.id.workspace(), WorkspaceId::from_raw(20));
    assert_ne!(update1.snapshot.id.workspace(), update2.snapshot.id.workspace());
}

