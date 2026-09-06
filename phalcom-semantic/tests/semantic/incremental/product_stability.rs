use super::support::multi_module_input;
use phalcom_common::selector::Selector;
use phalcom_modules::identity::{ModuleComponent, ModuleId, ModulePath, ResolvedProjectId};
use phalcom_modules::interface::{InterfaceBuilder, LinkedExport, LinkedExportTarget, LinkedModuleInterface};
use phalcom_modules::linker::{GlobalBindingId, ImportBindingId, LinkedModule, LinkedProgram, LinkedReadSpec, ModuleBindingLayout, SymbolId};
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
    let local_globals = InterfaceBuilder::build(module.clone(), ModuleKind::Module, &program)
        .map(|interface| {
            interface
                .declarations
                .keys()
                .enumerate()
                .map(|(index, name)| (name.clone().into_boxed_str(), GlobalBindingId(index as u32)))
                .collect()
        })
        .unwrap_or_default();
    let linked_module = LinkedModule {
        interface: LinkedModuleInterface {
            module: module.clone(),
            kind: ModuleKind::Module,
            exports: BTreeMap::new(),
            metadata: ModuleMetadata::default(),
        },
        bindings: ModuleBindingLayout {
            local_globals,
            ..ModuleBindingLayout::default()
        },
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

    SemanticWorkspaceInput::new(linked, BTreeMap::from([(module, unit)]), generation)
}

fn layered_modules() -> (ModuleId, ModuleId, ModuleId) {
    let project = ResolvedProjectId::from_raw(56);
    (
        ModuleId::resolved(
            project,
            ModulePath::from_components(vec![ModuleComponent::from_identifier("stable_a").unwrap()]),
        ),
        ModuleId::resolved(
            project,
            ModulePath::from_components(vec![ModuleComponent::from_identifier("stable_b").unwrap()]),
        ),
        ModuleId::resolved(
            project,
            ModulePath::from_components(vec![ModuleComponent::from_identifier("stable_c").unwrap()]),
        ),
    )
}

fn layered_input(a_source: &str, b_source: &str, c_source: &str, generation: u64) -> SemanticWorkspaceInput {
    let (a, b, c) = layered_modules();
    let sources = [(a.clone(), a_source), (b.clone(), b_source), (c.clone(), c_source)]
        .into_iter()
        .map(|(module, source)| {
            let program = Arc::new(phalcom_ast::parse(source, 0).program);
            (
                module.clone(),
                Arc::new(ParsedModuleUnit::new(module, ModuleKind::Module, None, Arc::from(source), program)),
            )
        })
        .collect::<BTreeMap<_, _>>();
    let export = |module: &ModuleId, name: &str| {
        (
            name.into(),
            LinkedExport {
                public_name: name.into(),
                target: LinkedExportTarget::Binding(SymbolId {
                    module: module.clone(),
                    name: name.into(),
                }),
                range: phalcom_common::range::SourceRange::default(),
            },
        )
    };
    let linked_modules = BTreeMap::from([
        (
            a.clone(),
            LinkedModule {
                interface: LinkedModuleInterface {
                    module: a.clone(),
                    kind: ModuleKind::Module,
                    exports: BTreeMap::from([export(&a, "Api")]),
                    metadata: ModuleMetadata::default(),
                },
                bindings: ModuleBindingLayout {
                    local_globals: BTreeMap::from([("Api".into(), GlobalBindingId(0))]),
                    ..ModuleBindingLayout::default()
                },
                linked_reads: Vec::new(),
                runtime_dependencies: Vec::new(),
            },
        ),
        (
            b.clone(),
            LinkedModule {
                interface: LinkedModuleInterface {
                    module: b.clone(),
                    kind: ModuleKind::Module,
                    exports: BTreeMap::from([export(&b, "Middle")]),
                    metadata: ModuleMetadata::default(),
                },
                bindings: ModuleBindingLayout {
                    local_globals: BTreeMap::from([("Middle".into(), GlobalBindingId(0))]),
                    imports: BTreeMap::from([("Api".into(), ImportBindingId(0))]),
                },
                linked_reads: vec![LinkedReadSpec::Binding(SymbolId {
                    module: a.clone(),
                    name: "Api".into(),
                })],
                runtime_dependencies: vec![a.clone()],
            },
        ),
        (
            c.clone(),
            LinkedModule {
                interface: LinkedModuleInterface {
                    module: c.clone(),
                    kind: ModuleKind::Module,
                    exports: BTreeMap::new(),
                    metadata: ModuleMetadata::default(),
                },
                bindings: ModuleBindingLayout {
                    local_globals: BTreeMap::from([("Top".into(), GlobalBindingId(0))]),
                    imports: BTreeMap::from([("Middle".into(), ImportBindingId(0))]),
                },
                linked_reads: vec![LinkedReadSpec::Binding(SymbolId {
                    module: b.clone(),
                    name: "Middle".into(),
                })],
                runtime_dependencies: vec![b.clone()],
            },
        ),
    ]);
    SemanticWorkspaceInput::new(
        Arc::new(LinkedProgram {
            universe: Arc::new(phalcom_modules::project::ProjectUniverse::new()),
            modules: linked_modules,
            graphs: phalcom_modules::graph::ModuleGraphs::default(),
            entry: c.clone(),
            initialization_order: vec![a, b, c],
        }),
        sources,
        generation,
    )
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
    let consumer_body_key = QueryKey::CallableBody(consumer_callable.clone());
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
    let attachment = update2
        .snapshot
        .source_index()
        .formal_attachment(&consumer_callable)
        .expect("reused caller source attachment");
    assert!(
        attachment
            .expression_sites
            .iter()
            .any(|site| source2.get(site.range.start..site.range.end) == Some("Api.value()")),
        "reused semantic product must project current expression ranges"
    );
}

