//! Deterministic advisory summary solver.
//!
//! The solver owns only a bounded, per-update worklist. It does not retain a
//! second workspace graph and it never feeds results into formal checking.

use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::sync::Arc;

use crate::db::CancellationToken;
use crate::identity::CallableId;

use super::{AdvisoryCallableSummary, AdvisoryParameterContributions, AdvisoryProductStatus};

/// Explicit work budget for one advisory solve.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AdvisorySolverBudget {
    pub max_steps: usize,
}

impl Default for AdvisorySolverBudget {
    fn default() -> Self {
        Self { max_steps: 100_000 }
    }
}

/// One callable transfer seed and its canonical dependency edges.
#[derive(Clone, Debug)]
pub struct AdvisorySolverNode {
    pub summary: Arc<AdvisoryCallableSummary>,
    pub parameters: AdvisoryParameterContributions,
}

/// Result of one deterministic advisory solve.
#[derive(Clone, Debug)]
pub struct AdvisorySolverResult {
    pub summaries: BTreeMap<CallableId, Arc<AdvisoryCallableSummary>>,
    pub status: AdvisoryProductStatus,
    pub steps: usize,
    pub converged: bool,
}

/// Worklist/SCC-compatible advisory summary solver.
#[derive(Clone, Debug)]
pub struct AdvisorySolver {
    budget: AdvisorySolverBudget,
}

impl AdvisorySolver {
    pub fn new(budget: AdvisorySolverBudget) -> Self {
        Self { budget }
    }

    /// Solves until no summary changes or the explicit budget is exhausted.
    /// Dependency order is canonical and recursive components converge through
    /// bounded `AdvisoryFact::join` rather than a fixed pass count.
    pub fn solve(&self, nodes: BTreeMap<CallableId, AdvisorySolverNode>) -> AdvisorySolverResult {
        self.solve_with_cancel(nodes, &CancellationToken::new())
    }

    pub fn solve_with_cancel(&self, nodes: BTreeMap<CallableId, AdvisorySolverNode>, cancel: &CancellationToken) -> AdvisorySolverResult {
        let mut summaries = nodes
            .iter()
            .map(|(callable, node)| (callable.clone(), node.summary.clone()))
            .collect::<BTreeMap<_, _>>();
        let mut dependents = BTreeMap::<CallableId, BTreeSet<CallableId>>::new();
        for (callable, node) in &nodes {
            for dependency in node.summary.dependencies.iter() {
                if nodes.contains_key(dependency) && dependency != callable {
                    dependents.entry(dependency.clone()).or_default().insert(callable.clone());
                }
            }
        }
        let mut queue = summaries.keys().cloned().collect::<VecDeque<_>>();
        let mut queued = summaries.keys().cloned().collect::<BTreeSet<_>>();
        let mut steps = 0;
        while let Some(callable) = queue.pop_front() {
            queued.remove(&callable);
            if cancel.is_cancelled() {
                return AdvisorySolverResult {
                    summaries,
                    status: AdvisoryProductStatus::Cancelled,
                    steps,
                    converged: false,
                };
            }
            if steps >= self.budget.max_steps {
                return AdvisorySolverResult {
                    summaries,
                    status: AdvisoryProductStatus::BudgetExceeded,
                    steps,
                    converged: false,
                };
            }
            steps += 1;
            let Some(node) = nodes.get(&callable) else { continue };
            let mut return_fact = node.summary.return_fact.clone();
            let mut status = node.summary.status.clone();
            for dependency in node.summary.dependencies.iter() {
                let Some(dependency_summary) = summaries.get(dependency) else {
                    status = nonready_status(status);
                    continue;
                };
                return_fact = return_fact.join(&dependency_summary.return_fact);
                if !matches!(dependency_summary.status, AdvisoryProductStatus::Complete) {
                    status = nonready_status(status);
                }
            }
            let parameters = node
                .parameters
                .joined_iter()
                .map(|(slot, fact)| (slot.clone(), fact.clone()))
                .collect::<Vec<_>>();
            let next = Arc::new(AdvisoryCallableSummary::new(
                callable.clone(),
                parameters,
                return_fact,
                node.summary.dependencies.to_vec(),
                node.summary.effects.clone(),
                status,
            ));
            let changed = summaries.get(&callable).is_none_or(|current| current.fingerprint != next.fingerprint);
            if changed {
                summaries.insert(callable.clone(), next);
                if let Some(dependents) = dependents.get(&callable) {
                    for dependent in dependents {
                        if queued.insert(dependent.clone()) {
                            queue.push_back(dependent.clone());
                        }
                    }
                }
            }
        }
        AdvisorySolverResult {
            summaries,
            status: AdvisoryProductStatus::Complete,
            steps,
            converged: true,
        }
    }
}

fn nonready_status(status: AdvisoryProductStatus) -> AdvisoryProductStatus {
    match status {
        AdvisoryProductStatus::Complete => AdvisoryProductStatus::Partial,
        other => other,
    }
}
