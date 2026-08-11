//! Callable-summary data model; solving is added in the interprocedural slice.

use super::facts::InferredValue;
use super::ids::CallableId;

/// Summary of a source callable's inferred inputs and output.
#[derive(Clone, Debug)]
pub struct CallableSummary {
    /// Callable identity.
    pub callable: CallableId,
    /// Inferred parameter values.
    pub params: Vec<InferredValue>,
    /// Inferred return value.
    pub returns: InferredValue,
    /// Direct callable dependencies.
    pub dependencies: Vec<CallableId>,
}
