//! A7 semantic work-count publication and cold/incremental metric evidence.

use super::support::multi_module_input;
use phalcom_modules::identity::{ModuleComponent, ModuleId, ModulePath, ResolvedProjectId};
use phalcom_modules::interface::{LinkedExport, LinkedExportTarget, LinkedModuleInterface};
use phalcom_modules::linker::{ImportBindingId, LinkedModule, LinkedProgram, LinkedReadSpec, ModuleBindingLayout, SymbolId};
use phalcom_modules::metadata::ModuleMetadata;
use phalcom_modules::source::ModuleKind;
use phalcom_modules::{SourceId, SourceLocation, SourceRevision, WorkspaceSourceBatchMutation};
use phalcom_semantic::db::QueryKey;
use phalcom_semantic::session::SemanticWorkspaceSession;
use phalcom_semantic::source::ParsedModuleUnit;
use phalcom_semantic::workspace::SemanticWorkspaceInput;
use phalcom_modules::graph::{SemanticEdgeKind, SemanticNodeId};
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
    assert!(initial.snapshot.semantic_graph.edges_from(&child).iter().any(|edge| {
        edge.kind == SemanticEdgeKind::Superclass && edge.to == base
    }));

    let updated = session.update(multi_module_input(
        vec![
            (owner, "class Base {}\nclass Child is Base {}".into()),
            (unrelated, "class Unrelated {}\nclass Added {}".into()),
        ],
        2,
    ));

    assert!(updated.snapshot.semantic_graph.edges_from(&child).iter().any(|edge| {
        edge.kind == SemanticEdgeKind::Superclass && edge.to == base
    }));
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
    assert!(hierarchy_dependencies
        .iter()
        .any(|edge| edge.dependency == QueryKey::LinkedName(module("a"), "B".into())));
    assert!(
        hierarchy_dependencies.iter().any(|edge| {
            edge.dependency == QueryKey::DeclarationShell(phalcom_semantic::identity::DeclarationId::new(module("b"), "B".into()))
        }),
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

fn child_module(node: &SemanticNodeId) -> phalcom_modules::identity::ModuleId {
    match node {
        SemanticNodeId::Declaration { module, .. } => module.clone(),
        SemanticNodeId::Module(module) => module.clone(),
    }
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
