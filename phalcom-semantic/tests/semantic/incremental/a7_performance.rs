//! A7 semantic work-count publication and cold/incremental metric evidence.

use super::support::multi_module_input;
use phalcom_common::selector::Selector;
use phalcom_modules::graph::{SemanticEdgeKind, SemanticNodeId};
use phalcom_modules::identity::{ModuleComponent, ModuleId, ModulePath, ResolvedProjectId};
use phalcom_modules::interface::{LinkedExport, LinkedExportTarget, LinkedModuleInterface};
use phalcom_modules::linker::{GlobalBindingId, ImportBindingId, LinkedModule, LinkedProgram, LinkedReadSpec, ModuleBindingLayout, SymbolId};
use phalcom_modules::metadata::ModuleMetadata;
use phalcom_modules::source::ModuleKind;
use phalcom_modules::{SourceId, SourceLocation, SourceRevision, WorkspaceSourceBatchMutation};
use phalcom_semantic::checker::analysis::{AnalysisStatus, CallableAnalysisStatus};
use phalcom_semantic::checker::causal::CausalInvalidity;
use phalcom_semantic::db::{CancellationToken, QueryBudget, QueryKey};
use phalcom_semantic::declaration_type::DeclaredTypeState;
use phalcom_semantic::diagnostic::{DiagnosticGuidance, SemanticDiagnostic};
use phalcom_semantic::identity::{CallableId, DeclarationId, DispatchSide};
use phalcom_semantic::session::SemanticWorkspaceSession;
use phalcom_semantic::snapshot::SemanticSnapshot;
use phalcom_semantic::source::ParsedModuleUnit;
use phalcom_semantic::surface::MemberSurface;
use phalcom_semantic::types::evidence::TypeKnowledge;
use phalcom_semantic::types::parameter::{GenericSignature, TypeTerm};
use phalcom_semantic::workspace::SemanticWorkspaceInput;
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::Arc;

fn module(name: &str) -> ModuleId {
    ModuleId::resolved(
        ResolvedProjectId::from_raw(707),
        ModulePath::from_components(vec![ModuleComponent::from_identifier(name).unwrap()]),
    )
}

#[test]
fn a7_semantic_stats_separate_recomputation_validation_and_dependents() {
    let first = module("first");
    let second = module("second");
    let mut session = SemanticWorkspaceSession::new();

    let initial = session.update(multi_module_input(
        vec![
            (first.clone(), "class First { value() -> Int { 1 } }".into()),
            (second.clone(), "class Second { value() -> Int { 2 } }".into()),
        ],
        1,
    ));
    assert!(initial.stats.query_products_recomputed > 0);
    assert_eq!(initial.stats.query_products_revalidated, 0);
    assert_eq!(initial.stats.semantic_structure_shards_recomputed, 2);

    let update = session.update(multi_module_input(
        vec![
            (first, "class First { value() -> Int { 3 } }".into()),
            (second, "class Second { value() -> Int { 2 } }".into()),
        ],
        2,
    ));
    assert!(update.stats.query_products_recomputed > 0);
    assert!(update.stats.query_products_revalidated > 0);
    assert_eq!(update.stats.semantic_structure_shards_recomputed, 1);
    assert_eq!(update.stats.semantic_structure_shards_reused, 1);
    assert!(update.stats.semantic_dependents_recomputed + update.stats.semantic_dependents_reused > 0);
}

#[test]
fn a6_retained_superclass_edge_survives_unrelated_structural_edit() {
    let owner = module("hierarchy_owner");
    let unrelated = module("hierarchy_unrelated");
    let mut session = SemanticWorkspaceSession::new();

    let initial = session.update(multi_module_input(
        vec![
            (owner.clone(), "class Base {}\nclass Child is Base {}".into()),
            (unrelated.clone(), "class Unrelated {}".into()),
        ],
        1,
    ));
    let child = SemanticNodeId::Declaration {
        module: owner.clone(),
        name: "Child".into(),
    };
    let base = SemanticNodeId::Declaration {
        module: owner.clone(),
        name: "Base".into(),
    };
    assert!(
        initial
            .snapshot
            .semantic_graph
            .edges_from(&child)
            .iter()
            .any(|edge| { edge.kind == SemanticEdgeKind::Superclass && edge.to == base })
    );

    let updated = session.update(multi_module_input(
        vec![
            (owner, "class Base {}\nclass Child is Base {}".into()),
            (unrelated, "class Unrelated {}\nclass Added {}".into()),
        ],
        2,
    ));

    assert!(
        updated
            .snapshot
            .semantic_graph
            .edges_from(&child)
            .iter()
            .any(|edge| { edge.kind == SemanticEdgeKind::Superclass && edge.to == base })
    );
}

#[test]
fn a6_implicit_object_hierarchy_edge_is_published_for_new_declarations() {
    let owner = module("implicit_hierarchy");
    let mut session = SemanticWorkspaceSession::new();
    let initial = session.update(multi_module_input(vec![(owner.clone(), "class A {}".into())], 1));
    let a = DeclarationId::new(owner.clone(), "A".into());
    let extra = DeclarationId::new(owner.clone(), "Extra".into());
    let object = phalcom_semantic::core_surface::universe_declaration(phalcom_native_meta::UniverseKey::Object);

    assert_eq!(initial.snapshot.hierarchy.superclasses.get(&a), Some(&object));
    assert!(initial.snapshot.surfaces().contains_key(&a));

    let updated = session.update(multi_module_input(vec![(owner.clone(), "class A {}\nclass Extra {}".into())], 2));
    assert_eq!(updated.snapshot.hierarchy.superclasses.get(&extra), Some(&object));
    assert!(updated.snapshot.surfaces().contains_key(&extra));
    let extra_edges = updated.snapshot.semantic_graph.edges_from(&SemanticNodeId::Declaration {
        module: owner.clone(),
        name: "Extra".into(),
    });
    assert!(
        extra_edges.iter().any(|edge| edge.kind == SemanticEdgeKind::Superclass
            && edge.to
                == SemanticNodeId::Declaration {
                    module: object.module.clone(),
                    name: object.name.clone(),
                }),
        "Extra hierarchy edges: {extra_edges:?}"
    );
    assert_eq!(
        session
            .db()
            .query_state(&QueryKey::HierarchyEdge(extra.clone()))
            .and_then(|state| state.validated_revision()),
        Some(session.db().revision())
    );

    let mut cold = SemanticWorkspaceSession::new();
    let cold_update = cold.update(multi_module_input(vec![(owner, "class A {}\nclass Extra {}".into())], 2));
    assert_eq!(updated.snapshot.hierarchy.superclasses, cold_update.snapshot.hierarchy.superclasses);
    assert_eq!(updated.snapshot.surfaces().len(), cold_update.snapshot.surfaces().len());
    assert!(
        cold_update
            .snapshot
            .surfaces()
            .keys()
            .all(|declaration| updated.snapshot.surfaces().contains_key(declaration))
    );
    assert_eq!(updated.snapshot.semantic_graph, cold_update.snapshot.semantic_graph);
}

#[test]
fn a6_body_edit_in_hierarchy_workspace_retains_graph_and_edge_product() {
    let owner = module("hierarchy_body_owner");
    let unrelated = module("hierarchy_body_unrelated");
    let mut session = SemanticWorkspaceSession::new();

    let initial = session.update(multi_module_input(
        vec![
            (owner.clone(), "class Base {}\nclass Child is Base {}".into()),
            (unrelated.clone(), "class Unrelated { value() -> Int { 1 } }".into()),
        ],
        1,
    ));
    let child = SemanticNodeId::Declaration {
        module: owner,
        name: "Child".into(),
    };
    let hierarchy_revision = session
        .db()
        .query_state(&QueryKey::HierarchyEdge(phalcom_semantic::identity::DeclarationId::new(
            child_module(&child),
            "Child".into(),
        )))
        .and_then(|state| state.revision())
        .expect("initial hierarchy edge product");

    let updated = session.update(multi_module_input(
        vec![
            (module("hierarchy_body_owner"), "class Base {}\nclass Child is Base {}".into()),
            (unrelated, "class Unrelated { value() -> Int { 2 } }".into()),
        ],
        2,
    ));

    assert_eq!(updated.snapshot.semantic_graph, initial.snapshot.semantic_graph);
    assert!(!updated.effects.module_graph_changed);
    assert_eq!(
        session
            .db()
            .query_state(&QueryKey::HierarchyEdge(phalcom_semantic::identity::DeclarationId::new(
                child_module(&child),
                "Child".into(),
            )))
            .and_then(|state| state.revision()),
        Some(hierarchy_revision)
    );
}

