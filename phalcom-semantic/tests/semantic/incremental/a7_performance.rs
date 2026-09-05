//! A7 semantic work-count publication and cold/incremental metric evidence.

use super::support::multi_module_input;
use phalcom_modules::identity::{ModuleComponent, ModuleId, ModulePath, ResolvedProjectId};
use phalcom_semantic::session::SemanticWorkspaceSession;

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
