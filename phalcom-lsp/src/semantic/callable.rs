//! Callable-summary data model; solving is added in the interprocedural slice.

use super::facts::InferredValue;
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

/// Conservative effect flags retained for future invalidation precision.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct SummaryEffects {
    /// Callable contains a reflective or dynamic send.
    pub dynamic_send: bool,
}