#[test]
fn a6_incremental_cross_module_inheritance_cycle_matches_cold_analysis() {
    let mut session = SemanticWorkspaceSession::new();
    let initial = session.update(cross_module_inheritance_input(false, 1));
    assert!(!initial.snapshot.has_errors(), "initial diagnostics: {:?}", initial.snapshot.diagnostics);
    let hierarchy_dependencies = session
        .db()
        .index()
        .dependencies_of(&QueryKey::HierarchyEdge(phalcom_semantic::identity::DeclarationId::new(
            module("a"),
            "A".into(),
        )))
        .expect("cross-module hierarchy dependencies");
    assert!(
        hierarchy_dependencies
            .iter()
            .any(|edge| edge.dependency == QueryKey::LinkedName(module("a"), "B".into()))
    );
    assert!(
        hierarchy_dependencies
            .iter()
            .any(|edge| { edge.dependency == QueryKey::DeclarationShell(phalcom_semantic::identity::DeclarationId::new(module("b"), "B".into())) }),
        "dependencies: {hierarchy_dependencies:?}"
    );

    let updated = session.update(cross_module_inheritance_input(true, 2));
    assert!(updated.snapshot.has_errors(), "incremental cycle must be diagnosed");

    let mut cold_session = SemanticWorkspaceSession::new();
    let cold = cold_session.update(cross_module_inheritance_input(true, 1));
    assert!(cold.snapshot.has_errors(), "cold cycle must be diagnosed");
    let incremental_codes = updated.snapshot.all_diagnostics().map(|diagnostic| diagnostic.code).collect::<Vec<_>>();
    let cold_codes = cold.snapshot.all_diagnostics().map(|diagnostic| diagnostic.code).collect::<Vec<_>>();
    assert_eq!(incremental_codes, cold_codes);
}

fn cross_module_inheritance_input(b_cycle: bool, generation: u64) -> SemanticWorkspaceInput {
    let a = module("a");
    let b = module("b");
    let a_source: Arc<str> = Arc::from("import b.B\nclass A is B {}\nexport A\n");
    let b_source: Arc<str> = if b_cycle {
        Arc::from("import a.A\nclass B is A {}\nexport B\n")
    } else {
        Arc::from("class B {}\nexport B\n")
    };
    let mut sources = BTreeMap::new();
    for (module_id, source) in [(a.clone(), a_source), (b.clone(), b_source)] {
        let program = Arc::new(phalcom_ast::parse(&source, 0).program);
        sources.insert(
            module_id.clone(),
            Arc::new(ParsedModuleUnit::new(module_id, ModuleKind::Module, None, source, program)),
        );
    }

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
    let a_module = LinkedModule {
        interface: LinkedModuleInterface {
            module: a.clone(),
            kind: ModuleKind::Module,
            exports: BTreeMap::from([export(&a, "A")]),
            metadata: ModuleMetadata::default(),
        },
        bindings: ModuleBindingLayout {
            local_globals: BTreeMap::from([("A".into(), phalcom_modules::linker::GlobalBindingId(0))]),
            imports: BTreeMap::from([("B".into(), ImportBindingId(0))]),
        },
        linked_reads: vec![LinkedReadSpec::Binding(SymbolId {
            module: b.clone(),
            name: "B".into(),
        })],
        runtime_dependencies: vec![b.clone()],
    };
    let b_module = LinkedModule {
        interface: LinkedModuleInterface {
            module: b.clone(),
            kind: ModuleKind::Module,
            exports: BTreeMap::from([export(&b, "B")]),
            metadata: ModuleMetadata::default(),
        },
        bindings: ModuleBindingLayout {
            local_globals: BTreeMap::from([("B".into(), phalcom_modules::linker::GlobalBindingId(0))]),
            imports: if b_cycle {
                BTreeMap::from([("A".into(), ImportBindingId(0))])
            } else {
                BTreeMap::new()
            },
        },
        linked_reads: if b_cycle {
            vec![LinkedReadSpec::Binding(SymbolId {
                module: a.clone(),
                name: "A".into(),
            })]
        } else {
            Vec::new()
        },
        runtime_dependencies: if b_cycle { vec![a.clone()] } else { Vec::new() },
    };
    let linked = Arc::new(LinkedProgram {
        universe: Arc::new(phalcom_modules::project::ProjectUniverse::new()),
        modules: BTreeMap::from([(a.clone(), a_module), (b.clone(), b_module)]),
        graphs: phalcom_modules::graph::ModuleGraphs::default(),
        entry: a.clone(),
        initialization_order: vec![a, b],
    });
    SemanticWorkspaceInput::new(linked, sources, generation)
}

#[test]
fn a7_unused_public_export_does_not_recompute_unrelated_consumer_body() {
    let mut session = SemanticWorkspaceSession::new();
    let initial = session.update(unused_public_export_input(false, 1));
    assert!(!initial.snapshot.has_errors(), "initial diagnostics: {:?}", initial.snapshot.diagnostics);

    let consumer = module("unused_consumer");
    let callable = CallableId::new(
        DeclarationId::new(consumer, "Consumer".into()),
        Selector::method("read", []).unwrap(),
        DispatchSide::Class,
    );
    let body_key = QueryKey::CallableBody(callable.clone());
    let initial_revision = session
        .db()
        .query_state(&body_key)
        .and_then(|state| state.revision())
        .expect("consumer body product");
    let initial_validated_revision = session
        .db()
        .query_state(&body_key)
        .and_then(|state| state.validated_revision())
        .expect("consumer body validation revision");
    let initial_analysis = initial.snapshot.callable_analyses.get(&callable).expect("consumer body analysis").clone();

    let updated = session.update(unused_public_export_input(true, 2));
    assert!(!updated.snapshot.has_errors(), "updated diagnostics: {:?}", updated.snapshot.diagnostics);
    let state = session.db().query_state(&body_key).expect("consumer body state");
    assert_eq!(
        state.revision(),
        Some(initial_revision),
        "unused provider export must not recompute consumer body"
    );
    assert_ne!(
        state.validated_revision(),
        Some(initial_validated_revision),
        "retained consumer body must validate in the new revision"
    );
    assert!(!updated.recomputed.contains(&body_key));
    assert!(Arc::ptr_eq(
        &initial_analysis,
        updated.snapshot.callable_analyses.get(&callable).expect("retained consumer analysis")
    ));
    assert!(session.db().index().dependencies_of(&body_key).is_some_and(|edges| {
        edges
            .iter()
            .any(|edge| edge.dependency == QueryKey::LinkedName(module("unused_consumer"), "Used".into()))
            && edges
                .iter()
                .all(|edge| edge.dependency != QueryKey::LinkedName(module("unused_consumer"), "Unused".into()))
            && edges
                .iter()
                .all(|edge| edge.dependency != QueryKey::PublicExport(module("unused_provider"), "Unused".into()))
    }));
}

