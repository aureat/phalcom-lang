use std::collections::BTreeMap;
use std::sync::Arc;

use phalcom_common::selector::Selector;
use phalcom_semantic::advisory::{
    AdvisoryCallableSummary, AdvisoryContributionSource, AdvisoryFact, AdvisoryParameterContributions, AdvisoryProductStatus, AdvisorySolver,
    AdvisorySolverBudget, AdvisorySolverNode, ValueShape,
};
use phalcom_semantic::db::CancellationToken;
use phalcom_semantic::identity::{CallableId, DeclarationId, DispatchSide, ModuleId};

fn callable(name: &str) -> CallableId {
    CallableId::new(
        DeclarationId::new(ModuleId::universe_root(), "Solver".into()),
        Selector::method(name, []).unwrap(),
        DispatchSide::Instance,
    )
}

fn node(id: CallableId, return_fact: AdvisoryFact, dependencies: Vec<CallableId>) -> AdvisorySolverNode {
    let summary = Arc::new(AdvisoryCallableSummary::new(
        id.clone(),
        Vec::new(),
        return_fact,
        dependencies,
        Default::default(),
        AdvisoryProductStatus::Complete,
    ));
    AdvisorySolverNode {
        summary,
        parameters: AdvisoryParameterContributions::default(),
    }
}

#[test]
fn recursive_summaries_converge_to_deterministic_bounded_join() {
    let left = callable("left");
    let right = callable("right");
    let mut nodes = BTreeMap::new();
    nodes.insert(
        left.clone(),
        node(
            left.clone(),
            AdvisoryFact::new(
                ValueShape::Instance(DeclarationId::new(ModuleId::universe_root(), "Left".into())),
                phalcom_semantic::AdvisoryConfidence::Exact,
            ),
            vec![right.clone()],
        ),
    );
    nodes.insert(
        right.clone(),
        node(
            right.clone(),
            AdvisoryFact::new(
                ValueShape::Instance(DeclarationId::new(ModuleId::universe_root(), "Right".into())),
                phalcom_semantic::AdvisoryConfidence::Exact,
            ),
            vec![left.clone()],
        ),
    );

    let first = AdvisorySolver::new(AdvisorySolverBudget { max_steps: 32 }).solve(nodes.clone());
    let second = AdvisorySolver::new(AdvisorySolverBudget { max_steps: 32 }).solve(nodes);

    assert!(first.converged);
    assert_eq!(first.status, AdvisoryProductStatus::Complete);
    assert_eq!(first.summaries, second.summaries);
    assert!(matches!(first.summaries[&left].return_fact.shape, ValueShape::Union(_)));
    assert!(matches!(first.summaries[&right].return_fact.shape, ValueShape::Union(_)));
}

#[test]
fn solver_budget_and_cancellation_are_explicit_non_ready_outcomes() {
    let id = callable("one");
    let nodes = BTreeMap::from([(
        id.clone(),
        node(id, AdvisoryFact::new(ValueShape::Unit, phalcom_semantic::AdvisoryConfidence::Exact), Vec::new()),
    )]);

    let budget = AdvisorySolver::new(AdvisorySolverBudget { max_steps: 0 }).solve(nodes.clone());
    assert_eq!(budget.status, AdvisoryProductStatus::BudgetExceeded);
    assert!(!budget.converged);

    let cancel = CancellationToken::new();
    cancel.cancel();
    let cancelled = AdvisorySolver::new(AdvisorySolverBudget { max_steps: 1 }).solve_with_cancel(nodes, &cancel);
    assert_eq!(cancelled.status, AdvisoryProductStatus::Cancelled);
    assert!(!cancelled.converged);
}

#[test]
fn contribution_replacement_and_removal_do_not_retain_stale_facts() {
    let slot = phalcom_semantic::advisory::AdvisoryParameterSlot::new(callable("consumer"), 0);
    let source_a = AdvisoryContributionSource::Callable(callable("caller_a"));
    let source_b = AdvisoryContributionSource::Callable(callable("caller_b"));
    let int = AdvisoryFact::new(
        ValueShape::Instance(DeclarationId::new(ModuleId::universe_root(), "Int".into())),
        phalcom_semantic::AdvisoryConfidence::Exact,
    );
    let string = AdvisoryFact::new(
        ValueShape::Instance(DeclarationId::new(ModuleId::universe_root(), "String".into())),
        phalcom_semantic::AdvisoryConfidence::Exact,
    );
    let mut contributions = AdvisoryParameterContributions::default();
    contributions.replace_source(source_a.clone(), BTreeMap::from([(slot.clone(), int.clone())]));
    contributions.replace_source(source_b.clone(), BTreeMap::from([(slot.clone(), string)]));
    assert!(matches!(contributions.get(&slot).unwrap().shape, ValueShape::Union(_)));
    contributions.remove_source(&source_a);
    assert_eq!(
        contributions.get(&slot).unwrap(),
        &AdvisoryFact::new(
            ValueShape::Instance(DeclarationId::new(ModuleId::universe_root(), "String".into())),
            phalcom_semantic::AdvisoryConfidence::Exact,
        )
    );
    contributions.remove_source(&source_b);
    assert!(contributions.get(&slot).is_none());
}