#[test]
fn previously_absent_name_addition_recomputes_exact_consumer() {
    let module = ModuleId::resolved(
        ResolvedProjectId::from_raw(8),
        ModulePath::from_components(vec![ModuleComponent::from_identifier("absent_name").unwrap()]),
    );
    let mut session = SemanticWorkspaceSession::new();
    let source_v1 = r#"
class Consumer {
  @class read() { Missing.value() }
}
"#;
    let update1 = session.update(input(module.clone(), source_v1, 1));
    let consumer = DeclarationId::new(module.clone(), "Consumer".into());
    let consumer_read = CallableId::new(consumer, Selector::method("read", []).unwrap(), DispatchSide::Class);
    let body_key = QueryKey::CallableBody(consumer_read.clone());
    let consumer_v1 = update1.snapshot.callable_analyses.get(&consumer_read).unwrap().clone();
    let rev1 = update1.snapshot.id.revision();
    assert!(session.db().index().dependencies_of(&body_key).is_some_and(|edges| {
        edges
            .iter()
            .any(|edge| edge.dependency == QueryKey::LinkedName(module.clone(), "Missing".into()))
    }));
    assert!(
        session
            .db()
            .index()
            .dependencies_of(&body_key)
            .is_some_and(|edges| { edges.iter().all(|edge| edge.dependency != QueryKey::LinkedInterface(module.clone())) })
    );

    let source_v2 = r#"
class Missing {
  @class value() -> Int { 1 }
}

class Consumer {
  @class read() { Missing.value() }
}
"#;
    let update2 = session.update(input(module.clone(), source_v2, 2));
    assert!(!update2.snapshot.has_errors());
    let rev2 = update2.snapshot.id.revision();
    let consumer_state = session.db().query_state(&body_key).unwrap();
    assert_eq!(consumer_state.revision(), Some(rev2));
    assert_ne!(consumer_state.revision(), Some(rev1));
    assert!(update2.recomputed.contains(&body_key));
    assert!(!Arc::ptr_eq(&consumer_v1, update2.snapshot.callable_analyses.get(&consumer_read).unwrap()));
}