#[test]
fn a6_cross_module_alias_relowers_when_provider_shell_changes() {
    let mut session = SemanticWorkspaceSession::new();
    let initial = session.update(cross_module_alias_input("type Number = Int\n", 1));
    assert!(!initial.snapshot.has_errors(), "initial diagnostics: {:?}", initial.snapshot.diagnostics);

    let consumer = module("alias_consumer");
    let alias = DeclarationId::new(consumer.clone(), "Local".into());
    let alias_key = QueryKey::DeclarationShell(alias.clone());
    let initial_form = initial.snapshot.type_aliases.form(&alias).expect("initial consumer alias");
    let initial_shape = initial.snapshot.store.format_type(initial_form);
    let initial_revision = session
        .db()
        .query_state(&alias_key)
        .and_then(|state| state.revision())
        .expect("initial alias shell");

    let updated = session.update(cross_module_alias_input("type Number = String\n", 2));
    assert!(!updated.snapshot.has_errors(), "updated diagnostics: {:?}", updated.snapshot.diagnostics);
    let provider_alias = DeclarationId::new(module("alias_provider"), "Number".into());
    let provider_form = updated.snapshot.type_aliases.form(&provider_alias).expect("updated provider alias");
    assert_eq!(updated.snapshot.store.format_type(provider_form), "String");
    let updated_form = updated.snapshot.type_aliases.form(&alias).expect("updated consumer alias");
    assert_ne!(initial_shape, updated.snapshot.store.format_type(updated_form));
    assert_eq!(updated.snapshot.store.format_type(updated_form), "String");
    assert_ne!(
        session.db().query_state(&alias_key).and_then(|state| state.revision()),
        Some(initial_revision),
        "a retained consumer alias must be lowered again when its provider alias shell changes"
    );
}

fn cross_module_alias_input(provider_source: &str, generation: u64) -> SemanticWorkspaceInput {
    let provider = module("alias_provider");
    let consumer = module("alias_consumer");
    let consumer_source: Arc<str> =
        Arc::from("import alias_provider.Number\ntype Local = Number\nclass Consumer { @class use(_ value: Local) -> Int { 1 } }\n");
    let provider_source: Arc<str> = Arc::from(provider_source.to_owned());
    let mut sources = BTreeMap::new();
    for (module_id, source) in [(provider.clone(), provider_source), (consumer.clone(), consumer_source)] {
        let program = Arc::new(phalcom_ast::parse(&source, 0).program);
        sources.insert(
            module_id.clone(),
            Arc::new(ParsedModuleUnit::new(module_id, ModuleKind::Module, None, source, program)),
        );
    }
    let provider_export = LinkedExport {
        public_name: "Number".into(),
        target: LinkedExportTarget::Binding(SymbolId {
            module: provider.clone(),
            name: "Number".into(),
        }),
        range: phalcom_common::range::SourceRange::default(),
    };
    let linked = Arc::new(LinkedProgram {
        universe: Arc::new(phalcom_modules::project::ProjectUniverse::new()),
        modules: BTreeMap::from([
            (
                provider.clone(),
                LinkedModule {
                    interface: LinkedModuleInterface {
                        module: provider.clone(),
                        kind: ModuleKind::Module,
                        exports: BTreeMap::from([("Number".into(), provider_export)]),
                        metadata: ModuleMetadata::default(),
                    },
                    bindings: ModuleBindingLayout {
                        local_globals: BTreeMap::from([("Number".into(), phalcom_modules::linker::GlobalBindingId(0))]),
                        imports: BTreeMap::new(),
                    },
                    linked_reads: Vec::new(),
                    runtime_dependencies: Vec::new(),
                },
            ),
            (
                consumer.clone(),
                LinkedModule {
                    interface: LinkedModuleInterface {
                        module: consumer.clone(),
                        kind: ModuleKind::Module,
                        exports: BTreeMap::new(),
                        metadata: ModuleMetadata::default(),
                    },
                    bindings: ModuleBindingLayout {
                        local_globals: BTreeMap::from([("Consumer".into(), phalcom_modules::linker::GlobalBindingId(0))]),
                        imports: BTreeMap::from([("Number".into(), ImportBindingId(0))]),
                    },
                    linked_reads: vec![LinkedReadSpec::Binding(SymbolId {
                        module: provider.clone(),
                        name: "Number".into(),
                    })],
                    runtime_dependencies: vec![provider.clone()],
                },
            ),
        ]),
        graphs: phalcom_modules::graph::ModuleGraphs::default(),
        entry: consumer,
        initialization_order: vec![provider, module("alias_consumer")],
    });
    SemanticWorkspaceInput::new(linked, sources, generation)
}

fn unused_public_export_input(extra_export: bool, generation: u64) -> SemanticWorkspaceInput {
    let provider = module("unused_provider");
    let consumer = module("unused_consumer");
    let provider_source: Arc<str> = if extra_export {
        Arc::from("class Used { @class value() -> Int { 1 } }\nclass Unused {}\nexport Used\nexport Unused\n")
    } else {
        Arc::from("class Used { @class value() -> Int { 1 } }\nclass Unused {}\nexport Used\n")
    };
    let consumer_source: Arc<str> = Arc::from("import unused_provider.Used\nclass Consumer { @class read() -> Int { Used.value() } }\n");
    let mut sources = BTreeMap::new();
    for (module_id, source) in [(provider.clone(), provider_source), (consumer.clone(), consumer_source)] {
        let program = Arc::new(phalcom_ast::parse(&source, 0).program);
        sources.insert(
            module_id.clone(),
            Arc::new(ParsedModuleUnit::new(module_id, ModuleKind::Module, None, source, program)),
        );
    }
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
    let provider_exports = if extra_export {
        BTreeMap::from([export(&provider, "Used"), export(&provider, "Unused")])
    } else {
        BTreeMap::from([export(&provider, "Used")])
    };
    let provider_module = LinkedModule {
        interface: LinkedModuleInterface {
            module: provider.clone(),
            kind: ModuleKind::Module,
            exports: provider_exports,
            metadata: ModuleMetadata::default(),
        },
        bindings: ModuleBindingLayout {
            local_globals: BTreeMap::from([
                ("Used".into(), phalcom_modules::linker::GlobalBindingId(0)),
                ("Unused".into(), phalcom_modules::linker::GlobalBindingId(1)),
            ]),
            imports: BTreeMap::new(),
        },
        linked_reads: Vec::new(),
        runtime_dependencies: Vec::new(),
    };
    let consumer_module = LinkedModule {
        interface: LinkedModuleInterface {
            module: consumer.clone(),
            kind: ModuleKind::Module,
            exports: BTreeMap::new(),
            metadata: ModuleMetadata::default(),
        },
        bindings: ModuleBindingLayout {
            local_globals: BTreeMap::from([("Consumer".into(), phalcom_modules::linker::GlobalBindingId(0))]),
            imports: BTreeMap::from([("Used".into(), ImportBindingId(0))]),
        },
        linked_reads: vec![LinkedReadSpec::Binding(SymbolId {
            module: provider.clone(),
            name: "Used".into(),
        })],
        runtime_dependencies: vec![provider.clone()],
    };
    let linked = Arc::new(LinkedProgram {
        universe: Arc::new(phalcom_modules::project::ProjectUniverse::new()),
        modules: BTreeMap::from([(provider.clone(), provider_module), (consumer.clone(), consumer_module)]),
        graphs: phalcom_modules::graph::ModuleGraphs::default(),
        entry: consumer.clone(),
        initialization_order: vec![provider, consumer],
    });
    SemanticWorkspaceInput::new(linked, sources, generation)
}

fn child_module(node: &SemanticNodeId) -> phalcom_modules::identity::ModuleId {
    match node {
        SemanticNodeId::Declaration { module, .. } => module.clone(),
        SemanticNodeId::Module(module) => module.clone(),
    }
}

#[test]
fn a6_high_fanout_recomputes_exact_consumers_not_reverse_importers() {
    let mut session = SemanticWorkspaceSession::new();
    let initial = session.update(high_fanout_input("class Provider { @class value() -> Int { 1 } }\nexport Provider\n", 1));
    assert!(!initial.snapshot.has_errors(), "initial diagnostics: {:?}", initial.snapshot.diagnostics);

    let updated = session.update(high_fanout_input(
        "class Provider { @class value() -> String { \"changed\" } }\nexport Provider\n",
        2,
    ));
    assert!(updated.snapshot.has_errors(), "the exact consumers should now reject the changed return type");

    // The bound is derived from the fixture: one provider signature root,
    // 100 consumer body products, up to two exact semantic products per
    // consumer, and a fixed provider/root allowance. The other 4,900
    // importers are reverse-connected only at the module layer and must not
    // enter semantic work.
    assert!(
        updated.stats.reverse_candidates_considered <= 350,
        "exact semantic closure exceeded the 100-consumer bound: {:?}",
        updated.stats
    );
    assert_eq!(
        updated.stats.callable_bodies_recomputed, 101,
        "provider plus the 100 exact consumers should recompute"
    );
    assert!(
        updated.stats.semantic_dependents_recomputed <= 350,
        "semantic recomputation must remain proportional to exact consumers: {:?}",
        updated.stats
    );
    assert!(
        updated.stats.exact_name_products_recomputed <= 10,
        "name products must remain bounded: {:?}",
        updated.stats
    );
}

