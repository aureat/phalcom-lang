//! Explanation nodes and steps (Spec 04.5).

use crate::identity::{BindingId, CallableId, ExplanationId, ExpressionId};
use crate::types::evidence::{EvidenceAuthority, TypeKnowledge};
use crate::types::id::TypeId;
use phalcom_common::range::SourceRange;

/// A formal step in explaining why a type was inferred, checked, or refuted.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ExplanationStep {
    /// Exact syntax / literal expression.
    Literal { expression: ExpressionId, ty: TypeId },
    /// Declared annotation on binding / signature.
    Declared {
        binding: Option<BindingId>,
        range: SourceRange,
        ty: TypeId,
    },
    /// Inferred from call return / method dispatch.
    MethodCall {
        call: ExpressionId,
        callable: CallableId,
        return_ty: TypeId,
    },
    /// Flow path refinement.
    FlowRefinement {
        binding: BindingId,
        prior: TypeKnowledge,
        refined: TypeKnowledge,
    },
    /// Control-flow branch join.
    BranchJoin {
        binding: BindingId,
        branches: Vec<TypeKnowledge>,
        joined: TypeKnowledge,
    },
    /// Subtyping assignability check.
    Subtyping { actual: TypeId, expected: TypeId, proven: bool },
}

/// An explanation node in the explanation arena.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExplanationNode {
    pub id: ExplanationId,
    pub step: ExplanationStep,
    pub authority: EvidenceAuthority,
    pub parents: Vec<ExplanationId>,
}
