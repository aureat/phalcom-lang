use phalcom_common::selector::Selector;
use phalcom_modules::identity::{ModuleComponent, ModuleId, ModulePath, ResolvedProjectId};
use phalcom_modules::interface::LinkedModuleInterface;
use phalcom_modules::linker::{LinkedModule, LinkedProgram, ModuleBindingLayout};
use phalcom_modules::metadata::ModuleMetadata;
use phalcom_modules::source::ModuleKind;
use phalcom_semantic::db::QueryKey;
use phalcom_semantic::identity::{CallableId, DeclarationId, DispatchSide};
use phalcom_semantic::session::SemanticWorkspaceSession;
use phalcom_semantic::source::ParsedModuleUnit;
use phalcom_semantic::workspace::SemanticWorkspaceInput;
use std::collections::BTreeMap;
use std::sync::Arc;

fn input(module: ModuleId, source: &str, generation: u64) -> SemanticWorkspaceInput {
    let parsed = phalcom_ast::parse(source, 0);
    let program = Arc::new(parsed.program);
    let linked_module = LinkedModule {
        interface: LinkedModuleInterface {
            module: module.clone(),
            kind: ModuleKind::Module,
            exports: BTreeMap::new(),
            metadata: ModuleMetadata::default(),
        },
        bindings: ModuleBindingLayout::default(),
        linked_reads: Vec::new(),
        runtime_dependencies: Vec::new(),
    };
    let linked = Arc::new(LinkedProgram {
        universe: Arc::new(phalcom_modules::project::ProjectUniverse::new()),
        modules: BTreeMap::from([(module.clone(), linked_module)]),
        graphs: phalcom_modules::graph::ModuleGraphs::default(),
        entry: module.clone(),
        initialization_order: vec![module.clone()],
    });
    let unit = Arc::new(ParsedModuleUnit::new(module.clone(), ModuleKind::Module, None, Arc::from(source), program));

    SemanticWorkspaceInput {
        linked,
        sources: BTreeMap::from([(module, unit)]),
        generation,
    }
}

#[test]
fn body_only_edit_stops_at_stable_semantic_products() {
    let module = ModuleId::resolved(
        ResolvedProjectId::from_raw(1),
        ModulePath::from_components(vec![ModuleComponent::from_identifier("main").unwrap()]),
    );
    let mut session = SemanticWorkspaceSession::new();

    let source_v1 = r#"
class Api {
  @class value() -> Int { 1 }
}

class Consumer {
  @class read() -> Int { Api.value() }
}
"#;
    let update1 = session.update(input(module.clone(), source_v1, 1));
    assert!(!update1.snapshot.has_errors());
    let rev1 = update1.snapshot.id.revision();

    let api = DeclarationId::new(module.clone(), "Api".into());
    let consumer = DeclarationId::new(module.clone(), "Consumer".into());
    let selector = Selector::method("value", []).unwrap();
    let api_value = CallableId::new(api.clone(), selector, DispatchSide::Class);
    let consumer_read = CallableId::new(consumer, Selector::method("read", []).unwrap(), DispatchSide::Class);

    let stable_keys = [
        QueryKey::LinkedInterface(module.clone()),
        QueryKey::HierarchyEdge(api.clone()),
        QueryKey::DeclarationSurface(api.clone()),
        QueryKey::CallableSignature(api_value.clone()),
        QueryKey::CallableBody(consumer_read.clone()),
    ];
    for key in &stable_keys {
        assert_eq!(session.db().query_state(key).unwrap().revision(), Some(rev1));
    }
    let consumer_v1 = update1.snapshot.callable_analyses.get(&consumer_read).unwrap().clone();

    let source_v2 = r#"
class Api {
  @class value() -> Int { 2 }
}

class Consumer {
  @class read() -> Int { Api.value() }
}
"#;
    let update2 = session.update(input(module.clone(), source_v2, 2));
    assert!(!update2.snapshot.has_errors());
    let rev2 = update2.snapshot.id.revision();
    assert_ne!(rev1, rev2);

    assert_eq!(
        session.db().query_state(&QueryKey::ParsedModule(module.clone())).unwrap().revision(),
        Some(rev2),
        "exact source bytes changed, so ParsedModule recomputes"
    );
    assert_eq!(
        session.db().query_state(&QueryKey::UnlinkedInterface(module.clone())).unwrap().revision(),
        Some(rev2),
        "UnlinkedInterface reevaluates from the changed parse even when its semantic product stays stable"
    );
    assert_eq!(
        session.db().query_state(&QueryKey::CallableBody(api_value)).unwrap().revision(),
        Some(rev2),
        "edited callable body recomputes"
    );

    for key in &stable_keys {
        let state = session.db().query_state(key).unwrap();
        assert_eq!(state.revision(), Some(rev1), "{key:?} must retain its original product computation");
        assert_eq!(
            state.validated_revision(),
            Some(rev2),
            "{key:?} must be revalidated for the current revision without recomputation"
        );
    }

    assert_eq!(update2.stats.callables_recomputed, 1);
    assert_eq!(update2.stats.callables_reused, 1);
    assert!(Arc::ptr_eq(&consumer_v1, update2.snapshot.callable_analyses.get(&consumer_read).unwrap()));
}