#[test]
fn a6_high_fanout_unused_provider_body_edit_has_no_external_semantic_recompute() {
    let mut session = SemanticWorkspaceSession::new();
    let initial = session.update(high_fanout_input("class Provider { @class value() -> Int { 1 } }\nexport Provider\n", 1));
    assert!(!initial.snapshot.has_errors(), "initial diagnostics: {:?}", initial.snapshot.diagnostics);

    let updated = session.update(high_fanout_input("class Provider { @class value() -> Int { 2 } }\nexport Provider\n", 2));
    assert!(
        !updated.snapshot.has_errors(),
        "body-only provider edit should remain valid: {:?}",
        updated.snapshot.diagnostics
    );
    assert_eq!(updated.stats.callable_bodies_recomputed, 1, "only the provider body should recompute");
    assert!(
        updated.stats.reverse_candidates_considered <= 20,
        "unused provider body must not traverse reverse importers: {:?}",
        updated.stats
    );
}

#[test]
fn a6_high_fanout_unrelated_declaration_and_hierarchy_retain_existing_products() {
    let mut session = SemanticWorkspaceSession::new();
    let initial = session.update(high_fanout_input("class Provider { @class value() -> Int { 1 } }\nexport Provider\n", 1));
    assert!(!initial.snapshot.has_errors(), "initial diagnostics: {:?}", initial.snapshot.diagnostics);

    let provider = module("fanout_provider");
    let provider_callable = CallableId::new(
        DeclarationId::new(provider.clone(), "Provider".into()),
        Selector::method("value", []).unwrap(),
        DispatchSide::Class,
    );
    let consumer = module("fanout_consumer0000");
    let consumer_callable = CallableId::new(
        DeclarationId::new(consumer, "Consumer0000".into()),
        Selector::method("read", []).unwrap(),
        DispatchSide::Class,
    );
    let provider_signature_key = QueryKey::CallableSignature(provider_callable);
    let consumer_body_key = QueryKey::CallableBody(consumer_callable);
    let provider_hierarchy_key = QueryKey::HierarchyEdge(DeclarationId::new(provider.clone(), "Provider".into()));
    let signature_revision = session.db().query_state(&provider_signature_key).and_then(|state| state.revision());
    let consumer_body_revision = session.db().query_state(&consumer_body_key).and_then(|state| state.revision());
    let hierarchy_revision = session.db().query_state(&provider_hierarchy_key).and_then(|state| state.revision());

    let updated = session.update(high_fanout_input(
        "class Provider { @class value() -> Int { 1 } }\nclass Extra is Provider {}\nexport Provider\n",
        2,
    ));
    assert!(
        !updated.snapshot.has_errors(),
        "unrelated declaration should remain valid: {:?}",
        updated.snapshot.diagnostics
    );
    assert_eq!(
        session.db().query_state(&provider_signature_key).and_then(|state| state.revision()),
        signature_revision
    );
    assert_eq!(
        session.db().query_state(&consumer_body_key).and_then(|state| state.revision()),
        consumer_body_revision
    );
    assert_eq!(
        session.db().query_state(&provider_hierarchy_key).and_then(|state| state.revision()),
        hierarchy_revision
    );
    assert_eq!(updated.stats.callable_signatures_recomputed, 0, "existing callable signature must be retained");
    assert_eq!(
        updated.stats.hierarchy_edges_recomputed, 1,
        "only the added declaration hierarchy edge should recompute"
    );
}

#[test]
fn pa9_large_hierarchy_edit_isolates_unrelated_formal_products() {
    let provider_v1 =
        "class BaseA { @class value() -> Int { 1 } }\nclass BaseB { @class value() -> String { \"b\" } }\nclass Provider is BaseA {}\nexport Provider\n";
    let provider_v2 =
        "class BaseA { @class value() -> Int { 1 } }\nclass BaseB { @class value() -> String { \"b\" } }\nclass Provider is BaseB {}\nexport Provider\n";
    let mut incremental = SemanticWorkspaceSession::new();
    let initial = incremental.update(high_fanout_input(provider_v1, 1));
    assert!(!initial.snapshot.has_errors(), "initial diagnostics: {:?}", initial.snapshot.diagnostics);

    let provider = module("fanout_provider");
    let provider_declaration = DeclarationId::new(provider.clone(), "Provider".into());
    let provider_hierarchy_key = QueryKey::HierarchyEdge(provider_declaration.clone());
    let consumer_module = module("fanout_consumer0000");
    let consumer_body = CallableId::new(
        DeclarationId::new(consumer_module, "Consumer0000".into()),
        Selector::method("read", []).unwrap(),
        DispatchSide::Class,
    );
    let stable_module = module("fanout_consumer4999");
    let stable_declaration = DeclarationId::new(stable_module.clone(), "Consumer4999".into());
    let stable_callable = CallableId::new(stable_declaration.clone(), Selector::method("stable", []).unwrap(), DispatchSide::Class);
    let stable_keys = [
        QueryKey::HierarchyEdge(stable_declaration.clone()),
        QueryKey::DeclarationSurface(stable_declaration.clone()),
        QueryKey::CallableSignature(stable_callable.clone()),
        QueryKey::CallableBody(stable_callable.clone()),
    ];
    let stable_revisions = stable_keys
        .iter()
        .map(|key| {
            (
                key.clone(),
                incremental
                    .db()
                    .query_state(key)
                    .and_then(|state| state.revision())
                    .expect("initial stable product"),
            )
        })
        .collect::<BTreeMap<_, _>>();
    let provider_hierarchy_revision = incremental
        .db()
        .query_state(&provider_hierarchy_key)
        .and_then(|state| state.revision())
        .expect("initial provider hierarchy edge");
    let consumer_initial_revision = incremental
        .db()
        .query_state(&QueryKey::CallableBody(consumer_body.clone()))
        .and_then(|state| state.revision())
        .expect("initial exact consumer body");

    let updated = incremental.update(high_fanout_input(provider_v2, 2));
    assert!(
        updated.snapshot.has_errors(),
        "the 100 exact consumers should observe BaseB's String method: {:?}",
        updated.snapshot.diagnostics
    );
    assert_eq!(updated.stats.semantic_structure_shards_recomputed, 1, "only the provider shard changes");
    assert_eq!(
        updated.stats.semantic_structure_shards_reused, 5_000,
        "all reverse-connected importer shards remain retained"
    );
    assert!(
        updated.stats.reverse_candidates_considered <= 350,
        "hierarchy propagation must remain bounded by exact consumers: {:?}",
        updated.stats
    );
    assert!(
        updated.stats.callable_bodies_recomputed >= 100,
        "all exact hierarchy consumers must recompute: {:?}",
        updated.stats
    );

    assert_ne!(
        incremental.db().query_state(&provider_hierarchy_key).and_then(|state| state.revision()),
        Some(provider_hierarchy_revision),
        "the changed Provider hierarchy edge must not retain its old revision"
    );
    assert_ne!(
        incremental
            .db()
            .query_state(&QueryKey::CallableBody(consumer_body))
            .and_then(|state| state.revision()),
        Some(consumer_initial_revision),
        "an exact downstream hierarchy consumer must recompute"
    );
    for (key, revision) in stable_revisions {
        let state = incremental.db().query_state(&key).expect("retained unrelated product");
        assert_eq!(
            state.revision(),
            Some(revision),
            "unrelated product {key:?} must retain its computation revision"
        );
    }

    let mut cold = SemanticWorkspaceSession::new();
    let cold_result = cold.update(high_fanout_input(provider_v2, 1));
    if updated.snapshot.hierarchy.superclasses != cold_result.snapshot.hierarchy.superclasses {
        let incremental_hierarchy = &updated.snapshot.hierarchy.superclasses;
        let cold_hierarchy = &cold_result.snapshot.hierarchy.superclasses;
        let missing = cold_hierarchy.keys().filter(|key| !incremental_hierarchy.contains_key(*key)).count();
        let extra = incremental_hierarchy.keys().filter(|key| !cold_hierarchy.contains_key(*key)).count();
        let changed = incremental_hierarchy
            .iter()
            .filter(|(key, value)| cold_hierarchy.get(*key) != Some(*value))
            .count();
        let missing_keys = cold_hierarchy
            .keys()
            .filter(|key| !incremental_hierarchy.contains_key(*key))
            .take(8)
            .collect::<Vec<_>>();
        panic!(
            "incremental and cold hierarchy results differ: incremental={}, cold={}, missing={missing}, extra={extra}, changed={changed}, missing_keys={missing_keys:?}",
            incremental_hierarchy.len(),
            cold_hierarchy.len()
        );
    }
}

