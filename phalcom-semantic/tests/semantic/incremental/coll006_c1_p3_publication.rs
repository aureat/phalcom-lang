use super::support::{multi_module_input, single_module_input};
use phalcom_common::selector::Selector;
use phalcom_modules::identity::{ModuleComponent, ModuleId, ModulePath, ResolvedProjectId};
use phalcom_semantic::checker::analysis::SequenceLiteralFamily;
use phalcom_semantic::identity::{CallableId, DeclarationId, DispatchSide};
use phalcom_semantic::session::SemanticWorkspaceSession;

fn module(name: &str) -> ModuleId {
    ModuleId::resolved(
        ResolvedProjectId::from_raw(606),
        ModulePath::from_components(vec![ModuleComponent::from_identifier(name).unwrap()]),
    )
}

fn run_id(module: &ModuleId) -> CallableId {
    CallableId::new(
        DeclarationId::new(module.clone(), "Consumer".into()),
        Selector::method("run", []).unwrap(),
        DispatchSide::Class,
    )
}

fn sequence_meanings(
    snapshot: &phalcom_semantic::snapshot::SemanticSnapshot,
    callable: &CallableId,
) -> Vec<(SequenceLiteralFamily, String, String, Option<String>)> {
    let analysis = snapshot.callable_analyses.get(callable).expect("run analysis");
    analysis
        .sequence_literals
        .values()
        .map(|resolution| {
            (
                resolution.family,
                snapshot.store.format_type(resolution.result_type),
                snapshot.store.format_type(resolution.element_type),
                resolution.extent.map(|extent| snapshot.store.format_type(extent)),
            )
        })
        .collect()
}

fn source(failing: bool) -> String {
    let constraint = if failing { "String" } else { "Int" };
    format!(
        r#"
class Consumer {{
  @class reject<T>(_ values: Vector<T, 1>) -> Int where T == {constraint} {{ 0 }}
  @class run() {{
    let independent = <Int, 1>[9]
    let rejected = Consumer.reject(<Int, 1>[1])
  }}
}}
"#
    )
}

#[test]
fn publication_invalid_valid_invalid_matches_cold_and_reuses_unrelated_callable() {
    let main = module("publication");
    let other = module("other");
    let other_source = "class Other { @class stable() -> Int { 1 } }";
    let mut incremental = SemanticWorkspaceSession::new();

    let first = incremental.update(multi_module_input(vec![(main.clone(), source(true)), (other.clone(), other_source.into())], 1));
    let run = run_id(&main);
    let other_callable = CallableId::new(
        DeclarationId::new(other.clone(), "Other".into()),
        Selector::method("stable", []).unwrap(),
        DispatchSide::Class,
    );
    assert!(first.snapshot.has_errors());
    assert_eq!(sequence_meanings(&first.snapshot, &run).len(), 1);
    let other_first = first.snapshot.callable_analyses.get(&other_callable).expect("other callable").clone();

    let second = incremental.update(multi_module_input(vec![(main.clone(), source(false)), (other.clone(), other_source.into())], 2));
    assert!(!second.snapshot.has_errors(), "diagnostics: {:?}", second.snapshot.diagnostics);
    assert_eq!(sequence_meanings(&second.snapshot, &run).len(), 2);
    assert!(std::sync::Arc::ptr_eq(
        &other_first,
        second.snapshot.callable_analyses.get(&other_callable).expect("reused other callable"),
    ));

    let third = incremental.update(multi_module_input(vec![(main.clone(), source(true)), (other.clone(), other_source.into())], 3));
    assert!(third.snapshot.has_errors());
    assert_eq!(sequence_meanings(&third.snapshot, &run).len(), 1);

    for (generation, text, update) in [(1, source(true), &first), (2, source(false), &second), (3, source(true), &third)] {
        let mut cold = SemanticWorkspaceSession::new();
        let cold_update = cold.update(single_module_input(main.clone(), &text, generation));
        assert_eq!(update.snapshot.has_errors(), cold_update.snapshot.has_errors(), "generation {generation}");
        assert_eq!(
            sequence_meanings(&update.snapshot, &run),
            sequence_meanings(&cold_update.snapshot, &run),
            "generation {generation}"
        );
    }
}