#[test]
fn range_only_body_edit_reuses_semantic_callers() {
    let module = ModuleId::resolved(
        ResolvedProjectId::from_raw(6),
        ModulePath::from_components(vec![ModuleComponent::from_identifier("range_only").unwrap()]),
    );
    let mut session = SemanticWorkspaceSession::new();
    let source1 = r#"
class Api {
  @class value() -> Int { 1 }
}

class Consumer {
  @class read() -> Int { Api.value() }
}
"#;
    let update1 = session.update(input(module.clone(), source1, 1));
    assert!(!update1.snapshot.has_errors());
    let api = DeclarationId::new(module.clone(), "Api".into());
    let consumer = DeclarationId::new(module.clone(), "Consumer".into());
    let api_callable = CallableId::new(api, Selector::method("value", []).unwrap(), DispatchSide::Class);
    let consumer_callable = CallableId::new(consumer, Selector::method("read", []).unwrap(), DispatchSide::Class);
    let api_body_key = QueryKey::CallableBody(api_callable);
    let consumer_body_key = QueryKey::CallableBody(consumer_callable);
    let rev1 = update1.snapshot.id.revision();
    assert_eq!(session.db().query_state(&consumer_body_key).unwrap().revision(), Some(rev1));

    let source2 = r#"
class Api {
  @class value() -> Int {
 1}
}

class Consumer {
  @class read() -> Int { Api.value() }
}
"#;
    let update2 = session.update(input(module, source2, 2));
    assert!(!update2.snapshot.has_errors());
    let rev2 = update2.snapshot.id.revision();
    assert_eq!(session.db().query_state(&api_body_key).unwrap().revision(), Some(rev2));
    let consumer_state = session.db().query_state(&consumer_body_key).unwrap();
    assert_eq!(consumer_state.revision(), Some(rev1));
    assert_eq!(consumer_state.validated_revision(), Some(rev2));
    assert_eq!(update2.stats.callables_recomputed, 1);
    assert_eq!(update2.stats.callables_reused, 1);
}

#[test]
fn callable_body_product_owns_tail_return_diagnostics() {
    use phalcom_semantic::diagnostic::DiagnosticCode;

    let module = ModuleId::resolved(
        ResolvedProjectId::from_raw(2),
        ModulePath::from_components(vec![ModuleComponent::from_identifier("diagnostics").unwrap()]),
    );
    let mut session = SemanticWorkspaceSession::new();
    let source = r#"
class Port {
  @class number() -> Int { "8080" }
}
"#;

    let update = session.update(input(module.clone(), source, 1));
    let callable = CallableId::new(
        DeclarationId::new(module, "Port".into()),
        Selector::method("number", []).unwrap(),
        DispatchSide::Class,
    );
    let body = session
        .db()
        .product(&QueryKey::CallableBody(callable))
        .and_then(|product| product.as_callable_body())
        .expect("callable body product");

    assert!(
        body.diagnostics.iter().any(|diagnostic| diagnostic.code == DiagnosticCode::ReturnMismatch),
        "tail-return mismatch must be owned by the DB callable-body product, not a second legacy class-body pass"
    );
    assert!(
        update
            .snapshot
            .diagnostics
            .values()
            .flat_map(|diagnostics| diagnostics.iter())
            .any(|diagnostic| diagnostic.code == DiagnosticCode::ReturnMismatch),
        "snapshot diagnostics must aggregate callable-body query diagnostics"
    );
}

