//! A7 semantic work-count publication and cold/incremental metric evidence.

use super::support::multi_module_input;
use phalcom_modules::identity::{ModuleComponent, ModuleId, ModulePath, ResolvedProjectId};
use phalcom_modules::{SourceId, SourceLocation, SourceRevision, WorkspaceSourceBatchMutation};
use phalcom_semantic::db::QueryKey;
use phalcom_semantic::session::SemanticWorkspaceSession;
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
