//! Explanation nodes and steps (Spec 04.5).

use crate::checker::binding::BindingConsistency;
use crate::checker::expected::ExpectationOrigin;
use crate::diagnostic::{DiagnosticCode, SemanticSourceSpan};
use crate::dispatch::CallableSemanticKind;
use crate::identity::{BindingId, CallResolutionId, CallableId, DeclarationId, DiagnosticCauseId, ExplanationId, ExpressionId};
use crate::types::evidence::{DynamicReason, EvidenceOrigin, EvidenceStatus, TypeKnowledge, UnknownReason};
use crate::types::id::{TypeId, TypeParameterId};
use crate::types::outcome::{BlockReason, RelationOutcome};
use phalcom_common::range::SourceRange;

/// Structured derivation rule tag. Rules classify explanation nodes; the
/// semantic payload lives in [`ExplanationStep`] rather than being duplicated
/// as rule metadata.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DerivationRule {
    LiteralSynthesis,
    ExpressionSynthesis,
    NameResolution,
    AnnotationConstraint,
    BindingContract,
    TypeRequirement,
    TypeRelation,
    CallableSelection,
    CallableReturn,
    SelfSpecialization,
    ArgumentChecking,
    CallShape,
    GenericConstraint,
    GenericSolution,
    GenericConflict,
    CollectionSynthesis,
    ProductDecomposition,
    FlowRefinement { predicate_kind: PredicateKind },
    BranchJoin { branch_count: usize },
    LoopJoin,
    IterationElementResolution,
    AssignmentPropagation,
    ReturnTypeCheck,
    CallableReturnSummary,
    UnknownPropagation,
    DynamicPropagation,
    InternalBlocked,

    // Compatibility classifications retained while existing emitters migrate
    // to the normalized rule vocabulary above.
    MethodCallReturn { selector: String },
    GenericInstantiation { type_args: Vec<TypeId> },
    PolicyEnforcement { code: DiagnosticCode },
}

impl DerivationRule {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::LiteralSynthesis => "literal_synthesis",
            Self::ExpressionSynthesis => "expression_synthesis",
            Self::NameResolution => "name_resolution",
            Self::AnnotationConstraint => "annotation_constraint",
            Self::BindingContract => "binding_contract",
            Self::TypeRequirement => "type_requirement",
            Self::TypeRelation => "type_relation",
            Self::CallableSelection => "callable_selection",
            Self::CallableReturn => "callable_return",
            Self::SelfSpecialization => "self_specialization",
            Self::ArgumentChecking => "argument_checking",
            Self::CallShape => "call_shape",
            Self::GenericConstraint => "generic_constraint",
            Self::GenericSolution => "generic_solution",
            Self::GenericConflict => "generic_conflict",
            Self::CollectionSynthesis => "collection_synthesis",
            Self::ProductDecomposition => "product_decomposition",
            Self::FlowRefinement { .. } => "flow_refinement",
            Self::BranchJoin { .. } => "branch_join",
            Self::LoopJoin => "loop_join",
            Self::IterationElementResolution => "iteration_element_resolution",
            Self::AssignmentPropagation => "assignment_propagation",
            Self::ReturnTypeCheck => "return_type_check",
            Self::CallableReturnSummary => "callable_return_summary",
            Self::UnknownPropagation => "unknown_propagation",
            Self::DynamicPropagation => "dynamic_propagation",
            Self::InternalBlocked => "internal_blocked",
            Self::MethodCallReturn { .. } => "method_call_return",
            Self::GenericInstantiation { .. } => "generic_instantiation",
            Self::PolicyEnforcement { .. } => "policy_enforcement",
        }
    }
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
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
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

/// Stable explanation of why a call did not match its callable shape.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CallShapeExplanation {
    MissingRequired { parameter_index: u16, label: Option<String> },
    UnexpectedPositional { argument_index: u16 },
    UnknownLabel { label: String },
    DuplicateParameter { parameter_index: u16 },
}

