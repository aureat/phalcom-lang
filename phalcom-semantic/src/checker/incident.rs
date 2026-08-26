//! Structured analyzer invariant incidents.

use crate::checker::binding::{BindingContract, BindingContractOrigin};
use crate::identity::{AnalysisIncidentId, BindingId, CallableId, ExpressionId, ModuleId};
use crate::types::id::TypeId;
use phalcom_common::range::SourceRange;

/// Operational policy applied after an internal incident has been recorded.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InternalFailurePolicy {
    /// Keep unrelated semantic products usable and contain the affected query.
    Contain,
    /// Let the enclosing test/developer harness fail after recording context.
    FailFast,
}

/// Semantic category of an analyzer invariant violation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum InternalSemanticIncidentKind {
    FlowInvariantViolation,
    RelationInvariantViolation,
    InferenceInvariantViolation,
    IdentityInvariantViolation,
    DatabaseInvariantViolation,
}

/// Compact, fingerprintable representation of a persistent binding contract.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BindingContractSummary {
    pub ty: Option<TypeId>,
    pub origin: Option<BindingContractOrigin>,
    pub source: Option<SourceRange>,
}

impl From<Option<&BindingContract>> for BindingContractSummary {
    fn from(contract: Option<&BindingContract>) -> Self {
        match contract {
            Some(contract) => Self {
                ty: Some(contract.ty),
                origin: Some(contract.origin),
                source: contract.source,
            },
            None => Self {
                ty: None,
                origin: None,
                source: None,
            },
        }
    }
}

/// Details needed to diagnose an internal semantic incident without making
/// it a source-owned diagnostic.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum InternalSemanticIncidentDetails {
    DivergentBindingContract {
        binding: BindingId,
        left: BindingContractSummary,
        right: BindingContractSummary,
    },
    DivergentMutability {
        binding: BindingId,
        left: bool,
        right: bool,
    },
    Message {
        message: Box<str>,
    },
}

/// Recorded invariant failure attached to one semantic analysis generation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InternalSemanticIncident {
    pub id: AnalysisIncidentId,
    pub kind: InternalSemanticIncidentKind,
    pub module: ModuleId,
    pub callable: Option<CallableId>,
    pub expression: Option<ExpressionId>,
    pub range: Option<SourceRange>,
    pub details: InternalSemanticIncidentDetails,
}