fn high_fanout_input(provider_source: &str, generation: u64) -> SemanticWorkspaceInput {
    const IMPORTER_COUNT: usize = 5_000;
    const EXACT_CONSUMER_COUNT: usize = 100;
    let provider = module("fanout_provider");
    let mut sources = BTreeMap::new();
    let provider_source: Arc<str> = Arc::from(provider_source.to_owned());
    let include_stable_body = provider_source.contains("class BaseA");
    let provider_program = Arc::new(phalcom_ast::parse(&provider_source, 0).program);
    sources.insert(
        provider.clone(),
        Arc::new(ParsedModuleUnit::new(
            provider.clone(),
            ModuleKind::Module,
            None,
            provider_source,
            provider_program,
        )),
    );

    let provider_export = LinkedExport {
        public_name: "Provider".into(),
        target: LinkedExportTarget::Binding(SymbolId {
            module: provider.clone(),
            name: "Provider".into(),
        }),
        range: phalcom_common::range::SourceRange::default(),
    };
    let provider_module = LinkedModule {
        interface: LinkedModuleInterface {
            module: provider.clone(),
            kind: ModuleKind::Module,
            exports: BTreeMap::from([("Provider".into(), provider_export)]),
            metadata: ModuleMetadata::default(),
        },
        bindings: ModuleBindingLayout {
            local_globals: BTreeMap::from([("Provider".into(), GlobalBindingId(0))]),
            ..ModuleBindingLayout::default()
        },
        linked_reads: Vec::new(),
        runtime_dependencies: Vec::new(),
    };

    let mut linked_modules = BTreeMap::from([(provider.clone(), provider_module)]);
    let mut initialization_order = vec![provider.clone()];
    for index in 0..IMPORTER_COUNT {
        let consumer = module(&format!("fanout_consumer{index:04}"));
        let reads_provider = index < EXACT_CONSUMER_COUNT;
        let source: Arc<str> = if reads_provider {
            Arc::from(format!(
                "import fanout_provider.Provider\nclass Consumer{index:04} {{ @class read() -> Int {{ Provider.value() }} }}\n"
            ))
        } else if index == IMPORTER_COUNT - 1 && include_stable_body {
            Arc::from(format!(
                "import fanout_provider.Provider\nclass Consumer{index:04} {{ @class stable() -> Int {{ 9 }} }}\n"
            ))
        } else {
            Arc::from(format!("import fanout_provider.Provider\nclass Consumer{index:04} {{}}\n"))
        };
        let program = Arc::new(phalcom_ast::parse(&source, 0).program);
        sources.insert(
            consumer.clone(),
            Arc::new(ParsedModuleUnit::new(consumer.clone(), ModuleKind::Module, None, source, program)),
        );
        linked_modules.insert(
            consumer.clone(),
            LinkedModule {
                interface: LinkedModuleInterface {
                    module: consumer.clone(),
                    kind: ModuleKind::Module,
                    exports: BTreeMap::new(),
                    metadata: ModuleMetadata::default(),
                },
                bindings: ModuleBindingLayout {
                    imports: BTreeMap::from([("Provider".into(), ImportBindingId(0))]),
                    ..ModuleBindingLayout::default()
                },
                linked_reads: vec![LinkedReadSpec::Binding(SymbolId {
                    module: provider.clone(),
                    name: "Provider".into(),
                })],
                runtime_dependencies: vec![provider.clone()],
            },
        );
        initialization_order.push(consumer);
    }

    SemanticWorkspaceInput::new(
        Arc::new(LinkedProgram {
            universe: Arc::new(phalcom_modules::project::ProjectUniverse::new()),
            modules: linked_modules,
            graphs: phalcom_modules::graph::ModuleGraphs::default(),
            entry: provider,
            initialization_order,
        }),
        sources,
        generation,
    )
}

#[test]
fn pa10_cold_and_incremental_snapshots_have_presentation_parity_across_mutations() {
    const LAST_STEP: usize = 15;
    let mut incremental = SemanticWorkspaceSession::new();
    let initial = incremental
        .update_with_budget_and_cancel(parity_input(0, 1), QueryBudget::default(), &CancellationToken::new())
        .expect("initial incremental parity update");
    let initial_projection = semantic_parity_projection(&initial.snapshot);
    let mut cold = SemanticWorkspaceSession::new();
    let cold_initial = cold
        .update_with_budget_and_cancel(parity_input(0, 1), QueryBudget::default(), &CancellationToken::new())
        .expect("initial cold parity update");
    assert_eq!(initial_projection, semantic_parity_projection(&cold_initial.snapshot));

    for step in 1..=LAST_STEP {
        let input = parity_input(step, step as u64 + 1);
        let incremental_result = incremental
            .update_with_budget_and_cancel(input.clone(), QueryBudget::default(), &CancellationToken::new())
            .expect("incremental parity update");
        cold = SemanticWorkspaceSession::new();
        let cold_result = cold
            .update_with_budget_and_cancel(input, QueryBudget::default(), &CancellationToken::new())
            .expect("cold parity update");

        let incremental_projection = semantic_parity_projection(&incremental_result.snapshot);
        let cold_projection = semantic_parity_projection(&cold_result.snapshot);

        if incremental_projection != cold_projection {
            report_semantic_parity_diff(step, &incremental_projection, &cold_projection);
        }

        assert_eq!(
            incremental_projection, cold_projection,
            "cold/incremental semantic presentation diverged after mutation step {step}",
        );
    }
}

#[derive(Debug, Eq, PartialEq)]
struct SemanticParityProjection {
    status: String,
    module_products: Vec<String>,
    declarations: Vec<String>,
    surfaces: Vec<String>,
    hierarchy: Vec<String>,
    callable_signatures: Vec<String>,
    field_signatures: Vec<String>,
    aliases: Vec<String>,
    diagnostics: Vec<String>,
    semantic_graph: Vec<String>,
    callable_analyses: Vec<String>,
    source_index: Vec<String>,
}

#[track_caller]
fn report_parity_vec_diff(step: usize, field: &str, incremental: &[String], cold: &[String]) {
    if incremental == cold {
        return;
    }

    eprintln!(
        "\nPA-10 step {step}: field `{field}` diverged \
         (incremental_len={}, cold_len={})",
        incremental.len(),
        cold.len(),
    );

    let common_len = incremental.len().min(cold.len());

    if let Some(index) = (0..common_len).find(|&index| incremental[index] != cold[index]) {
        eprintln!("first differing index: {index}");
        eprintln!("  incremental: {:?}", incremental[index]);
        eprintln!("  cold:        {:?}", cold[index]);
        return;
    }

    // If all shared positions agree, the difference is an extra/missing entry.
    if incremental.len() > common_len {
        eprintln!("first incremental-only trailing entry at {common_len}: {:?}", incremental[common_len]);
    }

    if cold.len() > common_len {
        eprintln!("first cold-only trailing entry at {common_len}: {:?}", cold[common_len]);
    }
}