#[test]
fn field_initializer_diagnostics_remain_after_callable_recheck_is_removed() {
    use phalcom_semantic::diagnostic::DiagnosticCode;

    let module = ModuleId::resolved(
        ResolvedProjectId::from_raw(3),
        ModulePath::from_components(vec![ModuleComponent::from_identifier("fields").unwrap()]),
    );
    let mut session = SemanticWorkspaceSession::new();
    let source = r#"
class Config {
  _port: Int = "invalid"
}
"#;

    let update = session.update(input(module, source, 1));
    assert!(
        update
            .snapshot
            .diagnostics
            .values()
            .flat_map(|diagnostics| diagnostics.iter())
            .any(|diagnostic| diagnostic.code == DiagnosticCode::FieldMismatch),
        "field initializer checking remains a non-callable class responsibility"
    );
}

#[test]
fn declaration_surface_query_owns_member_annotation_diagnostics() {
    use phalcom_semantic::diagnostic::DiagnosticCode;

    let module = ModuleId::resolved(
        ResolvedProjectId::from_raw(4),
        ModulePath::from_components(vec![ModuleComponent::from_identifier("surface_diagnostics").unwrap()]),
    );
    let mut session = SemanticWorkspaceSession::new();
    let source = r#"
class Handler {
  @class run(_ value: MissingType) -> Int { 1 }
}
"#;

    let update = session.update(input(module, source, 1));
    assert!(
        update
            .snapshot
            .diagnostics
            .values()
            .flat_map(|diagnostics| diagnostics.iter())
            .any(|diagnostic| diagnostic.code == DiagnosticCode::AnnotationUnresolved),
        "declaration-surface annotation diagnostics must survive removal of the duplicate legacy callable-body pass"
    );
}

#[test]
fn signature_edit_recomputes_exact_callers_and_reuses_unrelated_bodies() {
    let module = ModuleId::resolved(
        ResolvedProjectId::from_raw(5),
        ModulePath::from_components(vec![ModuleComponent::from_identifier("signature_edit").unwrap()]),
    );
    let mut session = SemanticWorkspaceSession::new();

    let source_v1 = r#"
class Api {
  @class value() -> Object { "one" }
}

class Consumer {
  @class read() { Api.value() }
}

class Unrelated {
  @class stable() -> Int { 7 }
}
"#;
    let update1 = session.update(input(module.clone(), source_v1, 1));
    assert!(!update1.snapshot.has_errors());
    let rev1 = update1.snapshot.id.revision();

    let api = DeclarationId::new(module.clone(), "Api".into());
    let consumer = DeclarationId::new(module.clone(), "Consumer".into());
    let unrelated = DeclarationId::new(module.clone(), "Unrelated".into());
    let api_value = CallableId::new(api.clone(), Selector::method("value", []).unwrap(), DispatchSide::Class);
    let consumer_read = CallableId::new(consumer, Selector::method("read", []).unwrap(), DispatchSide::Class);
    let unrelated_stable = CallableId::new(unrelated, Selector::method("stable", []).unwrap(), DispatchSide::Class);
    let unrelated_v1 = update1
        .snapshot
        .callable_analyses
        .get(&unrelated_stable)
        .expect("revision-1 unrelated analysis")
        .clone();

    let source_v2 = r#"
class Api {
  @class value() -> String { "two" }
}

class Consumer {
  @class read() { Api.value() }
}

class Unrelated {
  @class stable() -> Int { 7 }
}
"#;
    let update2 = session.update(input(module.clone(), source_v2, 2));
    assert!(!update2.snapshot.has_errors());
    let rev2 = update2.snapshot.id.revision();

    assert_eq!(
        session.db().query_state(&QueryKey::DeclarationSurface(api.clone())).unwrap().revision(),
        Some(rev2),
        "public member contract change must recompute the owning declaration surface"
    );
    assert_eq!(
        session.db().query_state(&QueryKey::CallableSignature(api_value.clone())).unwrap().revision(),
        Some(rev2),
        "callable signature product must change with the declared return type"
    );
    assert_eq!(
        session.db().query_state(&QueryKey::CallableBody(consumer_read)).unwrap().revision(),
        Some(rev2),
        "unchanged caller body must recompute because its consumed callable contract changed"
    );

    let unrelated_state = session
        .db()
        .query_state(&QueryKey::CallableBody(unrelated_stable.clone()))
        .expect("unrelated body query");
    assert_eq!(unrelated_state.revision(), Some(rev1));
    assert_eq!(unrelated_state.validated_revision(), Some(rev2));
    assert!(Arc::ptr_eq(&unrelated_v1, update2.snapshot.callable_analyses.get(&unrelated_stable).unwrap()));
}

