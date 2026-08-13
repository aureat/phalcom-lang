//! Callable-summary data model; solving is added in the interprocedural slice.

use std::collections::{BTreeMap, BTreeSet};

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