/// Collection/product synthesis family recorded in the explanation graph.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CollectionKind {
    List,
    Set,
    Map,
    Tuple,
    Record,
}

/// Stable source of a generic constraint. Solver-local inference variables are
/// deliberately absent from this representation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GenericConstraintOrigin {
    Argument { parameter_index: u16 },
    ExpectedResult,
    Receiver,
    ExplicitTypeArgument,
    WhereClause,
}

/// Canonical relation contributed by a generic constraint after projection
/// onto callable-owned type parameters.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GenericConstraintRelation {
    Equal(TypeId),
    SubtypeOf(TypeId),
    SupertypeOf(TypeId),
    AssignableTo(TypeId),
}

/// A formal step in explaining why a type was inferred, checked, or refuted.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ExplanationStep {
    Literal {
        expression: ExpressionId,
        ty: TypeId,
    },
    ExpressionResult {
        expression: ExpressionId,
        knowledge: TypeKnowledge,
    },
    BindingRead {
        expression: ExpressionId,
        binding: BindingId,
        knowledge: TypeKnowledge,
    },
    Declared {
        binding: Option<BindingId>,
        range: SourceRange,
        ty: TypeId,
    },
    BindingContract {
        binding: BindingId,
        actual: TypeKnowledge,
        contract: TypeId,
        consistency: BindingConsistency,
    },
    TypeRequirement {
        expected: TypeId,
        origin: ExpectationOrigin,
        source: Option<SemanticSourceSpan>,
    },
    TypeRelation {
        actual: TypeKnowledge,
        expected: TypeId,
        outcome: RelationOutcome<()>,
    },
    CallableSelection {
        callable: CallableId,
        receiver: TypeId,
        declaring_owner: DeclarationId,
        specialization_path: Box<[DeclarationId]>,
    },
    CallableKind {
        callable: CallableId,
        kind: CallableSemanticKind,
    },
    CallableReturn {
        callable: CallableId,
        ty: TypeId,
    },
    SelfTypeSpecialization {
        self_ty: TypeId,
        receiver: TypeId,
        resolved: TypeId,
    },
    ArgumentCheck {
        call: ExpressionId,
        argument: ExpressionId,
        parameter_index: u16,
        actual: TypeKnowledge,
        expected: TypeId,
    },
    CallShape {
        callable: Option<CallableId>,
        failures: Box<[CallShapeExplanation]>,
    },
    MethodCall {
        call: ExpressionId,
        callable: CallableId,
        return_ty: TypeId,
    },
    /// A known call result whose callable identity was unavailable. This is a
    /// blocked explanation rather than a fabricated callable edge.
    UnresolvedCall {
        call: ExpressionId,
        return_ty: TypeId,
    },
    GenericConstraint {
        parameter: TypeParameterId,
        origin: GenericConstraintOrigin,
        relation: GenericConstraintRelation,
    },
    GenericSolution {
        parameter: TypeParameterId,
        ty: TypeId,
        status: EvidenceStatus,
    },
    GenericConflict {
        parameter: Option<TypeParameterId>,
        constraints: Box<[ExplanationId]>,
    },
    CollectionSynthesis {
        expression: ExpressionId,
        kind: CollectionKind,
        element_types: Box<[TypeId]>,
        result: TypeId,
    },
    ProductComponent {
        source: ExplanationId,
        index: usize,
        result: TypeKnowledge,
    },
    FlowRefinement {
        binding: BindingId,
        predicate: PredicateKind,
        prior: TypeKnowledge,
        refined: TypeKnowledge,
    },
    BranchJoin {
        binding: Option<BindingId>,
        branches: Box<[TypeKnowledge]>,
        reachable: Box<[bool]>,
        joined: TypeKnowledge,
    },
    LoopJoin {
        inputs: Box<[TypeKnowledge]>,
        joined: TypeKnowledge,
    },
    IterationElement {
        iterable: TypeKnowledge,
        element: TypeKnowledge,
        callable: Option<CallableId>,
    },
    ReturnCheck {
        actual: TypeKnowledge,
        expected: Option<TypeId>,
    },
    CallableReturnSummary {
        callable: CallableId,
        returns: Box<[TypeKnowledge]>,
        result: TypeKnowledge,
    },
    UnknownBoundary {
        reason: UnknownReason,
        source: Option<SourceRange>,
    },
    DynamicBoundary {
        reason: DynamicReason,
        source: Option<SourceRange>,
    },
    InternalBlocked {
        reason: BlockReason,
    },

    /// Compatibility-only relation node. New emitters must use
    /// [`ExplanationStep::TypeRelation`] so non-refutation terminal outcomes
    /// remain distinguishable.
    Subtyping {
        actual: TypeId,
        expected: TypeId,
        proven: bool,
    },
}

