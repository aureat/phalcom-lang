//! Callable-summary data model; solving is added in the interprocedural slice.

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use super::facts::{InferredValue, ParameterFacts};
use super::ids::CallableId;
use super::query::SemanticGeneration;

/// Summary of a source callable's inferred inputs and output.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CallableSummary {
    /// Callable identity.
    pub callable: CallableId,
    /// Inferred parameter values.
    pub params: Vec<InferredValue>,
    /// Inferred return value.
    pub returns: InferredValue,
    /// Direct callable dependencies.
    pub dependencies: Vec<CallableId>,
    /// Effects observed while extracting the summary.
    pub effects: SummaryEffects,
    /// Semantic generation that produced this summary.
    pub revision: SemanticGeneration,
}

/// Conservative callable effects used by flow propagation and invalidation.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct SummaryEffects {
    /// Callable contains a reflective or dynamic send.
    pub dynamic_send: bool,
    /// Zero-based parameter positions whose callable value is invoked on a
    /// reachable source path. Callers use this contract to propagate effects
    /// from literal blocks passed to higher-order callables.
    pub invokes_parameters: BTreeSet<usize>,
}

/// Coherent result of one complete callable/parameter solve.
#[derive(Clone, Debug, Default)]
pub(crate) struct SolverResult {
    /// Fixed-point callable summaries.
    pub summaries: BTreeMap<CallableId, CallableSummary>,
    /// Joined call-site parameter facts.
    pub parameter_facts: ParameterFacts,
}

/// Deduplicating callable work queue used by incremental solver rounds.
#[derive(Clone, Debug, Default)]
pub(crate) struct CallableWorklist {
    queue: VecDeque<CallableId>,
    queued: BTreeSet<CallableId>,
}

impl CallableWorklist {
    /// Adds a callable once until it is popped.
    pub(crate) fn push(&mut self, callable: CallableId) {
        if self.queued.insert(callable.clone()) {
            self.queue.push_back(callable);
        }
    }

    /// Pops the next dirty callable in deterministic insertion order.
    pub(crate) fn pop(&mut self) -> Option<CallableId> {
        let callable = self.queue.pop_front()?;
        self.queued.remove(&callable);
        Some(callable)
    }

    /// Returns whether no callable remains dirty.
    pub(crate) fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }
}