fn report_semantic_parity_diff(step: usize, incremental: &SemanticParityProjection, cold: &SemanticParityProjection) {
    if incremental.status != cold.status {
        eprintln!(
            "\nPA-10 step {step}: field `status` diverged\n\
             incremental: {:?}\n\
             cold:        {:?}",
            incremental.status, cold.status,
        );
    }

    report_parity_vec_diff(step, "module_products", &incremental.module_products, &cold.module_products);
    report_parity_vec_diff(step, "declarations", &incremental.declarations, &cold.declarations);
    report_parity_vec_diff(step, "surfaces", &incremental.surfaces, &cold.surfaces);
    report_parity_vec_diff(step, "hierarchy", &incremental.hierarchy, &cold.hierarchy);
    report_parity_vec_diff(step, "callable_signatures", &incremental.callable_signatures, &cold.callable_signatures);
    report_parity_vec_diff(step, "field_signatures", &incremental.field_signatures, &cold.field_signatures);
    report_parity_vec_diff(step, "aliases", &incremental.aliases, &cold.aliases);
    report_parity_vec_diff(step, "diagnostics", &incremental.diagnostics, &cold.diagnostics);
    report_parity_vec_diff(step, "semantic_graph", &incremental.semantic_graph, &cold.semantic_graph);
    report_parity_vec_diff(step, "callable_analyses", &incremental.callable_analyses, &cold.callable_analyses);
    report_parity_vec_diff(step, "source_index", &incremental.source_index, &cold.source_index);
}

fn semantic_parity_projection(snapshot: &SemanticSnapshot) -> SemanticParityProjection {
    let mut module_products = Vec::new();
    for (module, interface) in snapshot.module_products.unlinked.iter() {
        module_products.push(format!("unlinked:{module:?}:{interface:?}"));
    }
    for (module, interface) in snapshot.module_products.linked.iter() {
        module_products.push(format!("linked:{module:?}:{interface:?}"));
    }
    for ((module, name), target) in snapshot.module_products.resolved_imports.iter() {
        module_products.push(format!("resolved-import:{module:?}:{name}:{target:?}"));
    }
    for (module, imports) in snapshot.module_products.reverse_imports.iter() {
        module_products.push(format!("reverse-import:{module:?}:{imports:?}"));
    }
    module_products.sort();

    let mut declarations = snapshot
        .declarations
        .iter()
        .map(|(declaration, info)| {
            format!(
                "{declaration:?}|form={}|class={}|kind={}|generic={}|super={}",
                snapshot.store.format_type(info.form),
                snapshot.store.format_type(info.class_object_type),
                snapshot.store.format_kind(info.kind),
                generic_signature_key(snapshot, info.generic_signature.as_ref()),
                info.supertype_template
                    .as_ref()
                    .and_then(|template| template.structural_form.as_deref())
                    .unwrap_or("-")
            )
        })
        .collect::<Vec<_>>();
    declarations.sort();

    let mut surfaces = snapshot
        .dispatch
        .surfaces()
        .iter()
        .map(|(declaration, surface)| {
            format!(
                "{declaration:?}|instance={}|class={}",
                member_surface_key(snapshot, &surface.instance),
                member_surface_key(snapshot, &surface.class)
            )
        })
        .collect::<Vec<_>>();
    surfaces.sort();

    let mut hierarchy = snapshot
        .hierarchy
        .superclasses
        .iter()
        .map(|(declaration, superclass)| format!("{declaration:?}->{superclass:?}"))
        .chain(
            snapshot
                .hierarchy
                .templates
                .iter()
                .map(|(declaration, template)| format!("template:{declaration:?}:{}", template.structural_form.as_deref().unwrap_or("-"))),
        )
        .collect::<Vec<_>>();
    hierarchy.sort();

    let mut callable_signatures = snapshot
        .callable_signatures
        .iter()
        .map(|(callable, signature)| {
            let parameters = signature
                .parameters
                .iter()
                .map(|parameter| {
                    format!(
                        "{}:{}:{:?}:{:?}",
                        parameter.local_name,
                        parameter.external_label.as_deref().unwrap_or("-"),
                        parameter.rest,
                        declared_type_key(snapshot, &parameter.declared_type),
                    )
                })
                .collect::<Vec<_>>();
            format!(
                "{callable:?}|owner={:?}|side={:?}|selector={:?}|generic={}|parameters={parameters:?}|return={}|validation={:?}|inferred={}|implementation={:?}|native={:?}|effects={:?}|raises={:?}|flow={:?}|lifecycle={:?}",
                signature.owner,
                signature.side,
                signature.selector,
                generic_signature_key(snapshot, signature.generics.as_ref()),
                declared_type_key(snapshot, &signature.declared_return),
                signature.return_validation,
                signature.inferred_return.as_ref().map_or_else(|| "-".to_string(), |knowledge| knowledge_key(snapshot, knowledge)),
                signature.implementation,
                signature.native_id,
                signature.effects,
                signature.raises,
                signature.flow,
                signature.lifecycle,
            )
        })
        .collect::<Vec<_>>();
    callable_signatures.sort();

    let mut field_signatures = snapshot
        .field_signatures
        .iter()
        .map(|(field, signature)| {
            format!(
                "{field:?}|owner={:?}|side={:?}|name={}|mutable={}|type={}",
                signature.owner,
                signature.side,
                signature.name,
                signature.mutable,
                declared_type_key(snapshot, &signature.declared_type),
            )
        })
        .collect::<Vec<_>>();
    field_signatures.sort();

    let mut aliases = snapshot
        .type_aliases
        .iter()
        .map(|(declaration, alias)| {
            format!(
                "{declaration:?}|kind={}|shape={}|form={}|generic={}|deps={:?}",
                snapshot.store.format_kind(alias.kind),
                alias.kind_shape,
                alias.structural_form,
                generic_signature_key(snapshot, alias.generic_signature.as_ref()),
                alias.dependencies,
            )
        })
        .collect::<Vec<_>>();
    aliases.sort();

    let mut diagnostics = snapshot
        .diagnostics
        .iter()
        .flat_map(|(module, values)| values.iter().map(move |diagnostic| diagnostic_key(snapshot, module, diagnostic)))
        .collect::<Vec<_>>();
    diagnostics.sort();

    let mut semantic_graph = Vec::new();
    for node in snapshot.semantic_graph.nodes() {
        for edge in snapshot.semantic_graph.edges_from(&node) {
            semantic_graph.push(format!("{node:?}->{:?}:{:?}", edge.to, edge.kind));
        }
    }
    semantic_graph.sort();

    let mut callable_analyses = snapshot
        .callable_analyses
        .iter()
        .map(|(callable, analysis)| {
            let current_body_range = snapshot
                .source_index
                .module(&callable.module())
                .and_then(|index| index.structure.callable_body_ranges.get(callable).copied())
                .unwrap_or(analysis.body_range);
            let expressions = analysis
                .expressions
                .values()
                .map(|expression| {
                    let current_range = snapshot
                        .source_index
                        .source_site_for_expression(callable, expression.id)
                        .map(|site| site.range)
                        .unwrap_or(expression.range);
                    format!(
                        "{:?}:{:?}:{}:{:?}:{:?}",
                        expression.id,
                        current_range,
                        knowledge_key(snapshot, &expression.knowledge),
                        analysis_status_key(&expression.status),
                        causal_invalidity_key(expression.causal_invalidity),
                    )
                })
                .collect::<Vec<_>>();
            let bindings = analysis
                .bindings
                .values()
                .map(|binding| {
                    let current_range = snapshot
                        .source_index
                        .source_site_for_binding(callable, binding.binding)
                        .map(|site| site.range)
                        .unwrap_or(binding.range);
                    format!(
                        "{:?}:{}:{:?}:{}:{:?}:{}:{}",
                        binding.binding,
                        binding.name,
                        current_range,
                        knowledge_key(snapshot, &binding.current),
                        binding.consistency,
                        binding.mutable,
                        causal_invalidity_key(binding.causal_invalidity),
                    )
                })
                .collect::<Vec<_>>();
            let analysis_diagnostics = analysis
                .diagnostics
                .iter()
                .map(|diagnostic| diagnostic_key(snapshot, callable.module(), diagnostic))
                .collect::<Vec<_>>();
            format!(
                "{callable:?}|range={:?}|status={}|validation={:?}|expressions={expressions:?}|bindings={bindings:?}|diagnostics={analysis_diagnostics:?}",
                current_body_range,
                callable_analysis_status_key(analysis.status),
                analysis.return_validation,
            )
        })
        .collect::<Vec<_>>();
    callable_analyses.sort();

    let mut source_index = snapshot
        .source_index
        .modules()
        .map(|(module, index)| {
            let fingerprints = index.fingerprints();
            format!(
                "{module:?}|semantic={:?}|presentation={:?}|sites={}|occurrences={}|attachments={}",
                fingerprints.semantic,
                fingerprints.presentation,
                index.structure.sites.len(),
                index.occurrences.all().len(),
                index.attachments.len()
            )
        })
        .collect::<Vec<_>>();
    source_index.sort();

    SemanticParityProjection {
        status: format!("{:?}", snapshot.status),
        module_products,
        declarations,
        surfaces,
        hierarchy,
        callable_signatures,
        field_signatures,
        aliases,
        diagnostics,
        semantic_graph,
        callable_analyses,
        source_index,
    }
}