#[test]
fn unrelated_edit_retains_unaffected_module_diagnostics() {
    use phalcom_semantic::diagnostic::DiagnosticCode;

    let module_a = ModuleId::resolved(
        ResolvedProjectId::from_raw(201),
        ModulePath::from_components(vec![ModuleComponent::from_identifier("diagnostic_a").unwrap()]),
    );
    let module_b = ModuleId::resolved(
        ResolvedProjectId::from_raw(201),
        ModulePath::from_components(vec![ModuleComponent::from_identifier("diagnostic_b").unwrap()]),
    );
    let source_a = r#"
class Broken {
  @class number() -> Int { "not an integer" }
}
"#;
    let source_b_v1 = "class Stable { @class value() -> Int { 1 } }";
    let source_b_v2 = "class Stable { @class value() -> Int { 2 } }";

    let mut session = SemanticWorkspaceSession::new();
    let initial = session.update(multi_module_input(
        vec![(module_a.clone(), source_a.into()), (module_b.clone(), source_b_v1.into())],
        1,
    ));
    let initial_a_diagnostics = initial
        .snapshot
        .diagnostics
        .get(&module_a)
        .cloned()
        .expect("module A must publish its semantic diagnostic");
    assert!(initial_a_diagnostics.iter().any(|diagnostic| diagnostic.code == DiagnosticCode::ReturnMismatch));

    let updated = session.update(multi_module_input(
        vec![(module_a.clone(), source_a.into()), (module_b.clone(), source_b_v2.into())],
        2,
    ));

    assert_eq!(updated.snapshot.diagnostics.get(&module_a), Some(&initial_a_diagnostics));
    assert!(!updated.effects.diagnostics_changed.contains(&module_a));

    let mut cold_session = SemanticWorkspaceSession::new();
    let cold = cold_session.update(multi_module_input(vec![(module_a.clone(), source_a.into()), (module_b, source_b_v2.into())], 2));
    assert_eq!(updated.snapshot.diagnostics, cold.snapshot.diagnostics);
}

#[test]
fn edited_module_repair_removes_its_diagnostic() {
    use phalcom_semantic::diagnostic::DiagnosticCode;

    let module = ModuleId::resolved(
        ResolvedProjectId::from_raw(202),
        ModulePath::from_components(vec![ModuleComponent::from_identifier("diagnostic_repair").unwrap()]),
    );
    let mut session = SemanticWorkspaceSession::new();
    let broken = "class Broken { @class number() -> Int { \"not an integer\" } }";
    let fixed = "class Broken { @class number() -> Int { 1 } }";

    let initial = session.update(input(module.clone(), broken, 1));
    assert!(
        initial
            .snapshot
            .diagnostics_for(&module)
            .is_some_and(|diagnostics| { diagnostics.iter().any(|diagnostic| diagnostic.code == DiagnosticCode::ReturnMismatch) })
    );

    let repaired = session.update(input(module.clone(), fixed, 2));
    assert!(
        repaired
            .snapshot
            .diagnostics_for(&module)
            .is_none_or(|diagnostics| { diagnostics.iter().all(|diagnostic| diagnostic.code != DiagnosticCode::ReturnMismatch) })
    );
    assert!(repaired.effects.diagnostics_changed.contains(&module));
    assert!(!repaired.snapshot.has_errors());
}

#[test]
fn provider_semantic_change_makes_consumer_diagnostic_appear() {
    use phalcom_semantic::diagnostic::DiagnosticCode;

    let module = ModuleId::resolved(
        ResolvedProjectId::from_raw(203),
        ModulePath::from_components(vec![ModuleComponent::from_identifier("provider_change").unwrap()]),
    );
    let mut session = SemanticWorkspaceSession::new();
    let source_v1 = r#"
class Provider {
  @class value() -> Int { 1 }
}

class Consumer {
  @class read() -> Int { Provider.value() }
}
"#;
    let source_v2 = r#"
class Provider {
  @class value() -> String { "one" }
}

class Consumer {
  @class read() -> Int { Provider.value() }
}
"#;

    let initial = session.update(input(module.clone(), source_v1, 1));
    assert!(!initial.snapshot.has_errors());
    let updated = session.update(input(module.clone(), source_v2, 2));
    assert!(
        updated
            .snapshot
            .diagnostics_for(&module)
            .is_some_and(|diagnostics| { diagnostics.iter().any(|diagnostic| diagnostic.code == DiagnosticCode::ReturnMismatch) })
    );
}

#[test]
fn provider_repair_removes_consumer_diagnostic() {
    use phalcom_semantic::diagnostic::DiagnosticCode;

    let module = ModuleId::resolved(
        ResolvedProjectId::from_raw(204),
        ModulePath::from_components(vec![ModuleComponent::from_identifier("provider_repair").unwrap()]),
    );
    let mut session = SemanticWorkspaceSession::new();
    let broken = r#"
class Provider {
  @class value() -> String { "one" }
}

class Consumer {
  @class read() -> Int { Provider.value() }
}
"#;
    let fixed = r#"
class Provider {
  @class value() -> Int { 1 }
}

class Consumer {
  @class read() -> Int { Provider.value() }
}
"#;

    let initial = session.update(input(module.clone(), broken, 1));
    assert!(
        initial
            .snapshot
            .diagnostics_for(&module)
            .is_some_and(|diagnostics| { diagnostics.iter().any(|diagnostic| diagnostic.code == DiagnosticCode::ReturnMismatch) })
    );
    let repaired = session.update(input(module.clone(), fixed, 2));
    assert!(
        repaired
            .snapshot
            .diagnostics_for(&module)
            .is_none_or(|diagnostics| { diagnostics.iter().all(|diagnostic| diagnostic.code != DiagnosticCode::ReturnMismatch) })
    );
    assert!(!repaired.snapshot.has_errors());
}

