use phalcom_modules::identity::{ModuleComponent, ModuleId, ModulePath, ResolvedProjectId};
use phalcom_modules::interface::InterfaceBuilder;
use phalcom_modules::linker::{LinkedModule, LinkedProgram};
use phalcom_modules::metadata::ModuleMetadata;
use phalcom_modules::source::ModuleKind;
use phalcom_semantic::db::QueryKey;
use phalcom_semantic::identity::DeclarationId;
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
        Arc::new(ParsedModuleUnit::new(module, ModuleKind::Module, None, Arc::from(source_code), program)),
    );

    SemanticWorkspaceInput { linked, sources, generation }
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
fn publication_effects_distinguish_initial_graph_build_from_body_edit() {
    let module = ModuleId::resolved(
        ResolvedProjectId::from_raw(101),
        ModulePath::from_components(vec![ModuleComponent::from_identifier("effects").unwrap()]),
    );
    let mut session = SemanticWorkspaceSession::new();
    let first = session.update(build_input(module.clone(), "class Sample { value() -> Int { 1 } }", 1));

    assert!(first.effects.diagnostics_changed.contains(&module));
    assert!(first.effects.source_index_changed.contains(&module));
    assert!(first.effects.formal_changed.contains(&module));
    assert!(first.effects.advisory_changed.contains(&module));
    assert!(first.effects.declaration_index_changed);
    assert!(first.effects.module_graph_changed);
    assert!(first.stats.project_graph_rebuilt);
    assert_eq!(first.stats.modules_relinked, 1);
    assert_eq!(first.stats.modules_recomputed, 1);
    assert_eq!(first.stats.callables_recomputed, 1);
    assert_eq!(first.stats.callables_reused, 0);
    assert_eq!(first.stats.source_indexes_recomputed, 1);
    assert_eq!(first.stats.advisory_sources_recomputed, 1);
    assert_eq!(first.stats.advisory_callables_recomputed, 1);

    let second = session.update(build_input(module.clone(), "class Sample { value() -> Int { 2 } }", 2));

    assert_eq!(first.snapshot.store.id(), second.snapshot.store.id());
    assert!(second.effects.source_index_changed.contains(&module));
    assert!(second.effects.formal_changed.contains(&module));
    assert!(second.effects.advisory_changed.contains(&module));
    assert!(!second.effects.declaration_index_changed);
    assert!(!second.effects.module_graph_changed);
    assert!(!second.stats.project_graph_rebuilt);
    assert_eq!(second.stats.modules_relinked, 0);
    assert_eq!(second.stats.modules_recomputed, 1);
    assert_eq!(second.stats.callables_recomputed, 1);
    assert_eq!(second.stats.callables_reused, 0);
    assert_eq!(second.stats.source_indexes_recomputed, 1);
    assert_eq!(second.stats.advisory_sources_recomputed, 1);
    assert_eq!(second.stats.advisory_callables_recomputed, 1);
}