fn generic_signature_key(snapshot: &SemanticSnapshot, signature: Option<&GenericSignature>) -> String {
    let Some(signature) = signature else {
        return "-".to_string();
    };
    let kinds = signature
        .parameter_kinds
        .iter()
        .map(|kind| snapshot.store.format_kind(*kind))
        .collect::<Vec<_>>();
    format!(
        "count={}|kinds={kinds:?}|shapes={:?}|variances={:?}|constraints={:?}|constraint_count={}",
        signature.parameters.len(),
        signature.parameter_kind_shapes,
        signature.parameter_variances,
        signature.constraint_shapes,
        signature.constraints.len(),
    )
}

fn declared_type_key(snapshot: &SemanticSnapshot, fact: &phalcom_semantic::declaration_type::DeclaredTypeFact) -> String {
    let state = match &fact.state {
        DeclaredTypeState::Known(TypeTerm::Canonical(ty)) => format!("known:{}", snapshot.store.format_type(*ty)),
        DeclaredTypeState::Known(TypeTerm::SelfType(term)) => format!("self:{:?}:{:?}:{:?}", term.owner, term.side, term.role),
        DeclaredTypeState::Known(TypeTerm::Infer(_)) => "infer".to_string(),
        DeclaredTypeState::Dynamic(reason) => format!("dynamic:{reason:?}"),
        DeclaredTypeState::Unknown(reason) => format!("unknown:{reason:?}"),
    };
    format!("{state}:basis={:?}", fact.basis)
}

fn knowledge_key(snapshot: &SemanticSnapshot, knowledge: &TypeKnowledge) -> String {
    format!(
        "{}:{:?}:{:?}",
        snapshot.store.format_knowledge(knowledge),
        knowledge.status(),
        knowledge.origin(),
    )
}

fn member_surface_key(snapshot: &SemanticSnapshot, surface: &MemberSurface) -> String {
    let mut fields = surface
        .fields
        .iter()
        .map(|(name, knowledge)| format!("{name}:{}:{:?}", snapshot.store.format_knowledge(knowledge), surface.field_visibility.get(name)))
        .collect::<Vec<_>>();
    fields.sort();
    let mut callables = surface
        .callable_signatures
        .iter()
        .map(|(selector, signature)| format!("{selector:?}:{:?}:{:?}", surface.callable_visibility.get(selector), signature.return_type))
        .collect::<Vec<_>>();
    callables.sort();
    format!("fields={fields:?}|callables={callables:?}")
}

fn diagnostic_key(snapshot: &SemanticSnapshot, module: &phalcom_modules::identity::ModuleId, diagnostic: &SemanticDiagnostic) -> String {
    let labels = diagnostic
        .labels
        .iter()
        .map(|label| format!("{:?}:{:?}:{}", label.span.module, label.range, label.message))
        .collect::<Vec<_>>();
    let guidance = diagnostic
        .guidance
        .iter()
        .map(|guidance| match guidance {
            DiagnosticGuidance::ChangeAnnotation { range, ty } => format!("change:{range:?}:{}", snapshot.store.format_type(*ty)),
            DiagnosticGuidance::SupplyAssignableValue { expected } => format!("supply:{}", snapshot.store.format_type(*expected)),
            DiagnosticGuidance::UseCallableShape { callable } => format!("callable:{callable:?}"),
            DiagnosticGuidance::EstablishTypeEvidence { range, expected } => {
                format!(
                    "evidence:{range:?}:{}",
                    expected.map_or_else(|| "-".to_string(), |ty| snapshot.store.format_type(ty))
                )
            }
            DiagnosticGuidance::ResolveGenericParameter { parameter } => format!("parameter-index={}", parameter.index()),
        })
        .collect::<Vec<_>>();
    format!(
        "{module:?}|{}|{:?}|{}|{:?}|labels={labels:?}|notes={:?}|helps={:?}|guidance={guidance:?}|fixes={:?}",
        diagnostic.code, diagnostic.severity, diagnostic.message, diagnostic.primary_range, diagnostic.notes, diagnostic.helps, diagnostic.fixes,
    )
}

fn analysis_status_key(status: &AnalysisStatus) -> &'static str {
    match status {
        AnalysisStatus::Ready => "ready",
        AnalysisStatus::Invalid(_) => "invalid",
        AnalysisStatus::Suppressed(_) => "suppressed",
        AnalysisStatus::Blocked(_) => "blocked",
        AnalysisStatus::DynamicBoundary(_) => "dynamic",
        AnalysisStatus::Cancelled => "cancelled",
        AnalysisStatus::BudgetExceeded(_) => "budget",
        AnalysisStatus::InternalFailure(_) => "internal",
    }
}

fn callable_analysis_status_key(status: CallableAnalysisStatus) -> &'static str {
    match status {
        CallableAnalysisStatus::Complete => "complete",
        CallableAnalysisStatus::Partial => "partial",
        CallableAnalysisStatus::Blocked => "blocked",
        CallableAnalysisStatus::Cancelled => "cancelled",
        CallableAnalysisStatus::BudgetExceeded => "budget",
        CallableAnalysisStatus::InternalFailure(_) => "internal",
    }
}

fn causal_invalidity_key(causal: CausalInvalidity) -> &'static str {
    match causal {
        CausalInvalidity::Clean => "clean",
        CausalInvalidity::One(_) => "one",
        CausalInvalidity::Multiple => "multiple",
    }
}