impl ExplanationStep {
    pub fn derivation_rule(&self) -> DerivationRule {
        match self {
            Self::Literal { .. } => DerivationRule::LiteralSynthesis,
            Self::ExpressionResult { .. } => DerivationRule::ExpressionSynthesis,
            Self::BindingRead { .. } => DerivationRule::NameResolution,
            Self::Declared { .. } => DerivationRule::AnnotationConstraint,
            Self::BindingContract { .. } => DerivationRule::BindingContract,
            Self::TypeRequirement { .. } => DerivationRule::TypeRequirement,
            Self::TypeRelation { .. } => DerivationRule::TypeRelation,
            Self::CallableSelection { .. } => DerivationRule::CallableSelection,
            Self::CallableKind { .. } => DerivationRule::CallableSelection,
            Self::CallableReturn { .. } => DerivationRule::CallableReturn,
            Self::SelfTypeSpecialization { .. } => DerivationRule::SelfSpecialization,
            Self::ArgumentCheck { .. } => DerivationRule::ArgumentChecking,
            Self::CallShape { .. } => DerivationRule::CallShape,
            Self::MethodCall { .. } => DerivationRule::CallableReturn,
            Self::UnresolvedCall { .. } => DerivationRule::InternalBlocked,
            Self::GenericConstraint { .. } => DerivationRule::GenericConstraint,
            Self::GenericSolution { .. } => DerivationRule::GenericSolution,
            Self::GenericConflict { .. } => DerivationRule::GenericConflict,
            Self::CollectionSynthesis { .. } => DerivationRule::CollectionSynthesis,
            Self::ProductComponent { .. } => DerivationRule::ProductDecomposition,
            Self::FlowRefinement { predicate, .. } => DerivationRule::FlowRefinement { predicate_kind: *predicate },
            Self::BranchJoin { branches, .. } => DerivationRule::BranchJoin { branch_count: branches.len() },
            Self::LoopJoin { .. } => DerivationRule::LoopJoin,
            Self::IterationElement { .. } => DerivationRule::IterationElementResolution,
            Self::ReturnCheck { .. } => DerivationRule::ReturnTypeCheck,
            Self::CallableReturnSummary { .. } => DerivationRule::CallableReturnSummary,
            Self::UnknownBoundary { .. } => DerivationRule::UnknownPropagation,
            Self::DynamicBoundary { .. } => DerivationRule::DynamicPropagation,
            Self::InternalBlocked { .. } => DerivationRule::InternalBlocked,
            Self::Subtyping { .. } => DerivationRule::TypeRelation,
        }
    }
}

/// An explanation node in the explanation arena.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExplanationNode {
    pub id: ExplanationId,
    pub step: ExplanationStep,
    pub rule: DerivationRule,
    pub status: EvidenceStatus,
    pub origin: EvidenceOrigin,
    pub evidence: Vec<EvidenceRef>,
    pub parents: Vec<ExplanationId>,
}