#[test]
fn cancelled_update_preserves_last_known_good() {
    use phalcom_semantic::db::{CancellationToken, QueryBudget, QueryOutcome};

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
    let published = session.last_snapshot().expect("published snapshot exists");
    assert_eq!(published.generation, 1, "cancelled candidate must not replace published snapshot");
    assert_eq!(published.sources[&module].text.as_ref(), src1);

    let src3 = r#"
class Worker {
  work -> Int {
    300
  }
}
"#;
    let budgeted = session.update_with_budget_and_cancel(build_input(module.clone(), src3, 3), QueryBudget::new(0), &CancellationToken::new());
    assert!(matches!(budgeted, Err(QueryOutcome::BudgetExceeded(_))));
    let published = session.last_snapshot().expect("published snapshot remains available");
    assert_eq!(published.generation, 1, "budget-exceeded candidate must not replace published snapshot");
    assert_eq!(published.sources[&module].text.as_ref(), src1);
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

    // Capture a type that already exists in revision 1. This proves the retained
    // snapshot itself owns a denotation for the old TypeId before later interning.
    let point_decl = phalcom_semantic::identity::DeclarationId::new(module.clone(), "Point".into());
    let point_name_ty = snapshot1.declarations.form(&point_decl).expect("Point form exists in revision-1 snapshot");
    let original_data = snapshot1.store.get(point_name_ty).clone();

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

    // The retained snapshot and later live store both preserve the exact old denotation.
    assert_eq!(snapshot1.store.get(point_name_ty), &original_data);
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

#[test]
fn generic_parameter_semantic_edits_version_ids_and_preserve_old_denotations() {
    use phalcom_semantic::diagnostic::SemanticSourceSpan;
    use phalcom_semantic::types::parameter::{TypeParameterData, TypeParameterOwner};
    use phalcom_semantic::{KindId, TypeStore};

    let module = ModuleId::resolved(
        ResolvedProjectId::from_raw(91),
        ModulePath::from_components(vec![ModuleComponent::from_identifier("types").unwrap()]),
    );
    let declaration = phalcom_semantic::identity::DeclarationId::new(module.clone(), "Functor".into());
    let owner = TypeParameterOwner::Declaration(declaration);
    let mut store = TypeStore::new();

    let first = store.intern_type_parameter(
        TypeParameterData::new(owner.clone(), 0, "F", KindId::TYPE)
            .with_source(SemanticSourceSpan::new(module.clone(), phalcom_common::range::SourceRange::new(10, 11))),
    );
    let first_form = store.parameter_form(first);
    let retained = store.clone();

    let constructor_kind = store.arrow_kind(Box::new([KindId::TYPE]), KindId::TYPE);
    let second = store.intern_type_parameter(
        TypeParameterData::new(owner.clone(), 0, "F", constructor_kind)
            .with_source(SemanticSourceSpan::new(module, phalcom_common::range::SourceRange::new(20, 21))),
    );
    let second_form = store.parameter_form(second);

    assert_ne!(first, second, "semantic binder changes require a new TypeParameterId version");
    assert_ne!(first_form, second_form, "different parameter kinds require distinct canonical TypeIds");
    assert_eq!(store.find_type_parameter_id(&owner, 0), Some(second));
    assert_eq!(retained.kind_of(first_form), KindId::TYPE);
    assert_eq!(store.kind_of(first_form), KindId::TYPE, "old live-store forms must keep their original kind");
    assert_eq!(store.kind_of(second_form), constructor_kind);
}

#[test]
fn generic_parameter_source_moves_refresh_provenance_without_changing_semantic_identity() {
    use phalcom_semantic::diagnostic::SemanticSourceSpan;
    use phalcom_semantic::types::parameter::{TypeParameterData, TypeParameterOwner};
    use phalcom_semantic::{KindId, TypeStore};

    let module = ModuleId::resolved(
        ResolvedProjectId::from_raw(92),
        ModulePath::from_components(vec![ModuleComponent::from_identifier("provenance").unwrap()]),
    );
    let declaration = phalcom_semantic::identity::DeclarationId::new(module.clone(), "Box".into());
    let owner = TypeParameterOwner::Declaration(declaration);
    let mut store = TypeStore::new();

    let first = store.intern_type_parameter(
        TypeParameterData::new(owner.clone(), 0, "T", KindId::TYPE)
            .with_source(SemanticSourceSpan::new(module.clone(), phalcom_common::range::SourceRange::new(1, 2))),
    );
    let retained = store.clone();
    let second = store.intern_type_parameter(
        TypeParameterData::new(owner, 0, "T", KindId::TYPE).with_source(SemanticSourceSpan::new(module, phalcom_common::range::SourceRange::new(50, 51))),
    );

    assert_eq!(first, second, "source-only movement must not perturb semantic type identity");
    assert_eq!(
        retained.type_parameter(first).source.as_ref().unwrap().range,
        phalcom_common::range::SourceRange::new(1, 2)
    );
    assert_eq!(
        store.type_parameter(second).source.as_ref().unwrap().range,
        phalcom_common::range::SourceRange::new(50, 51)
    );
}

#[test]
fn workspace_generic_kind_edit_versions_parameter_and_nominal_forms() {
    use phalcom_semantic::KindId;

    let module = ModuleId::resolved(
        ResolvedProjectId::from_raw(93),
        ModulePath::from_components(vec![ModuleComponent::from_identifier("generic_kind").unwrap()]),
    );
    let declaration = phalcom_semantic::identity::DeclarationId::new(module.clone(), "Holder".into());
    let mut session = SemanticWorkspaceSession::new();

    let update1 = session.update(build_input(module.clone(), "class Holder<F: Type -> Type> {}", 1));
    assert!(!update1.snapshot.has_errors());
    let signature1 = update1
        .snapshot
        .declarations
        .generic_signature(&declaration)
        .expect("revision-1 generic signature");
    let parameter1 = signature1.parameters[0];
    let mut snapshot_store1 = (*update1.snapshot.store).clone();
    let parameter_form1 = snapshot_store1.parameter_form(parameter1);
    let declaration_form1 = update1.snapshot.declarations.form(&declaration).expect("revision-1 declaration form");
    let parameter_kind1 = update1.snapshot.store.type_parameter(parameter1).kind;
    assert_ne!(parameter_kind1, KindId::TYPE);

    let update2 = session.update(build_input(module, "class Holder<F> {}", 2));
    assert!(!update2.snapshot.has_errors());
    let signature2 = update2
        .snapshot
        .declarations
        .generic_signature(&declaration)
        .expect("revision-2 generic signature");
    let parameter2 = signature2.parameters[0];
    let mut snapshot_store2 = (*update2.snapshot.store).clone();
    let parameter_form2 = snapshot_store2.parameter_form(parameter2);
    let declaration_form2 = update2.snapshot.declarations.form(&declaration).expect("revision-2 declaration form");

    assert_ne!(parameter1, parameter2, "kind edit must version the generic parameter identity");
    assert_ne!(parameter_form1, parameter_form2, "kind edit must version the parameter TypeId");
    assert_ne!(
        declaration_form1, declaration_form2,
        "declaration forms with different kinds must not alias in the TypeStore"
    );
    assert_eq!(update1.snapshot.store.type_parameter(parameter1).kind, parameter_kind1);
    assert_eq!(update1.snapshot.store.kind_of(parameter_form1), parameter_kind1);
    assert_eq!(update2.snapshot.store.type_parameter(parameter2).kind, KindId::TYPE);
    assert_eq!(update2.snapshot.store.kind_of(parameter_form2), KindId::TYPE);
}

#[test]
fn generic_kind_shell_change_recomputes_dependent_surface() {
    let module = ModuleId::resolved(
        ResolvedProjectId::from_raw(94),
        ModulePath::from_components(vec![ModuleComponent::from_identifier("generic_surface").unwrap()]),
    );
    let mut session = SemanticWorkspaceSession::new();
    let source1 = r#"
class Holder<F: Type -> Type> {}

class Consumer {
  @class use(_ value: Holder) -> Int { 1 }
}
"#;
    let _update1 = session.update(build_input(module.clone(), source1, 1));
    let holder = DeclarationId::new(module.clone(), "Holder".into());
    let consumer = DeclarationId::new(module.clone(), "Consumer".into());
    let holder_shell = QueryKey::DeclarationShell(holder);
    let consumer_surface = QueryKey::DeclarationSurface(consumer);
    let shell_fp1 = session.db().ready_product_fingerprint(&holder_shell).expect("holder shell product");
    let consumer_revision1 = session.db().query_state(&consumer_surface).expect("consumer surface").revision();
    let source2 = r#"
class Holder<F> {}

class Consumer {
  @class use(_ value: Holder) -> Int { 1 }
}
"#;
    let update2 = session.update(build_input(module, source2, 2));
    let shell_fp2 = session.db().ready_product_fingerprint(&holder_shell).expect("updated holder shell product");
    let consumer_state2 = session.db().query_state(&consumer_surface).expect("updated consumer surface");

    assert_ne!(shell_fp1, shell_fp2, "generic kind edit must change declaration-shell semantics");
    assert_ne!(
        consumer_revision1,
        consumer_state2.revision(),
        "dependent surface must recompute through shell product"
    );
    assert_eq!(consumer_state2.validated_revision(), Some(update2.snapshot.id.revision()));
}