fn parity_input(step: usize, generation: u64) -> SemanticWorkspaceInput {
    let a = module("parity_a");
    let b = module("parity_b");
    let c = module("parity_c");
    let provider_body = match step {
        1 => "1 + 1",
        4 => "\"changed\"",
        _ => "1",
    };
    let provider_field = if step >= 5 { "\n  _field: Int" } else { "" };
    let provider_extra = if step == 2 { "\nclass Extra {}\nexport Extra" } else { "" };
    let provider_cycle = if matches!(step, 9) {
        "\ntype CycleA = CycleB\ntype CycleB = CycleA"
    } else {
        ""
    };
    let value_return = if step == 4 { "String" } else { "Int" };
    let provider_source: Arc<str> = Arc::from(format!(
        "class Base {{ @class value() -> {value_return} {{ {provider_body} }} }}\nclass Alt {{ @class value() -> Int {{ 1 }} }}{provider_field}{provider_extra}{provider_cycle}\ntype Alias = Base\nexport Base\nexport Alt\nexport Alias\n"
    ));

    let (imports, child_target, local_target) = if step == 7 {
        ("import parity_a.Alt as Base\nimport parity_a.Alt\n", "Base", "Base")
    } else {
        (
            "import parity_a.Base\nimport parity_a.Alt\n",
            if step == 6 { "Alt" } else { "Base" },
            if step == 8 { "Alt" } else { "Base" },
        )
    };
    let missing_body = if step == 11 { "Missing.value()" } else { "Base.value()" };
    let broken = if step == 14 {
        "\nclass Broken { @class value() -> Int { \"x\" } }"
    } else if step == 15 {
        "\nclass Broken { @class value() -> Int { 1 } }"
    } else {
        ""
    };
    let child_source: Arc<str> = Arc::from(format!(
        "{imports}class Child is {child_target} {{}}\ntype Local = {local_target}\nclass Consumer {{ @class read(_ value: Local) -> Int {{ {missing_body} }} }}{broken}\n"
    ));
    let c_source: Arc<str> = Arc::from("class Other { @class value() -> Int { 1 } }\nexport Other\n");

    let mut sources = BTreeMap::new();
    let mut source_units = vec![(a.clone(), provider_source), (b.clone(), child_source)];
    if step != 12 {
        source_units.push((c.clone(), c_source));
    }
    for (module_id, source) in source_units {
        let program = Arc::new(phalcom_ast::parse(&source, 0).program);
        sources.insert(
            module_id.clone(),
            Arc::new(ParsedModuleUnit::new(module_id, ModuleKind::Module, None, source, program)),
        );
    }

    let export = |owner: &ModuleId, name: &str| {
        (
            name.into(),
            LinkedExport {
                public_name: name.into(),
                target: LinkedExportTarget::Binding(SymbolId {
                    module: owner.clone(),
                    name: name.into(),
                }),
                range: phalcom_common::range::SourceRange::default(),
            },
        )
    };
    let provider_exports = if step == 2 {
        BTreeMap::from([export(&a, "Base"), export(&a, "Alt"), export(&a, "Alias"), export(&a, "Extra")])
    } else {
        BTreeMap::from([export(&a, "Base"), export(&a, "Alt"), export(&a, "Alias")])
    };
    let mut provider_globals = BTreeMap::from([
        ("Base".into(), GlobalBindingId(0)),
        ("Alt".into(), GlobalBindingId(1)),
        ("Alias".into(), GlobalBindingId(2)),
    ]);
    if step == 2 {
        provider_globals.insert("Extra".into(), GlobalBindingId(3));
    }
    let provider_module = LinkedModule {
        interface: LinkedModuleInterface {
            module: a.clone(),
            kind: ModuleKind::Module,
            exports: provider_exports,
            metadata: ModuleMetadata::default(),
        },
        bindings: ModuleBindingLayout {
            local_globals: provider_globals,
            ..ModuleBindingLayout::default()
        },
        linked_reads: Vec::new(),
        runtime_dependencies: Vec::new(),
    };
    let b_imports = BTreeMap::from([("Base".into(), ImportBindingId(0)), ("Alt".into(), ImportBindingId(1))]);
    let b_reads = if step == 7 {
        vec![
            LinkedReadSpec::Binding(SymbolId {
                module: a.clone(),
                name: "Alt".into(),
            }),
            LinkedReadSpec::Binding(SymbolId {
                module: a.clone(),
                name: "Alt".into(),
            }),
        ]
    } else {
        vec![
            LinkedReadSpec::Binding(SymbolId {
                module: a.clone(),
                name: "Base".into(),
            }),
            LinkedReadSpec::Binding(SymbolId {
                module: a.clone(),
                name: "Alt".into(),
            }),
        ]
    };
    let b_module = LinkedModule {
        interface: LinkedModuleInterface {
            module: b.clone(),
            kind: ModuleKind::Module,
            exports: BTreeMap::new(),
            metadata: ModuleMetadata::default(),
        },
        bindings: ModuleBindingLayout {
            local_globals: BTreeMap::from([
                ("Child".into(), GlobalBindingId(0)),
                ("Consumer".into(), GlobalBindingId(1)),
                ("Local".into(), GlobalBindingId(2)),
            ]),
            imports: b_imports,
        },
        linked_reads: b_reads,
        runtime_dependencies: vec![a.clone()],
    };
    let c_module = LinkedModule {
        interface: LinkedModuleInterface {
            module: c.clone(),
            kind: ModuleKind::Module,
            exports: BTreeMap::from([export(&c, "Other")]),
            metadata: ModuleMetadata::default(),
        },
        bindings: ModuleBindingLayout {
            local_globals: BTreeMap::from([("Other".into(), GlobalBindingId(0))]),
            ..ModuleBindingLayout::default()
        },
        linked_reads: Vec::new(),
        runtime_dependencies: Vec::new(),
    };
    let mut linked_modules = BTreeMap::from([(a.clone(), provider_module), (b.clone(), b_module)]);
    let mut initialization_order = vec![a.clone(), b.clone()];
    if step != 12 {
        linked_modules.insert(c.clone(), c_module);
        initialization_order.push(c.clone());
    }
    let linked = Arc::new(LinkedProgram {
        universe: Arc::new(phalcom_modules::project::ProjectUniverse::new()),
        modules: linked_modules,
        graphs: phalcom_modules::graph::ModuleGraphs::default(),
        entry: b,
        initialization_order,
    });
    SemanticWorkspaceInput::new(linked, sources, generation)
}

#[test]
fn a7_production_module_delta_body_edit_reuses_structural_world() {
    let source = |path: &str| {
        let path = PathBuf::from(path);
        SourceLocation {
            source_id: SourceId(path.to_string_lossy().into()),
            display_path: path,
        }
    };
    let a = source("/plan-a/a.ph");
    let b = source("/plan-a/b.ph");
    let c = source("/plan-a/c.ph");
    let mut session = SemanticWorkspaceSession::new();

    let initial = session
        .apply_module_mutations([
            WorkspaceSourceBatchMutation::SetOverlay {
                source: a.clone(),
                text: Arc::from("class A { value() -> Int { 1 } }\n"),
                revision: SourceRevision(1),
                recovered_program: None,
            },
            WorkspaceSourceBatchMutation::SetOverlay {
                source: b.clone(),
                text: Arc::from("class B { value() -> Int { 2 } }\n"),
                revision: SourceRevision(1),
                recovered_program: None,
            },
            WorkspaceSourceBatchMutation::SetOverlay {
                source: c.clone(),
                text: Arc::from("class C { value() -> Int { 3 } }\n"),
                revision: SourceRevision(1),
                recovered_program: None,
            },
        ])
        .expect("initial module workspace publication");

    let changed_module = initial
        .snapshot
        .sources
        .iter()
        .find(|(_, source)| source.source.as_ref().is_some_and(|location| location.display_path == a.display_path))
        .map(|(module, _)| module.clone())
        .expect("changed module is published");
    let retained_module = initial
        .snapshot
        .sources
        .iter()
        .find(|(_, source)| source.source.as_ref().is_some_and(|location| location.display_path == b.display_path))
        .map(|(module, _)| module.clone())
        .expect("unrelated module is published");
    let untouched_body_revisions = initial
        .snapshot
        .callable_analyses
        .keys()
        .filter(|callable| callable.module() != &changed_module)
        .filter(|callable| callable.declaration_owner().name.as_ref() != "<main>")
        .map(|callable| {
            let revision = session
                .db()
                .query_state(&QueryKey::CallableBody(callable.clone()))
                .and_then(|state| state.revision())
                .unwrap_or_else(|| panic!("untouched callable body product missing for {callable:?}"));
            (callable.clone(), revision)
        })
        .collect::<Vec<_>>();
    assert!(!untouched_body_revisions.is_empty(), "fixture must publish cross-module callable bodies");

    let updated = session
        .apply_module_mutations([WorkspaceSourceBatchMutation::SetOverlay {
            source: a,
            text: Arc::from("class A { value() -> Int { 4 } }\n"),
            revision: SourceRevision(2),
            recovered_program: None,
        }])
        .expect("body-only module delta publication");

    let module_stats = updated.module_stats.expect("production path publishes module stats");
    assert_eq!(module_stats.imports_resolved, 0);
    assert_eq!(module_stats.linked_components_recomputed, 0);
    assert_eq!(updated.stats.semantic_structure_shards_recomputed, 0);
    assert_eq!(updated.stats.semantic_structure_shards_reused, 3);
    assert!(updated.stats.query_products_recomputed > 0);
    assert!(Arc::ptr_eq(
        initial
            .snapshot
            .semantic_structure_shards
            .get(&retained_module)
            .expect("initial retained shard"),
        updated
            .snapshot
            .semantic_structure_shards
            .get(&retained_module)
            .expect("updated retained shard"),
    ));
    assert!(updated.snapshot.semantic_structure_shards.contains_key(&changed_module));
    for (callable, revision) in untouched_body_revisions {
        assert_eq!(
            session.db().query_state(&QueryKey::CallableBody(callable)).and_then(|state| state.revision()),
            Some(revision),
            "unrelated cross-module callable body must retain computation revision",
        );
    }
}