#[test]
fn superclass_edit_recomputes_hierarchy_consumers_without_touching_unrelated_bodies() {
    let module = ModuleId::resolved(
        ResolvedProjectId::from_raw(6),
        ModulePath::from_components(vec![ModuleComponent::from_identifier("superclass_edit").unwrap()]),
    );
    let mut session = SemanticWorkspaceSession::new();

    let source_v1 = r#"
class BaseA {
  value() -> Int { 1 }
}
class BaseB {
  value() -> String { "b" }
}
class Child is BaseA {}
class Consumer {
  @class read(_ child: Child) { child.value() }
}
class Unrelated {
  @class stable() -> Int { 9 }
}
"#;
    let update1 = session.update(input(module.clone(), source_v1, 1));
    assert!(!update1.snapshot.has_errors());
    let rev1 = update1.snapshot.id.revision();

    let child = DeclarationId::new(module.clone(), "Child".into());
    let consumer_read = CallableId::new(
        DeclarationId::new(module.clone(), "Consumer".into()),
        Selector::method("read", [phalcom_common::selector::SelectorSlot::Positional]).unwrap(),
        DispatchSide::Class,
    );
    let unrelated_stable = CallableId::new(
        DeclarationId::new(module.clone(), "Unrelated".into()),
        Selector::method("stable", []).unwrap(),
        DispatchSide::Class,
    );
    let unrelated_v1 = update1
        .snapshot
        .callable_analyses
        .get(&unrelated_stable)
        .expect("revision-1 unrelated analysis")
        .clone();

    let source_v2 = r#"
class BaseA {
  value() -> Int { 1 }
}
class BaseB {
  value() -> String { "b" }
}
class Child is BaseB {}
class Consumer {
  @class read(_ child: Child) { child.value() }
}
class Unrelated {
  @class stable() -> Int { 9 }
}
"#;
    let update2 = session.update(input(module, source_v2, 2));
    assert!(!update2.snapshot.has_errors());
    let rev2 = update2.snapshot.id.revision();

    assert_eq!(
        session.db().query_state(&QueryKey::HierarchyEdge(child)).unwrap().revision(),
        Some(rev2),
        "changed direct superclass must recompute exactly that hierarchy edge"
    );
    assert_eq!(
        session.db().query_state(&QueryKey::CallableBody(consumer_read)).unwrap().revision(),
        Some(rev2),
        "dispatch consumer must recompute when its visited hierarchy path changes"
    );

    let unrelated_state = session
        .db()
        .query_state(&QueryKey::CallableBody(unrelated_stable.clone()))
        .expect("unrelated body query");
    assert_eq!(unrelated_state.revision(), Some(rev1));
    assert_eq!(unrelated_state.validated_revision(), Some(rev2));
    assert!(Arc::ptr_eq(&unrelated_v1, update2.snapshot.callable_analyses.get(&unrelated_stable).unwrap()));
}