#[test]
fn removed_module_removes_all_diagnostics() {
    use phalcom_semantic::diagnostic::DiagnosticCode;

    let module_a = ModuleId::resolved(
        ResolvedProjectId::from_raw(205),
        ModulePath::from_components(vec![ModuleComponent::from_identifier("removed_diagnostics").unwrap()]),
    );
    let module_b = ModuleId::resolved(
        ResolvedProjectId::from_raw(205),
        ModulePath::from_components(vec![ModuleComponent::from_identifier("survivor").unwrap()]),
    );
    let mut session = SemanticWorkspaceSession::new();
    let initial = session.update(multi_module_input(
        vec![
            (module_a.clone(), "class Broken { @class number() -> Int { \"bad\" } }".into()),
            (module_b.clone(), "class Stable { @class value() -> Int { 1 } }".into()),
        ],
        1,
    ));
    assert!(
        initial
            .snapshot
            .diagnostics_for(&module_a)
            .is_some_and(|diagnostics| { diagnostics.iter().any(|diagnostic| diagnostic.code == DiagnosticCode::ReturnMismatch) })
    );

    let updated = session.update(multi_module_input(
        vec![(module_b.clone(), "class Stable { @class value() -> Int { 2 } }".into())],
        2,
    ));
    assert!(updated.snapshot.diagnostics_for(&module_a).is_none());
    assert!(!updated.snapshot.has_errors());
}

