//! Explanation nodes and steps (Spec 04.5).

use crate::diagnostic::DiagnosticCode;
use crate::identity::{BindingId, CallResolutionId, CallableId, DiagnosticCauseId, ExplanationId, ExpressionId};
use crate::types::evidence::{EvidenceAuthority, TypeKnowledge};
use crate::types::id::TypeId;
use phalcom_common::range::SourceRange;

/// Structured derivation rule tag — what logical step produced this node.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DerivationRule {
    LiteralSynthesis,
    AnnotationConstraint,
    MethodCallReturn { selector: String },
    GenericInstantiation { type_args: Vec<TypeId> },
    FlowRefinement { predicate_kind: PredicateKind },
    BranchJoin { branch_count: usize },
    PolicyEnforcement { code: DiagnosticCode },
    IterationElementResolution,
    AssignmentPropagation,
    ReturnTypeCheck,
    InternalBlocked { reason: String },
}

/// A reference to a piece of evidence: declared span, type ID, or call resolution.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EvidenceRef {
    SourceSpan(SourceRange),
    TypeId(TypeId),
    CallResolution(CallResolutionId),
    BindingVersion { binding: BindingId, version: u32 },
    Suppressed { cause: DiagnosticCauseId },
}

/// A `PredicateKind` tag for explanation classification.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PredicateKind {
    IsInstance,
    IsNotInstance,
    IsNil,
    NotNil,
    EqualLiteral,
    NotEqualLiteral,
    Ordered,
    Truthy,
    Falsy,
}

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

impl ExplanationStep {
    pub fn derivation_rule(&self) -> DerivationRule {
        match self {
            Self::Literal { .. } => DerivationRule::LiteralSynthesis,
            Self::Declared { .. } => DerivationRule::AnnotationConstraint,
            Self::MethodCall { callable, .. } => DerivationRule::MethodCallReturn {
                selector: format!("{}", callable.selector),
            },
            Self::FlowRefinement { .. } => DerivationRule::FlowRefinement {
                predicate_kind: PredicateKind::IsInstance,
            },
            Self::BranchJoin { branches, .. } => DerivationRule::BranchJoin { branch_count: branches.len() },
            Self::Subtyping { .. } => DerivationRule::PolicyEnforcement {
                code: DiagnosticCode::TypeMismatch,
            },
        }
    }
}

/// An explanation node in the explanation arena.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExplanationNode {
    pub id: ExplanationId,
    pub step: ExplanationStep,
    pub rule: DerivationRule,
    pub authority: EvidenceAuthority,
    pub evidence: Vec<EvidenceRef>,
    pub parents: Vec<ExplanationId>,
}