#[test]
fn diagnostic_aggregate_matches_cold_after_repair_and_unrelated_edit() {
    let module_a = ModuleId::resolved(
        ResolvedProjectId::from_raw(206),
        ModulePath::from_components(vec![ModuleComponent::from_identifier("cold_diagnostics_a").unwrap()]),
    );
    let module_b = ModuleId::resolved(
        ResolvedProjectId::from_raw(206),
        ModulePath::from_components(vec![ModuleComponent::from_identifier("cold_diagnostics_b").unwrap()]),
    );
    let source_a = "class Broken { @class number() -> Int { \"bad\" } }";
    let source_b_v1 = "class Stable { @class value() -> Int { 1 } }";
    let source_b_v2 = "class Stable { @class value() -> Int { 2 } }";

    let mut session = SemanticWorkspaceSession::new();
    let _ = session.update(multi_module_input(
        vec![(module_a.clone(), source_a.into()), (module_b.clone(), source_b_v1.into())],
        1,
    ));
    let incremental = session.update(multi_module_input(
        vec![(module_a.clone(), source_a.into()), (module_b.clone(), source_b_v2.into())],
        2,
    ));

    let mut cold_session = SemanticWorkspaceSession::new();
    let cold = cold_session.update(multi_module_input(vec![(module_a, source_a.into()), (module_b, source_b_v2.into())], 2));
    assert_eq!(incremental.snapshot.diagnostics, cold.snapshot.diagnostics);
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
fn stable_intermediate_callable_product_stops_later_propagation() {
    let module = ModuleId::resolved(
        ResolvedProjectId::from_raw(55),
        ModulePath::from_components(vec![ModuleComponent::from_identifier("stable_intermediate").unwrap()]),
    );
    let mut session = SemanticWorkspaceSession::new();
    let source_a = r#"
class Api {
  @class value() -> Int { 1 }
}
class Consumer {
  @class read() { Api.value() }
}
"#;
    let source_b = r#"
class Api {
  @class value() -> String { "changed" }
}
class Consumer {
  @class read() { Api.value() }
}
"#;
    let source_c = r#"
class Api {
  @class value() -> String { "changed again" }
}
class Consumer {
  @class read() { Api.value() }
}
"#;

    let update_a = session.update(input(module.clone(), source_a, 1));
    let consumer = DeclarationId::new(module.clone(), "Consumer".into());
    let consumer_read = CallableId::new(consumer, Selector::method("read", []).unwrap(), DispatchSide::Class);
    let body_key = QueryKey::CallableBody(consumer_read.clone());
    let api = DeclarationId::new(module.clone(), "Api".into());
    let api_value = CallableId::new(api, Selector::method("value", []).unwrap(), DispatchSide::Class);

    let update_b = session.update(input(module.clone(), source_b, 2));
    assert!(
        update_b.recomputed.contains(&body_key),
        "the changed callable contract must recompute its exact consumer"
    );
    let consumer_revision_b = session
        .db()
        .query_state(&body_key)
        .and_then(|state| state.revision())
        .expect("consumer revision after B");
    let api_body_revision_b = session
        .db()
        .query_state(&QueryKey::CallableBody(api_value.clone()))
        .and_then(|state| state.revision())
        .expect("API body revision after B");

    let update_c = session.update(input(module, source_c, 3));
    assert!(
        !update_c.recomputed.contains(&body_key),
        "a body-only edit with a stable signature must stop at the provider"
    );
    assert_eq!(
        session.db().query_state(&body_key).and_then(|state| state.revision()),
        Some(consumer_revision_b)
    );
    assert!(session.db().query_state(&body_key).is_some_and(|state| state.validated_revision().is_some()));
    assert_ne!(
        session.db().query_state(&QueryKey::CallableBody(api_value)).and_then(|state| state.revision()),
        Some(api_body_revision_b),
        "the provider body itself still recomputes"
    );
    assert!(!update_a.snapshot.has_errors());
    assert!(!update_b.snapshot.has_errors());
    assert!(!update_c.snapshot.has_errors());
}

#[test]
fn pa8_recomputed_stable_intermediate_allows_downstream_reuse() {
    let (_a, b, c) = layered_modules();
    let source_a_v1 = "class Api { @class value(_ first: Int) -> Int { 1 } }\nexport Api\n";
    let source_a_v2 = "class Api { @class value(_ second: Object) -> Int { 2 } }\nexport Api\n";
    let source_b = "import stable_a.Api\nclass Middle { @class value() -> Int { Api.value(0) } }\nexport Middle\n";
    let source_c = "import stable_b.Middle\nclass Top { @class value() -> Int { Middle.value() } }\n";
    let mut session = SemanticWorkspaceSession::new();

    let first = session.update(layered_input(source_a_v1, source_b, source_c, 1));
    assert!(!first.snapshot.has_errors(), "initial diagnostics: {:?}", first.snapshot.diagnostics);
    let middle = CallableId::new(
        DeclarationId::new(b.clone(), "Middle".into()),
        Selector::method("value", []).unwrap(),
        DispatchSide::Class,
    );
    let top = CallableId::new(
        DeclarationId::new(c.clone(), "Top".into()),
        Selector::method("value", []).unwrap(),
        DispatchSide::Class,
    );
    let middle_key = QueryKey::CallableBody(middle.clone());
    let top_key = QueryKey::CallableBody(top.clone());
    let top_v1 = first.snapshot.callable_analyses.get(&top).cloned().expect("Top v1");
    let middle_product_v1 = session.db().ready_product_fingerprint(&middle_key).expect("Middle product v1");
    let top_revision_v1 = session.db().query_state(&top_key).and_then(|state| state.revision()).expect("Top revision v1");

    let second = session.update(layered_input(source_a_v2, source_b, source_c, 2));
    assert!(!second.snapshot.has_errors(), "updated diagnostics: {:?}", second.snapshot.diagnostics);
    assert!(second.recomputed.contains(&middle_key), "A's contract refresh must recompute B");
    assert!(!second.recomputed.contains(&top_key), "C must stop at B's stable product");
    let middle_state = session.db().query_state(&middle_key).expect("Middle state v2");
    assert_eq!(
        middle_state.revision(),
        Some(second.snapshot.id.revision()),
        "B must actually recompute in the new revision"
    );
    assert_eq!(middle_state.validated_revision(), Some(second.snapshot.id.revision()));
    assert_eq!(
        session.db().ready_product_fingerprint(&middle_key),
        Some(middle_product_v1),
        "B must republish the same product fingerprint"
    );

    let top_state = session.db().query_state(&top_key).expect("Top state v2");
    assert_eq!(top_state.revision(), Some(top_revision_v1), "C computation must remain at v1");
    assert_eq!(
        top_state.validated_revision(),
        Some(second.snapshot.id.revision()),
        "C must validate against B's republished product"
    );
    assert!(Arc::ptr_eq(&top_v1, second.snapshot.callable_analyses.get(&top).expect("Top v2")));
    assert!(second.stats.callables_recomputed >= 1);
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
