//! Solver-local type inference session and term calculus (Spec 04.5).
//!
//! Law: InferVarId != TypeId. Inference variables are session-local reasoning entities,
//! never interned into canonical TypeStore or published in snapshots.

use super::context::CheckerControl;
use super::row_inference::{InferenceRecord, InferenceRecordField, InferenceRecordTail};
use crate::identity::{CallableId, DeclarationId, ExplanationId, ExpressionId, InferVarId};
use phalcom_common::range::SourceRange;
use crate::traits::AssociatedTypeRequirementId;
use crate::types::application::TypeApplicationError;
use crate::types::evidence::{DynamicReason, EvidenceStatus, TypeKnowledge, UnknownReason};
use crate::types::id::{AlgebraicStaticConstructorId, KindId, RigidTypeVariableId, StaticDomainId, TypeId, TypeParameterId, VariantTypeId};
use crate::types::kind::KindData;
use crate::types::outcome::{BlockReason, BudgetReport, CancellationToken, QueryBudget};
use crate::types::relation::{TypeHierarchy, is_subtype};
use crate::types::store::{CallableParameterType, CallableType, TypeData, TypeStore};
use crate::types::type_member::{ConstTerm, ProjectionSubject, TypeLevelProjection, TypeMemberArgument, TypeMemberDeclarationId, TypeMemberSelector};
use crate::types::type_lambda::{ScopedRecordTail, ScopedTypeData};
use crate::types::variance::Variance;
use crate::types::row::RecordRowField;
use crate::types::row_solver::{
    RecordRowShapeSolveResult, RecordRowSolution, RecordRowSolveResult, RecordRowSolver, RecordRowTerm, RecordRowTermTail, RecordRowVarId,
};
use crate::types::{FamilyMemberType, FamilyMemberTypeKind, FamilyOperationShape, RecordRowTail, TupleTypeElement};
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::ops::{Deref, DerefMut};
use std::rc::{Rc, Weak};

/// Solver-local representation of a trait application in an associated projection.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InferenceTraitRef {
    pub declaration: DeclarationId,
    pub arguments: Box<[InferenceTerm]>,
}

/// A local inference term representing a canonical type, a solver variable, or a compound inference form.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum InferenceTerm {
    Canonical(TypeId),
    Var(InferVarId),
    /// Opaque branch-local rigid. Flexible variables may be solved to this
    /// term, but no solver path may assign the rigid itself.
    Rigid(RigidTypeVariableId),
    Applied {
        origin: Box<InferenceTerm>,
        arguments: Box<[InferenceStaticArgument]>,
    },
    /// Structural static-data constructor term. Concrete terms may remain
    /// canonical, but symbolic children must stay visible to first-order
    /// inference until they are materialized through `TypeStore`.
    StaticData {
        domain: StaticDomainId,
        constructor: AlgebraicStaticConstructorId,
        arguments: Box<[InferenceStaticArgument]>,
    },
    ExactCase {
        variant: VariantTypeId,
        enum_type: Box<InferenceTerm>,
    },
    Union(Box<[InferenceTerm]>),
    Tuple(Box<[InferenceTupleElement]>),
    Callable(InferenceCallable),
    Record(InferenceRecord),
    Family(InferenceFamily),
    AssociatedProjection {
        subject: Box<InferenceTerm>,
        trait_ref: InferenceTraitRef,
        requirement: AssociatedTypeRequirementId,
    },
    /// A source/domain-owned type-member operation whose symbolic children
    /// must remain visible until query-local substitutions are applied.
    /// Projection equality is never used to infer its subject or arguments.
    TypeMemberProjection {
        subject: InferenceProjectionSubject,
        selector: TypeMemberSelector,
        arguments: Box<[InferenceTypeMemberArgument]>,
        source_declaration: Option<TypeMemberDeclarationId>,
        result_kind: KindId,
    },
    TypePack(Box<[InferenceTerm]>),
    LabeledTypePack(Box<[InferenceLabeledTypePackElement]>),
    CompleteTypePack {
        positionals: Box<InferenceTerm>,
        labeled: Box<InferenceTerm>,
    },
    TypePackConcat(Box<[InferenceTerm]>),
    LabeledTypePackConcat(Box<[InferenceTerm]>),
    TupleFromPacks {
        positionals: Box<InferenceTerm>,
        labeled: Box<InferenceTerm>,
    },
    CallableFromPacks {
        positionals: Box<InferenceTerm>,
        labeled: Box<InferenceTerm>,
        return_type: Box<InferenceTerm>,
    },
    PackProjection {
        subject: Box<InferenceTerm>,
        lane: InferencePackProjectionLane,
    },
}

/// One heterogeneous static argument in a solver-local applied term. Row
/// arguments retain their own identity space and are routed by the query
/// inference coordinator instead of being coerced into ordinary type terms.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum InferenceStaticArgument {
    Term(InferenceTerm),
    RecordRow(InferenceRecord),
}

/// Subject lane for a solver-local type-member projection.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum InferenceProjectionSubject {
    Type(Box<InferenceTerm>),
    RecordRow(InferenceRecord),
}

/// Argument lane for a solver-local type-member projection.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum InferenceTypeMemberArgument {
    Type(InferenceTerm),
    Const(ConstTerm),
    RecordRow(InferenceRecord),
}

impl InferenceStaticArgument {
    pub fn term(&self) -> Option<&InferenceTerm> {
        match self {
            Self::Term(term) => Some(term),
            Self::RecordRow(_) => None,
        }
    }
}

fn static_terms(arguments: &[InferenceStaticArgument]) -> Option<Vec<InferenceTerm>> {
    arguments.iter().map(|argument| argument.term().cloned()).collect()
}

fn inference_arguments_from_terms(arguments: &[InferenceTerm]) -> Box<[InferenceStaticArgument]> {
    arguments.iter().cloned().map(InferenceStaticArgument::Term).collect::<Vec<_>>().into_boxed_slice()
}

/// A labeled element inside a solver-local labeled type pack.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InferenceLabeledTypePackElement {
    pub label: Box<str>,
    pub term: InferenceTerm,
}

/// Lane selector for a solver-local pack projection.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum InferencePackProjectionLane {
    Positionals,
    Labeled,
    Complete,
}

/// Query-local owner for solver variables shared by nested generic applications.
///
/// This identity is intentionally not part of durable semantic products.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct InferenceContextId(pub(crate) u32);

/// Application frame inside one query-local inference context.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct InferenceFrameId(pub u32);

#[derive(Clone, Debug)]
struct InferenceFrame {
    parent: Option<InferenceFrameId>,
    closed: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InferenceTupleElement {
    pub label: Option<Box<str>>,
    pub term: InferenceTerm,
}

use phalcom_ast::ast::RestMode;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InferenceCallableParameter {
    pub label: Option<Box<str>>,
    pub term: InferenceTerm,
    pub rest: RestMode,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InferenceCallable {
    pub parameters: Box<[InferenceCallableParameter]>,
    pub return_type: Box<InferenceTerm>,
}

/// Solver-local associated-family member with a recursively lifted type term.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InferenceFamilyMember {
    pub operation: FamilyOperationShape,
    pub member_kind: FamilyMemberTypeKind,
    pub term: InferenceTerm,
}

pub type InferenceFamily = Box<[InferenceFamilyMember]>;

/// State of an inference variable in a session.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum InferVarState {
    Unsolved,
    Solved(TypeId),
    Failed(InferenceFailureReason),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum InferenceFailureReason {
    OccursCheck { var: InferVarId },
    KindMismatch { var: InferVarId, expected: KindId, actual: KindId },
    ConflictingBounds { var: InferVarId, lower: TypeId, upper: TypeId },
    MissingVariableMetadata { var: InferVarId },
    StructuralMismatch { left: Box<InferenceTerm>, right: Box<InferenceTerm> },
    RowRejected(crate::types::row_solver::RecordRowFailure<InferenceTerm>),
    UnresolvedSelf,
    InternalRowSolverFailure { incident: u64 },
}

/// Description of an inference variable.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InferenceVariable {
    pub id: InferVarId,
    pub(crate) frame: InferenceFrameId,
    pub kind: KindId,
    pub state: InferVarState,
    pub support: Option<InferenceSupport>,
    /// Complete proof state for value premises that can influence this variable.
    pub proof: Option<InferenceProofState>,
}

/// Value evidence supporting an inferred generic variable.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum InferenceSupport {
    Established,
    Assumed,
}

impl InferenceSupport {
    fn join(self, other: Self) -> Self {
        if matches!(self, Self::Assumed) || matches!(other, Self::Assumed) {
            Self::Assumed
        } else {
            Self::Established
        }
    }
}

/// Proof state for required value premises participating in generic inference.
///
/// This domain is intentionally separate from solver support: a solved
/// substitution can still depend on an unavailable or dynamic source value.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum InferenceProofState {
    Established,
    Assumed,
    Unknown(UnknownReason),
    Dynamic(DynamicReason),
}

impl InferenceProofState {
    /// Converts complete expression knowledge into solver-local proof state.
    pub fn from_knowledge(knowledge: &TypeKnowledge) -> Self {
        match knowledge {
            TypeKnowledge::Known(evidence) => match evidence.status() {
                EvidenceStatus::Established => Self::Established,
                EvidenceStatus::Assumed => Self::Assumed,
            },
            TypeKnowledge::Unknown(reason) => Self::Unknown(reason.clone()),
            TypeKnowledge::Dynamic(reason) => Self::Dynamic(reason.clone()),
        }
    }

    /// Meets required premises without discarding the weakest epistemic state.
    pub fn meet(self, other: Self) -> Self {
        use InferenceProofState::{Assumed, Dynamic, Established, Unknown};

        match (self, other) {
            (Unknown(left), Unknown(right)) => Unknown(crate::types::evidence::join_unknown_reason(left, right)),
            (Unknown(reason), _) | (_, Unknown(reason)) => Unknown(reason),
            (Dynamic(left), Dynamic(right)) => Dynamic(crate::types::evidence::join_dynamic_reason(left, right)),
            (Dynamic(reason), _) | (_, Dynamic(reason)) => Dynamic(reason),
            (Assumed, _) | (_, Assumed) => Assumed,
            (Established, Established) => Established,
        }
    }
}

/// A required generic value premise, retained even when no canonical type can
/// be extracted from its expression knowledge.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RequiredInferencePremise {
    pub term: InferenceTerm,
    pub origin: ConstraintOrigin,
    pub proof: InferenceProofState,
    pub explanation: Option<ExplanationId>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct InferenceSubtypeEdge {
    sub: InferVarId,
    sup: InferVarId,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct PendingInferenceRowEquation {
    left: InferenceRecord,
    right: InferenceRecord,
    diagnostic_left: InferenceTerm,
    diagnostic_right: InferenceTerm,
}

fn record_term_for_row_solver(record: &InferenceRecord) -> RecordRowTerm<InferenceTerm> {
    RecordRowTerm {
        fields: record
            .fields
            .iter()
            .map(|field| RecordRowField {
                name: field.name.clone(),
                ty: field.term.clone(),
            })
            .collect::<Vec<_>>()
            .into_boxed_slice(),
        tail: match record.tail {
            InferenceRecordTail::Closed => RecordRowTermTail::Closed,
            InferenceRecordTail::Parameter(parameter) => RecordRowTermTail::Parameter(parameter),
            InferenceRecordTail::Projection(projection) => RecordRowTermTail::Projection(projection),
            InferenceRecordTail::Var(variable) => RecordRowTermTail::Var(variable),
        },
    }
}

fn inference_term_contains_row_variable(
    term: &InferenceTerm,
    target: RecordRowVarId,
    row_solver: &RecordRowSolver<InferenceTerm>,
    var_terms: &HashMap<InferVarId, InferenceTerm>,
    aliases: &HashMap<InferVarId, InferVarId>,
    visiting: &mut HashSet<InferVarId>,
) -> bool {
    macro_rules! visit {
        ($nested:expr) => {
            inference_term_contains_row_variable($nested, target, row_solver, var_terms, aliases, visiting)
        };
    }
    match term {
        InferenceTerm::Var(variable) => {
            let mut representative = *variable;
            while let Some(next) = aliases.get(&representative).copied() {
                if next == representative {
                    break;
                }
                representative = next;
            }
            if !visiting.insert(representative) {
                return false;
            }
            let contains = var_terms.get(&representative).or_else(|| var_terms.get(variable)).is_some_and(|nested| visit!(nested));
            visiting.remove(&representative);
            contains
        }
        InferenceTerm::Applied { origin, arguments } => {
            visit!(origin) || arguments.iter().any(|argument| inference_static_argument_contains_row_variable(argument, target, row_solver, var_terms, aliases, visiting))
        }
        InferenceTerm::StaticData { arguments, .. } => arguments
            .iter()
            .any(|argument| inference_static_argument_contains_row_variable(argument, target, row_solver, var_terms, aliases, visiting)),
        InferenceTerm::ExactCase { enum_type, .. } => visit!(enum_type),
        InferenceTerm::Union(members) => members.iter().any(|nested| visit!(nested)),
        InferenceTerm::Tuple(elements) => elements.iter().any(|element| visit!(&element.term)),
        InferenceTerm::Callable(callable) => {
            callable.parameters.iter().any(|parameter| visit!(&parameter.term)) || visit!(&callable.return_type)
        }
        InferenceTerm::Record(record) => inference_record_contains_row_variable(record, target, row_solver, var_terms, aliases, visiting),
        InferenceTerm::Family(family) => family.iter().any(|member| visit!(&member.term)),
        InferenceTerm::AssociatedProjection { subject, trait_ref, .. } => {
            visit!(subject) || trait_ref.arguments.iter().any(|nested| visit!(nested))
        }
        InferenceTerm::TypeMemberProjection { subject, arguments, .. } => {
            let subject_contains = match subject {
                InferenceProjectionSubject::Type(term) => visit!(term),
                InferenceProjectionSubject::RecordRow(row) => {
                    inference_record_contains_row_variable(row, target, row_solver, var_terms, aliases, visiting)
                }
            };
            subject_contains
                || arguments.iter().any(|argument| match argument {
                    InferenceTypeMemberArgument::Type(term) => visit!(term),
                    InferenceTypeMemberArgument::Const(_) => false,
                    InferenceTypeMemberArgument::RecordRow(row) => {
                        inference_record_contains_row_variable(row, target, row_solver, var_terms, aliases, visiting)
                    }
                })
        }
        InferenceTerm::TypePack(elements) => elements.iter().any(|nested| visit!(nested)),
        InferenceTerm::LabeledTypePack(elements) => elements.iter().any(|element| visit!(&element.term)),
        InferenceTerm::CompleteTypePack { positionals, labeled } => visit!(positionals) || visit!(labeled),
        InferenceTerm::TypePackConcat(segments) | InferenceTerm::LabeledTypePackConcat(segments) => segments.iter().any(|nested| visit!(nested)),
        InferenceTerm::TupleFromPacks { positionals, labeled } => visit!(positionals) || visit!(labeled),
        InferenceTerm::CallableFromPacks { positionals, labeled, return_type } => {
            visit!(positionals) || visit!(labeled) || visit!(return_type)
        }
        InferenceTerm::PackProjection { subject, .. } => visit!(subject),
        InferenceTerm::Canonical(_) | InferenceTerm::Rigid(_) => false,
    }
}

fn inference_static_argument_contains_row_variable(
    argument: &InferenceStaticArgument,
    target: RecordRowVarId,
    row_solver: &RecordRowSolver<InferenceTerm>,
    var_terms: &HashMap<InferVarId, InferenceTerm>,
    aliases: &HashMap<InferVarId, InferVarId>,
    visiting: &mut HashSet<InferVarId>,
) -> bool {
    match argument {
        InferenceStaticArgument::Term(term) => inference_term_contains_row_variable(term, target, row_solver, var_terms, aliases, visiting),
        InferenceStaticArgument::RecordRow(record) => {
            inference_record_contains_row_variable(record, target, row_solver, var_terms, aliases, visiting)
        }
    }
}

fn inference_record_contains_row_variable(
    record: &InferenceRecord,
    target: RecordRowVarId,
    row_solver: &RecordRowSolver<InferenceTerm>,
    var_terms: &HashMap<InferVarId, InferenceTerm>,
    aliases: &HashMap<InferVarId, InferVarId>,
    visiting: &mut HashSet<InferVarId>,
) -> bool {
    let normalized = row_solver.normalize_term(&record_term_for_row_solver(record));
    matches!(normalized.tail, RecordRowTermTail::Var(variable) if variable == target)
        || normalized.fields.iter().any(|field| {
            inference_term_contains_row_variable(&field.ty, target, row_solver, var_terms, aliases, visiting)
        })
}

/// Kind of constraint relation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum InferenceRelation {
    Equivalent(InferenceTerm, InferenceTerm),
    Subtype(InferenceTerm, InferenceTerm),
}

/// The origin of an inference constraint for causality and explanation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ConstraintOrigin {
    DataExpansionArgument {
        call: ExpressionId,
        argument: SourceRange,
        parameter: TypeParameterId,
    },
    Argument {
        call: ExpressionId,
        argument: ExpressionId,
        parameter_index: u16,
    },
    ExpectedResult {
        expression: ExpressionId,
    },
    BlockParameter {
        block: ExpressionId,
        parameter_index: u16,
    },
    BlockResult {
        block: ExpressionId,
    },
    CollectionElement {
        literal: ExpressionId,
    },
    GenericWhere {
        callable: CallableId,
        constraint_index: u16,
    },
    Explicit,
}

/// Semantic role of a constraint. Selection narrows candidates; admissibility
/// and declaration restrictions validate a selected candidate without creating
/// value evidence by themselves.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InferenceConstraintRole {
    ValueSelection,
    ContextSelection,
    ExactSemanticSelection,
    DeclarationRestriction,
}

/// Structured constraint tracked in an inference session.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InferenceConstraint {
    pub relation: InferenceRelation,
    pub origin: ConstraintOrigin,
    pub explanation: Option<ExplanationId>,
    pub support: Option<InferenceSupport>,
    pub role: InferenceConstraintRole,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InferenceSolution {
    pub substitutions: HashMap<InferVarId, TypeId>,
    pub support: HashMap<InferVarId, InferenceSupport>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UnderconstrainedInference {
    pub unsolved_vars: Vec<InferVarId>,
}

/// Structured failure while turning a solver term into a canonical type.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum InferenceMaterializationFailure {
    Unsolved(UnderconstrainedInference),
    Rigid(RigidTypeVariableId),
    TypeApplication(TypeApplicationError),
    InvalidExactCase,
    UnsupportedDomain,
    Pack(crate::types::store::PackFormationError),
    InternalInvariant,
}

impl From<UnderconstrainedInference> for InferenceMaterializationFailure {
    fn from(value: UnderconstrainedInference) -> Self {
        Self::Unsolved(value)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InferenceConflict {
    /// Deterministic bounded causal constraint set. The legacy single index is
    /// retained during v1 migration for callers that only need the failing edge.
    pub constraint_indices: Box<[u32]>,
    pub constraint_index: Option<u32>,
    pub origin: Option<ConstraintOrigin>,
    pub failure: InferenceFailureReason,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SolveEffect {
    Unchanged,
    Changed,
}

/// A required relation that could not yet be proved or refuted because a
/// non-injective projection remained symbolic.
///
/// Residual obligations are retried by the existing inference fixed point
/// after substitutions and projection normalization. They are never used to
/// infer projection subjects backwards.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResidualInferenceObligation {
    pub constraint_index: u32,
    pub relation: InferenceRelation,
    pub origin: ConstraintOrigin,
}

impl SolveEffect {
    fn is_changed(self) -> bool {
        matches!(self, Self::Changed)
    }
}

/// Outcome of solving an inference session.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum InferenceOutcome {
    Solved(InferenceSolution),
    Underconstrained(UnderconstrainedInference),
    Ambiguous(AmbiguousInference),
    Conflicting(InferenceConflict),
    Blocked(BlockReason),
    Cancelled,
    BudgetExceeded(BudgetReport),
    InternalFailure(InferenceFailureReason),
}

/// One admissible substitution in a finite ambiguity product.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InferenceCandidate {
    pub variable: InferVarId,
    pub ty: TypeId,
}

/// A finite set of incomparable substitutions that all satisfy inference constraints.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AmbiguousInference {
    pub variables: Box<[InferVarId]>,
    pub candidates: Box<[InferenceCandidate]>,
    pub constraint_indices: Box<[u32]>,
}

impl InferenceOutcome {
    pub fn is_solved(&self) -> bool {
        matches!(self, Self::Solved(_))
    }
}

use crate::types::parameter::{GenericSignature, TypeTerm};

/// Solver-local inference session.
#[derive(Clone, Debug, Default)]
pub struct InferenceSession {
    variables: Vec<InferenceVariable>,
    constraints: Vec<InferenceConstraint>,
    substitutions: HashMap<InferVarId, TypeId>,
    lower_bounds: HashMap<InferVarId, Vec<TypeId>>,
    upper_bounds: HashMap<InferVarId, Vec<TypeId>>,
    var_aliases: HashMap<InferVarId, InferVarId>,
    var_terms: HashMap<InferVarId, InferenceTerm>,
    required_premises: Vec<RequiredInferencePremise>,
    subtype_edges: Vec<InferenceSubtypeEdge>,
    bound_origins: HashMap<(InferVarId, TypeId), (u32, ConstraintOrigin)>,
    bound_roles: HashMap<(InferVarId, TypeId), Vec<InferenceConstraintRole>>,
    variable_constraint_indices: HashMap<InferVarId, Vec<u32>>,
    next_var_index: u32,
    frames: Vec<InferenceFrame>,
    root_frame: Option<InferenceFrameId>,
    next_frame_index: u32,
    /// Non-owning access for structural term traversal and materialization.
    /// `InferenceCoordinator` owns the row solver alongside this session.
    row_solver: Weak<RefCell<RecordRowSolver<InferenceTerm>>>,
    pending_row_equations: Vec<PendingInferenceRowEquation>,
    queued_row_equations: Vec<PendingInferenceRowEquation>,
    /// Mandatory constraints that are currently neither proven nor refuted.
    /// The key is the stable index in `constraints`.
    residual_obligations: HashMap<u32, ResidualInferenceObligation>,
    /// Constraint currently being evaluated. Projection unification uses this
    /// only to attach a residual proof obligation to its owning relation.
    active_constraint_index: Option<u32>,
}

#[derive(Clone, Debug)]
struct CanonicalConstructorView {
    actual: TypeId,
    origin: TypeId,
    arguments: Box<[TypeId]>,
    origin_kind: KindId,
}

impl InferenceSession {
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the currently unresolved mandatory inference obligations.
    ///
    /// This is primarily useful for diagnostics and law-level solver tests;
    /// callers must not treat a residual as a solved relation.
    pub fn residual_obligations(&self) -> Vec<ResidualInferenceObligation> {
        let mut obligations = self.residual_obligations.values().cloned().collect::<Vec<_>>();
        obligations.sort_by_key(|obligation| obligation.constraint_index);
        obligations
    }

    fn defer_active_constraint(&mut self) {
        let Some(index) = self.active_constraint_index else {
            return;
        };
        let Some(constraint) = self.constraints.get(index as usize).cloned() else {
            return;
        };
        self.residual_obligations.insert(
            index,
            ResidualInferenceObligation {
                constraint_index: index,
                relation: constraint.relation,
                origin: constraint.origin,
            },
        );
    }

    fn residual_blocks_frame(&self, frame: InferenceFrameId) -> bool {
        self.residual_obligations.values().any(|obligation| {
            let terms = match &obligation.relation {
                InferenceRelation::Equivalent(left, right) | InferenceRelation::Subtype(left, right) => [left, right],
            };
            let mut contains_variable = false;
            for term in terms {
                for variable in self.term_variables(term) {
                    contains_variable = true;
                    if self
                        .variables
                        .iter()
                        .find(|candidate| candidate.id == variable)
                        .is_some_and(|candidate| candidate.frame == frame)
                    {
                        return true;
                    }
                }
            }
            // A closed symbolic projection relation has no inference variable
            // to attribute to a child frame. It is owned by the root proof.
            !contains_variable && self.root_frame == Some(frame)
        })
    }

    fn enqueue_row_equation(
        &mut self,
        left: &InferenceRecord,
        right: &InferenceRecord,
        diagnostic_left: &InferenceTerm,
        diagnostic_right: &InferenceTerm,
    ) -> SolveEffect {
        let equation = PendingInferenceRowEquation {
            left: left.clone(),
            right: right.clone(),
            diagnostic_left: diagnostic_left.clone(),
            diagnostic_right: diagnostic_right.clone(),
        };
        if self.queued_row_equations.iter().any(|queued| queued.left == equation.left && queued.right == equation.right) {
            return SolveEffect::Unchanged;
        }
        self.queued_row_equations.push(equation.clone());
        self.pending_row_equations.push(equation);
        SolveEffect::Changed
    }

    pub(crate) fn enqueue_record_row_equation(&mut self, left: &InferenceRecord, right: &InferenceRecord) -> SolveEffect {
        let diagnostic_left = InferenceTerm::Record(left.clone());
        let diagnostic_right = InferenceTerm::Record(right.clone());
        self.enqueue_row_equation(left, right, &diagnostic_left, &diagnostic_right)
    }

    fn take_pending_row_equations(&mut self) -> Vec<PendingInferenceRowEquation> {
        std::mem::take(&mut self.pending_row_equations)
    }

    /// Returns the root application frame, creating it for standalone callers
    /// that use an inference session without a [`CheckingContext`].
    pub fn root_frame(&mut self) -> InferenceFrameId {
        if let Some(frame) = self.root_frame {
            return frame;
        }
        let frame = self.begin_frame(None);
        self.root_frame = Some(frame);
        frame
    }

    /// Begins a nested application frame in this session's variable space.
    pub fn begin_frame(&mut self, parent: Option<InferenceFrameId>) -> InferenceFrameId {
        let frame = InferenceFrameId(self.next_frame_index);
        self.next_frame_index = self.next_frame_index.saturating_add(1);
        self.frames.push(InferenceFrame { parent, closed: false });
        frame
    }

    /// Marks an application frame closed after its result has been consumed.
    /// Variables remain in the graph until the owning query context is dropped.
    pub(crate) fn close_frame(&mut self, frame: InferenceFrameId) {
        let index = frame.0 as usize;
        if let Some(metadata) = self.frames.get_mut(index) {
            metadata.closed = true;
        }
    }

    pub(crate) fn frame_is_closed(&self, frame: InferenceFrameId) -> bool {
        self.frames.get(frame.0 as usize).is_some_and(|metadata| metadata.closed)
    }

    pub(crate) fn frame_parent(&self, frame: InferenceFrameId) -> Option<InferenceFrameId> {
        self.frames.get(frame.0 as usize).and_then(|metadata| metadata.parent)
    }

    /// Allocates a fresh inference variable with the given kind.
    pub fn fresh_variable(&mut self, kind: KindId) -> InferVarId {
        let frame = self.root_frame();
        self.fresh_variable_in_frame(frame, kind)
    }

    /// Allocates an inference variable with explicit value support.
    pub fn fresh_variable_with_support(&mut self, kind: KindId, support: Option<InferenceSupport>) -> InferVarId {
        let frame = self.root_frame();
        self.fresh_variable_with_support_in_frame(frame, kind, support)
    }

    pub fn fresh_variable_in_frame(&mut self, frame: InferenceFrameId, kind: KindId) -> InferVarId {
        self.fresh_variable_with_support_in_frame(frame, kind, None)
    }

    pub(crate) fn fresh_variable_with_support_in_frame(&mut self, frame: InferenceFrameId, kind: KindId, support: Option<InferenceSupport>) -> InferVarId {
        let var = InferVarId::from_index(self.next_var_index as usize);
        self.next_var_index += 1;
        self.variables.push(InferenceVariable {
            id: var,
            frame,
            kind,
            state: InferVarState::Unsolved,
            support,
            proof: None,
        });
        var
    }

    /// Finds the canonical representative variable for `v`.
    pub fn find_var(&self, mut v: InferVarId) -> InferVarId {
        while let Some(&next) = self.var_aliases.get(&v) {
            if next == v {
                break;
            }
            v = next;
        }
        v
    }

    pub fn solved_type_for(&self, solution: &InferenceSolution, variable: InferVarId) -> Option<TypeId> {
        solution.substitutions.get(&self.find_var(variable)).copied()
    }

    /// Adds a constraint to the session.
    pub fn add_constraint(&mut self, relation: InferenceRelation, origin: ConstraintOrigin, explanation: Option<ExplanationId>) {
        let role = Self::role_for_origin(&origin);
        self.constraints.push(InferenceConstraint {
            relation,
            origin,
            explanation,
            support: None,
            role,
        });
    }

    /// Adds a relation with explicit selection/admissibility provenance.
    pub fn add_constraint_with_role(
        &mut self,
        relation: InferenceRelation,
        origin: ConstraintOrigin,
        explanation: Option<ExplanationId>,
        role: InferenceConstraintRole,
    ) {
        self.constraints.push(InferenceConstraint {
            relation,
            origin,
            explanation,
            support: None,
            role,
        });
    }

    /// Adds a constraint and records value support for any variables it reaches.
    pub fn add_constraint_with_support(
        &mut self,
        relation: InferenceRelation,
        origin: ConstraintOrigin,
        explanation: Option<ExplanationId>,
        support: InferenceSupport,
    ) {
        let role = Self::role_for_origin(&origin);
        self.record_relation_support(&relation, support);
        self.constraints.push(InferenceConstraint {
            relation,
            origin,
            explanation,
            support: Some(support),
            role,
        });
    }

    fn role_for_origin(origin: &ConstraintOrigin) -> InferenceConstraintRole {
        match origin {
            ConstraintOrigin::ExpectedResult { .. } => InferenceConstraintRole::ContextSelection,
            ConstraintOrigin::GenericWhere { .. } => InferenceConstraintRole::DeclarationRestriction,
            ConstraintOrigin::Explicit => InferenceConstraintRole::ExactSemanticSelection,
            ConstraintOrigin::Argument { .. }
            | ConstraintOrigin::DataExpansionArgument { .. }
            | ConstraintOrigin::BlockParameter { .. }
            | ConstraintOrigin::BlockResult { .. }
            | ConstraintOrigin::CollectionElement { .. } => InferenceConstraintRole::ValueSelection,
        }
    }

    /// Records one generic argument as a proof premise before any type-id
    /// filtering occurs.
    pub fn record_required_premise(&mut self, term: &InferenceTerm, origin: ConstraintOrigin, knowledge: &TypeKnowledge, explanation: Option<ExplanationId>) {
        let proof = InferenceProofState::from_knowledge(knowledge);
        self.record_term_proof_state(term, proof.clone());
        self.required_premises.push(RequiredInferencePremise {
            term: term.clone(),
            origin,
            proof,
            explanation,
        });
    }

    /// Records that an expected type selected the variables in `term`.
    /// Contextual selection is weaker than value evidence, but it must still
    /// make an otherwise solvable result publishable. Existing unknown or
    /// dynamic premises remain weakest through the proof-state meet.
    pub fn record_context_selection(&mut self, term: &InferenceTerm) {
        self.record_term_proof_state(term, InferenceProofState::Assumed);
    }

    /// Returns proof state for all inference variables occurring in a term.
    pub fn proof_state_for_term(&self, term: &InferenceTerm) -> InferenceProofState {
        let mut variables = self.term_variables(term).into_iter();
        let Some(first) = variables.next() else {
            return InferenceProofState::Established;
        };
        let mut proof = self
            .variable_by_representative(first)
            .and_then(|variable| variable.proof.clone())
            .unwrap_or(InferenceProofState::Unknown(UnknownReason::UnderconstrainedTypeVariable));
        for variable in variables {
            let current = self
                .variable_by_representative(variable)
                .and_then(|candidate| candidate.proof.clone())
                .unwrap_or(InferenceProofState::Unknown(UnknownReason::UnderconstrainedTypeVariable));
            proof = proof.meet(current);
        }
        proof
    }

    pub fn projected_parameters_for_term(&self, term: &InferenceTerm, parameters: &HashMap<TypeParameterId, InferenceTerm>) -> Vec<TypeParameterId> {
        let variables = self.term_variables(term);
        let mut result: Vec<_> = parameters
            .iter()
            .filter_map(|(parameter, mapped)| match mapped {
                InferenceTerm::Var(variable) if variables.contains(&self.find_var(*variable)) => Some(*parameter),
                _ => None,
            })
            .collect();
        result.sort_by_key(|parameter| parameter.index());
        result
    }

    pub fn constraint_explanation_roots(&self, indices: &[u32]) -> Vec<ExplanationId> {
        let mut roots = indices
            .iter()
            .filter_map(|index| self.constraints.get(*index as usize).and_then(|constraint| constraint.explanation))
            .collect::<Vec<_>>();
        roots.sort_by_key(|id| id.0);
        roots.dedup();
        roots
    }

    pub fn constraint_origin(&self, index: u32) -> Option<&ConstraintOrigin> {
        self.constraints.get(index as usize).map(|constraint| &constraint.origin)
    }

    pub fn constraint_role(&self, index: u32) -> Option<InferenceConstraintRole> {
        self.constraints.get(index as usize).map(|constraint| constraint.role)
    }

    pub fn parameter_for_variable(&self, variable: InferVarId, parameters: &HashMap<TypeParameterId, InferenceTerm>) -> Option<TypeParameterId> {
        let representative = self.find_var(variable);
        let mut candidates = parameters
            .iter()
            .filter_map(|(parameter, term)| match term {
                InferenceTerm::Var(candidate) if self.find_var(*candidate) == representative => Some(*parameter),
                _ => None,
            })
            .collect::<Vec<_>>();
        candidates.sort_by_key(|parameter| parameter.index());
        candidates.into_iter().next()
    }

    pub fn all_constraint_explanation_roots(&self) -> Vec<ExplanationId> {
        let mut roots = self.constraints.iter().filter_map(|constraint| constraint.explanation).collect::<Vec<_>>();
        roots.sort_by_key(|id| id.0);
        roots.dedup();
        roots
    }

    pub fn projected_solution(
        &mut self,
        parameter: TypeParameterId,
        parameters: &HashMap<TypeParameterId, InferenceTerm>,
        store: &mut TypeStore,
    ) -> Option<TypeId> {
        let term = parameters.get(&parameter)?;
        self.materialize(term, store).ok()
    }

    fn variable_by_representative(&self, variable: InferVarId) -> Option<&InferenceVariable> {
        let representative = self.find_var(variable);
        self.variables.iter().find(|candidate| candidate.id == representative)
    }

    /// Instantiates fresh inference variables for each generic parameter in `generic_sig`.
    pub fn instantiate_generic_signature(&mut self, generic_sig: &GenericSignature, store: &TypeStore) -> HashMap<TypeParameterId, InferenceTerm> {
        let frame = self.root_frame();
        self.instantiate_generic_signature_in_frame(generic_sig, store, frame)
    }

    pub(crate) fn instantiate_generic_signature_in_frame(
        &mut self,
        generic_sig: &GenericSignature,
        store: &TypeStore,
        frame: InferenceFrameId,
    ) -> HashMap<TypeParameterId, InferenceTerm> {
        self.instantiate_generic_signature_in_frame_excluding(generic_sig, store, frame, &HashSet::new())
    }

    pub(crate) fn instantiate_generic_signature_in_frame_excluding(
        &mut self,
        generic_sig: &GenericSignature,
        store: &TypeStore,
        frame: InferenceFrameId,
        excluded: &HashSet<TypeParameterId>,
    ) -> HashMap<TypeParameterId, InferenceTerm> {
        let mut map = HashMap::new();
        for &param in &generic_sig.parameters {
            if !excluded.contains(&param) && store.type_parameter(param).kind != KindId::RECORD_ROW {
                let var = self.fresh_variable_in_frame(frame, store.type_parameter(param).kind);
                map.insert(param, InferenceTerm::Var(var));
            }
        }
        map
    }

    /// Converts a canonical `TypeId` to an `InferenceTerm`, replacing generic parameters with their instantiated inference terms.
    pub fn type_id_to_inference(&self, ty: TypeId, subst: &HashMap<TypeParameterId, InferenceTerm>, store: &TypeStore) -> InferenceTerm {
        match store.get(ty) {
            TypeData::Parameter(p) => {
                if let Some(t) = subst.get(p) {
                    t.clone()
                } else {
                    InferenceTerm::Canonical(ty)
                }
            }
            TypeData::Applied { origin, arguments } => {
                let orig_term = self.type_id_to_inference(*origin, subst, store);
                let row_subst = HashMap::new();
                let arg_terms = arguments
                    .iter()
                    .map(|argument| self.static_argument_to_inference_with_rows(argument, subst, &row_subst, store))
                    .collect::<Vec<_>>();
                InferenceTerm::Applied {
                    origin: Box::new(orig_term),
                    arguments: arg_terms.into_boxed_slice(),
                }
            }
            TypeData::StaticData(term) => {
                let arguments = term
                    .arguments
                    .iter()
                    .map(|argument| {
                        self.static_argument_to_inference_with_rows(argument, subst, &HashMap::new(), store)
                    })
                    .collect::<Vec<_>>();
                InferenceTerm::StaticData {
                    domain: term.domain,
                    constructor: term.constructor,
                    arguments: arguments.into_boxed_slice(),
                }
            }
            TypeData::ExactCase { variant, enum_type } => {
                let enum_term = self.type_id_to_inference(*enum_type, subst, store);
                InferenceTerm::ExactCase {
                    variant: *variant,
                    enum_type: Box::new(enum_term),
                }
            }
            TypeData::Union(members) => {
                let member_terms: Vec<InferenceTerm> = members.iter().map(|&m| self.type_id_to_inference(m, subst, store)).collect();
                InferenceTerm::Union(member_terms.into_boxed_slice())
            }
            TypeData::Tuple(elems) => {
                let elem_terms: Vec<InferenceTupleElement> = elems
                    .iter()
                    .map(|e| InferenceTupleElement {
                        label: e.label.clone(),
                        term: self.type_id_to_inference(e.ty, subst, store),
                    })
                    .collect();
                InferenceTerm::Tuple(elem_terms.into_boxed_slice())
            }
            TypeData::Callable(c) => {
                let param_terms: Vec<InferenceCallableParameter> = c
                    .parameters
                    .iter()
                    .map(|p| InferenceCallableParameter {
                        label: p.label.clone(),
                        term: self.type_id_to_inference(p.ty, subst, store),
                        rest: p.rest,
                    })
                    .collect();
                let ret_term = self.type_id_to_inference(c.return_type, subst, store);
                InferenceTerm::Callable(InferenceCallable {
                    parameters: param_terms.into_boxed_slice(),
                    return_type: Box::new(ret_term),
                })
            }
            TypeData::Record(row_id) => {
                let row = store.record_row(*row_id);
                InferenceTerm::Record(InferenceRecord {
                    fields: row
                        .fields
                        .iter()
                        .map(|field| InferenceRecordField {
                            name: field.name.clone(),
                            term: self.type_id_to_inference(field.ty, subst, store),
                        })
                        .collect::<Vec<_>>()
                        .into_boxed_slice(),
                    tail: match row.tail {
                        RecordRowTail::Closed => InferenceRecordTail::Closed,
                        RecordRowTail::Parameter(parameter) => InferenceRecordTail::Parameter(parameter),
                        RecordRowTail::Projection(proj) => InferenceRecordTail::Projection(proj),
                    },
                })
            }
            TypeData::Family(family_id) => InferenceTerm::Family(
                store
                    .get_family(*family_id)
                    .members
                    .iter()
                    .map(|member| InferenceFamilyMember {
                        operation: member.operation.clone(),
                        member_kind: member.member_kind,
                        term: self.type_id_to_inference(member.ty, subst, store),
                    })
                    .collect::<Vec<_>>()
                    .into_boxed_slice(),
            ),
            TypeData::AssociatedProjection(proj) => InferenceTerm::AssociatedProjection {
                subject: Box::new(self.type_id_to_inference(proj.subject, subst, store)),
                trait_ref: InferenceTraitRef {
                    declaration: proj.trait_ref.declaration.clone(),
                    arguments: proj
                        .trait_ref
                        .arguments
                        .iter()
                        .map(|&arg| self.type_id_to_inference(arg, subst, store))
                        .collect::<Vec<_>>()
                        .into_boxed_slice(),
                },
                requirement: proj.requirement.clone(),
            },
            TypeData::TypePack(elements) => InferenceTerm::TypePack(
                elements
                    .iter()
                    .map(|&elem| self.type_id_to_inference(elem, subst, store))
                    .collect::<Vec<_>>()
                    .into_boxed_slice(),
            ),
            TypeData::LabeledTypePack(elements) => InferenceTerm::LabeledTypePack(
                elements
                    .iter()
                    .map(|elem| InferenceLabeledTypePackElement {
                        label: elem.label.clone(),
                        term: self.type_id_to_inference(elem.element, subst, store),
                    })
                    .collect::<Vec<_>>()
                    .into_boxed_slice(),
            ),
            TypeData::CompleteTypePack { positionals, labeled } => InferenceTerm::CompleteTypePack {
                positionals: Box::new(self.type_id_to_inference(*positionals, subst, store)),
                labeled: Box::new(self.type_id_to_inference(*labeled, subst, store)),
            },
            TypeData::TypePackConcat(segments) => InferenceTerm::TypePackConcat(
                segments
                    .iter()
                    .map(|&seg| self.type_id_to_inference(seg, subst, store))
                    .collect::<Vec<_>>()
                    .into_boxed_slice(),
            ),
            TypeData::LabeledTypePackConcat(segments) => InferenceTerm::LabeledTypePackConcat(
                segments
                    .iter()
                    .map(|&seg| self.type_id_to_inference(seg, subst, store))
                    .collect::<Vec<_>>()
                    .into_boxed_slice(),
            ),
            TypeData::TupleFromPacks { positionals, labeled } => InferenceTerm::TupleFromPacks {
                positionals: Box::new(self.type_id_to_inference(*positionals, subst, store)),
                labeled: Box::new(self.type_id_to_inference(*labeled, subst, store)),
            },
            TypeData::CallableFromPacks {
                positionals,
                labeled,
                return_type,
            } => InferenceTerm::CallableFromPacks {
                positionals: Box::new(self.type_id_to_inference(*positionals, subst, store)),
                labeled: Box::new(self.type_id_to_inference(*labeled, subst, store)),
                return_type: Box::new(self.type_id_to_inference(*return_type, subst, store)),
            },
            TypeData::TypeMemberProjection(proj) => {
                if let ProjectionSubject::Type(subj_ty) = &proj.subject {
                    if store.kind_of(*subj_ty) == KindId::COMPLETE_TYPE_PACK && proj.arguments.is_empty() {
                        let lane = match proj.selector.name() {
                            "Positionals" => Some(InferencePackProjectionLane::Positionals),
                            "Labeled" => Some(InferencePackProjectionLane::Labeled),
                            "Complete" => Some(InferencePackProjectionLane::Complete),
                            _ => None,
                        };
                        if let Some(lane) = lane {
                            return InferenceTerm::PackProjection {
                                subject: Box::new(self.type_id_to_inference(*subj_ty, subst, store)),
                                lane,
                            };
                        }
                    }
                }
                self.type_member_projection_to_inference(proj, store.kind_of(ty), subst, &HashMap::new(), store)
            }
            _ => InferenceTerm::Canonical(ty),
        }
    }

    /// Converts a canonical type while mapping only row parameters owned by
    /// the current generic application into row-domain variables.
    pub fn type_id_to_inference_with_rows(
        &self,
        ty: TypeId,
        subst: &HashMap<TypeParameterId, InferenceTerm>,
        row_subst: &HashMap<TypeParameterId, crate::types::row_solver::RecordRowVarId>,
        store: &TypeStore,
    ) -> InferenceTerm {
        match store.get(ty) {
            TypeData::Parameter(parameter) => subst.get(parameter).cloned().unwrap_or(InferenceTerm::Canonical(ty)),
            TypeData::Applied { origin, arguments } => {
                let arguments = arguments
                    .iter()
                    .map(|argument| self.static_argument_to_inference_with_rows(argument, subst, row_subst, store))
                    .collect::<Vec<_>>();
                InferenceTerm::Applied {
                    origin: Box::new(self.type_id_to_inference_with_rows(*origin, subst, row_subst, store)),
                    arguments: arguments.into_boxed_slice(),
                }
            }
            TypeData::StaticData(term) => {
                let arguments = term
                    .arguments
                    .iter()
                    .map(|argument| self.static_argument_to_inference_with_rows(argument, subst, row_subst, store))
                    .collect::<Vec<_>>();
                InferenceTerm::StaticData {
                    domain: term.domain,
                    constructor: term.constructor,
                    arguments: arguments.into_boxed_slice(),
                }
            }
            TypeData::ExactCase { variant, enum_type } => InferenceTerm::ExactCase {
                variant: *variant,
                enum_type: Box::new(self.type_id_to_inference_with_rows(*enum_type, subst, row_subst, store)),
            },
            TypeData::Union(members) => InferenceTerm::Union(
                members
                    .iter()
                    .map(|&member| self.type_id_to_inference_with_rows(member, subst, row_subst, store))
                    .collect::<Vec<_>>()
                    .into_boxed_slice(),
            ),
            TypeData::Tuple(elements) => InferenceTerm::Tuple(
                elements
                    .iter()
                    .map(|element| InferenceTupleElement {
                        label: element.label.clone(),
                        term: self.type_id_to_inference_with_rows(element.ty, subst, row_subst, store),
                    })
                    .collect::<Vec<_>>()
                    .into_boxed_slice(),
            ),
            TypeData::Callable(callable) => InferenceTerm::Callable(InferenceCallable {
                parameters: callable
                    .parameters
                    .iter()
                    .map(|parameter| InferenceCallableParameter {
                        label: parameter.label.clone(),
                        term: self.type_id_to_inference_with_rows(parameter.ty, subst, row_subst, store),
                        rest: parameter.rest,
                    })
                    .collect::<Vec<_>>()
                    .into_boxed_slice(),
                return_type: Box::new(self.type_id_to_inference_with_rows(callable.return_type, subst, row_subst, store)),
            }),
            TypeData::Record(row_id) => {
                let row = store.record_row(*row_id);
                InferenceTerm::Record(InferenceRecord {
                    fields: row
                        .fields
                        .iter()
                        .map(|field| InferenceRecordField {
                            name: field.name.clone(),
                            term: self.type_id_to_inference_with_rows(field.ty, subst, row_subst, store),
                        })
                        .collect::<Vec<_>>()
                        .into_boxed_slice(),
                    tail: match row.tail {
                        RecordRowTail::Closed => InferenceRecordTail::Closed,
                        RecordRowTail::Parameter(parameter) => row_subst
                            .get(&parameter)
                            .copied()
                            .map_or(InferenceRecordTail::Parameter(parameter), InferenceRecordTail::Var),
                        RecordRowTail::Projection(proj) => InferenceRecordTail::Projection(proj),
                    },
                })
            }
            TypeData::Family(family_id) => InferenceTerm::Family(
                store
                    .get_family(*family_id)
                    .members
                    .iter()
                    .map(|member| InferenceFamilyMember {
                        operation: member.operation.clone(),
                        member_kind: member.member_kind,
                        term: self.type_id_to_inference_with_rows(member.ty, subst, row_subst, store),
                    })
                    .collect::<Vec<_>>()
                    .into_boxed_slice(),
            ),
            TypeData::AssociatedProjection(proj) => InferenceTerm::AssociatedProjection {
                subject: Box::new(self.type_id_to_inference_with_rows(proj.subject, subst, row_subst, store)),
                trait_ref: InferenceTraitRef {
                    declaration: proj.trait_ref.declaration.clone(),
                    arguments: proj
                        .trait_ref
                        .arguments
                        .iter()
                        .map(|&arg| self.type_id_to_inference_with_rows(arg, subst, row_subst, store))
                        .collect::<Vec<_>>()
                        .into_boxed_slice(),
                },
                requirement: proj.requirement.clone(),
            },
            TypeData::TypePack(elements) => InferenceTerm::TypePack(
                elements
                    .iter()
                    .map(|&elem| self.type_id_to_inference_with_rows(elem, subst, row_subst, store))
                    .collect::<Vec<_>>()
                    .into_boxed_slice(),
            ),
            TypeData::LabeledTypePack(elements) => InferenceTerm::LabeledTypePack(
                elements
                    .iter()
                    .map(|elem| InferenceLabeledTypePackElement {
                        label: elem.label.clone(),
                        term: self.type_id_to_inference_with_rows(elem.element, subst, row_subst, store),
                    })
                    .collect::<Vec<_>>()
                    .into_boxed_slice(),
            ),
            TypeData::CompleteTypePack { positionals, labeled } => InferenceTerm::CompleteTypePack {
                positionals: Box::new(self.type_id_to_inference_with_rows(*positionals, subst, row_subst, store)),
                labeled: Box::new(self.type_id_to_inference_with_rows(*labeled, subst, row_subst, store)),
            },
            TypeData::TypePackConcat(segments) => InferenceTerm::TypePackConcat(
                segments
                    .iter()
                    .map(|&seg| self.type_id_to_inference_with_rows(seg, subst, row_subst, store))
                    .collect::<Vec<_>>()
                    .into_boxed_slice(),
            ),
            TypeData::LabeledTypePackConcat(segments) => InferenceTerm::LabeledTypePackConcat(
                segments
                    .iter()
                    .map(|&seg| self.type_id_to_inference_with_rows(seg, subst, row_subst, store))
                    .collect::<Vec<_>>()
                    .into_boxed_slice(),
            ),
            TypeData::TupleFromPacks { positionals, labeled } => InferenceTerm::TupleFromPacks {
                positionals: Box::new(self.type_id_to_inference_with_rows(*positionals, subst, row_subst, store)),
                labeled: Box::new(self.type_id_to_inference_with_rows(*labeled, subst, row_subst, store)),
            },
            TypeData::CallableFromPacks {
                positionals,
                labeled,
                return_type,
            } => InferenceTerm::CallableFromPacks {
                positionals: Box::new(self.type_id_to_inference_with_rows(*positionals, subst, row_subst, store)),
                labeled: Box::new(self.type_id_to_inference_with_rows(*labeled, subst, row_subst, store)),
                return_type: Box::new(self.type_id_to_inference_with_rows(*return_type, subst, row_subst, store)),
            },
            TypeData::TypeMemberProjection(proj) => {
                if let ProjectionSubject::Type(subj_ty) = &proj.subject {
                    if store.kind_of(*subj_ty) == KindId::COMPLETE_TYPE_PACK && proj.arguments.is_empty() {
                        let lane = match proj.selector.name() {
                            "Positionals" => Some(InferencePackProjectionLane::Positionals),
                            "Labeled" => Some(InferencePackProjectionLane::Labeled),
                            "Complete" => Some(InferencePackProjectionLane::Complete),
                            _ => None,
                        };
                        if let Some(lane) = lane {
                            return InferenceTerm::PackProjection {
                                subject: Box::new(self.type_id_to_inference_with_rows(*subj_ty, subst, row_subst, store)),
                                lane,
                            };
                        }
                    }
                }
                self.type_member_projection_to_inference(proj, store.kind_of(ty), subst, row_subst, store)
            }
            _ => InferenceTerm::Canonical(ty),
        }
    }

    fn type_member_projection_to_inference(
        &self,
        projection: &TypeLevelProjection,
        result_kind: KindId,
        subst: &HashMap<TypeParameterId, InferenceTerm>,
        row_subst: &HashMap<TypeParameterId, RecordRowVarId>,
        store: &TypeStore,
    ) -> InferenceTerm {
        let subject = match &projection.subject {
            ProjectionSubject::Type(ty) => {
                InferenceProjectionSubject::Type(Box::new(self.type_id_to_inference_with_rows(*ty, subst, row_subst, store)))
            }
            ProjectionSubject::RecordRow(row) => {
                InferenceProjectionSubject::RecordRow(self.record_row_to_inference(*row, subst, row_subst, store))
            }
        };
        let arguments = projection
            .arguments
            .iter()
            .map(|argument| match argument {
                TypeMemberArgument::Type(ty) => {
                    InferenceTypeMemberArgument::Type(self.type_id_to_inference_with_rows(*ty, subst, row_subst, store))
                }
                TypeMemberArgument::Const(value) => InferenceTypeMemberArgument::Const(value.clone()),
                TypeMemberArgument::RecordRow(row) => {
                    InferenceTypeMemberArgument::RecordRow(self.record_row_to_inference(*row, subst, row_subst, store))
                }
            })
            .collect::<Vec<_>>()
            .into_boxed_slice();
        InferenceTerm::TypeMemberProjection {
            subject,
            selector: projection.selector.clone(),
            arguments,
            source_declaration: projection.source_declaration.clone(),
            result_kind,
        }
    }

    fn record_row_to_inference(
        &self,
        row: crate::types::id::RecordRowId,
        subst: &HashMap<TypeParameterId, InferenceTerm>,
        row_subst: &HashMap<TypeParameterId, RecordRowVarId>,
        store: &TypeStore,
    ) -> InferenceRecord {
        let canonical = store.record_row(row);
        InferenceRecord {
            fields: canonical
                .fields
                .iter()
                .map(|field| InferenceRecordField {
                    name: field.name.clone(),
                    term: self.type_id_to_inference_with_rows(field.ty, subst, row_subst, store),
                })
                .collect::<Vec<_>>()
                .into_boxed_slice(),
            tail: match canonical.tail {
                RecordRowTail::Closed => InferenceRecordTail::Closed,
                RecordRowTail::Parameter(parameter) => row_subst
                    .get(&parameter)
                    .copied()
                    .map_or(InferenceRecordTail::Parameter(parameter), InferenceRecordTail::Var),
                RecordRowTail::Projection(projection) => InferenceRecordTail::Projection(projection),
            },
        }
    }

    fn static_argument_to_inference_with_rows(
        &self,
        argument: &crate::types::store::StaticArgument,
        subst: &HashMap<TypeParameterId, InferenceTerm>,
        row_subst: &HashMap<TypeParameterId, crate::types::row_solver::RecordRowVarId>,
        store: &TypeStore,
    ) -> InferenceStaticArgument {
        match argument {
            crate::types::store::StaticArgument::Term(ty) => InferenceStaticArgument::Term(self.type_id_to_inference_with_rows(*ty, subst, row_subst, store)),
            crate::types::store::StaticArgument::RecordRow(row) => {
                let canonical = store.record_row(*row);
                InferenceStaticArgument::RecordRow(InferenceRecord {
                    fields: canonical
                        .fields
                        .iter()
                        .map(|field| InferenceRecordField {
                            name: field.name.clone(),
                            term: self.type_id_to_inference_with_rows(field.ty, subst, row_subst, store),
                        })
                        .collect::<Vec<_>>()
                        .into_boxed_slice(),
                    tail: match canonical.tail {
                        RecordRowTail::Closed => InferenceRecordTail::Closed,
                        RecordRowTail::Parameter(parameter) => row_subst
                            .get(&parameter)
                            .copied()
                            .map_or(InferenceRecordTail::Parameter(parameter), InferenceRecordTail::Var),
                        RecordRowTail::Projection(projection) => InferenceRecordTail::Projection(projection),
                    },
                })
            }
        }
    }

    /// Materializes an inference term for bidirectional checking while
    /// retaining unresolved generic variables as inference terms. Solved
    /// variables become canonical types; unresolved ones stay solver-local so
    /// nested callable expectations can continue constraining one session.
    pub fn materialize_for_expected(&self, term: &InferenceTerm, store: &mut TypeStore) -> Option<TypeId> {
        match term {
            InferenceTerm::Canonical(ty) => Some(*ty),
            InferenceTerm::Var(variable) => {
                let representative = self.find_var(*variable);
                if let Some(&ty) = self.substitutions.get(&representative) {
                    return Some(ty);
                }
                None
            }
            InferenceTerm::Rigid(_) => None,
            InferenceTerm::Applied { origin, arguments } => {
                let origin = self.materialize_for_expected(origin, store)?;
                let arguments = arguments
                    .iter()
                    .map(|argument| self.materialize_static_argument_for_expected(argument, store))
                    .collect::<Option<Vec<_>>>()?;
                store.apply_static_form(origin, &arguments).ok()
            }
            InferenceTerm::StaticData {
                domain,
                constructor,
                arguments,
            } => {
                let arguments = arguments
                    .iter()
                    .map(|argument| self.materialize_static_argument_for_expected(argument, store))
                    .collect::<Option<Vec<_>>>()?;
                store.static_data(*domain, *constructor, arguments.into_boxed_slice()).ok()
            }
            InferenceTerm::ExactCase { variant, enum_type } => {
                let enum_type = self.materialize_for_expected(enum_type, store)?;
                let variant_id = store.variant_identity(*variant).clone();
                store.exact_case_type(&variant_id, enum_type).ok()
            }
            InferenceTerm::Union(members) => {
                let members = members
                    .iter()
                    .map(|member| self.materialize_for_expected(member, store))
                    .collect::<Option<Vec<_>>>()?;
                Some(store.union(&members))
            }
            InferenceTerm::Tuple(elements) => {
                let elements = elements
                    .iter()
                    .map(|element| {
                        Some(TupleTypeElement {
                            label: element.label.clone(),
                            ty: self.materialize_for_expected(&element.term, store)?,
                        })
                    })
                    .collect::<Option<Vec<_>>>()?;
                Some(store.tuple(elements.into_boxed_slice()))
            }
            InferenceTerm::Callable(callable) => {
                let callable_parameters = callable
                    .parameters
                    .iter()
                    .map(|parameter| {
                        Some(CallableParameterType {
                            label: parameter.label.clone(),
                            ty: self.materialize_for_expected(&parameter.term, store)?,
                            rest: parameter.rest,
                        })
                    })
                    .collect::<Option<Vec<_>>>()?;
                let return_type = self.materialize_for_expected(&callable.return_type, store)?;
                Some(store.callable(CallableType {
                    parameters: callable_parameters.into_boxed_slice(),
                    return_type,
                }))
            }
            InferenceTerm::Record(record) => {
                let fields = record
                    .fields
                    .iter()
                    .map(|field| {
                        Some(crate::types::row::RecordRowField {
                            name: field.name.clone(),
                            ty: self.materialize_for_expected(&field.term, store)?,
                        })
                    })
                    .collect::<Option<Vec<_>>>()?;
                let tail = match record.tail {
                    InferenceRecordTail::Closed => RecordRowTail::Closed,
                    InferenceRecordTail::Parameter(parameter) => RecordRowTail::Parameter(parameter),
                    InferenceRecordTail::Projection(proj) => RecordRowTail::Projection(proj),
                    InferenceRecordTail::Var(_) => return None,
                };
                store.record_row_type_checked(fields, tail).ok()
            }
            InferenceTerm::Family(family) => {
                let members = family
                    .iter()
                    .map(|member| {
                        Some(FamilyMemberType {
                            operation: member.operation.clone(),
                            member_kind: member.member_kind,
                            ty: self.materialize_for_expected(&member.term, store)?,
                        })
                    })
                    .collect::<Option<Vec<_>>>()?;
                store.family_type(members).ok()
            }
            InferenceTerm::AssociatedProjection {
                subject,
                trait_ref,
                requirement,
            } => {
                let subject_ty = self.materialize_for_expected(subject, store)?;
                let mut args = Vec::with_capacity(trait_ref.arguments.len());
                for arg in trait_ref.arguments.iter() {
                    args.push(self.materialize_for_expected(arg, store)?);
                }
                let trait_ref = crate::traits::TraitRef::new(trait_ref.declaration.clone(), args);
                Some(store.associated_projection(subject_ty, trait_ref, requirement.clone()))
            }
            InferenceTerm::TypeMemberProjection {
                subject,
                selector,
                arguments,
                source_declaration,
                result_kind,
            } => {
                let projection = self
                    .materialize_type_member_projection(subject, selector, arguments, source_declaration.as_ref(), store)
                    .ok()?;
                Some(store.type_member_projection(projection, *result_kind))
            }
            InferenceTerm::TypePack(elements) => {
                let elements = elements
                    .iter()
                    .map(|elem| self.materialize_for_expected(elem, store))
                    .collect::<Option<Vec<_>>>()?;
                store.type_pack(elements.into_boxed_slice()).ok()
            }
            InferenceTerm::LabeledTypePack(elements) => {
                let elements = elements
                    .iter()
                    .map(|elem| {
                        Some(crate::types::store::LabeledTypePackElement {
                            label: elem.label.clone(),
                            element: self.materialize_for_expected(&elem.term, store)?,
                        })
                    })
                    .collect::<Option<Vec<_>>>()?;
                store.labeled_type_pack(elements.into_boxed_slice()).ok()
            }
            InferenceTerm::CompleteTypePack { positionals, labeled } => {
                let pos = self.materialize_for_expected(positionals, store)?;
                let lab = self.materialize_for_expected(labeled, store)?;
                store.complete_type_pack(pos, lab).ok()
            }
            InferenceTerm::TypePackConcat(segments) => {
                let segments = segments
                    .iter()
                    .map(|seg| self.materialize_for_expected(seg, store))
                    .collect::<Option<Vec<_>>>()?;
                store.type_pack_concat(&segments).ok()
            }
            InferenceTerm::LabeledTypePackConcat(segments) => {
                let segments = segments
                    .iter()
                    .map(|seg| self.materialize_for_expected(seg, store))
                    .collect::<Option<Vec<_>>>()?;
                store.labeled_type_pack_concat(&segments).ok()
            }
            InferenceTerm::TupleFromPacks { positionals, labeled } => {
                let pos = self.materialize_for_expected(positionals, store)?;
                let lab = self.materialize_for_expected(labeled, store)?;
                store.tuple_from_packs(pos, lab).ok()
            }
            InferenceTerm::CallableFromPacks {
                positionals,
                labeled,
                return_type,
            } => {
                let pos = self.materialize_for_expected(positionals, store)?;
                let lab = self.materialize_for_expected(labeled, store)?;
                let ret = self.materialize_for_expected(return_type, store)?;
                store.callable_from_packs(pos, lab, ret).ok()
            }
            InferenceTerm::PackProjection { subject, lane } => {
                let subj = self.materialize_for_expected(subject, store)?;
                match lane {
                    InferencePackProjectionLane::Positionals => store.project_positionals(subj),
                    InferencePackProjectionLane::Labeled => store.project_labeled(subj),
                    InferencePackProjectionLane::Complete => store.project_complete(subj),
                }
                .ok()
            }
        }
    }

    fn materialize_static_argument_for_expected(
        &self,
        argument: &InferenceStaticArgument,
        store: &mut TypeStore,
    ) -> Option<crate::types::store::StaticArgument> {
        match argument {
            InferenceStaticArgument::Term(term) => self.materialize_for_expected(term, store).map(crate::types::store::StaticArgument::Term),
            InferenceStaticArgument::RecordRow(row) => {
                let fields = row
                    .fields
                    .iter()
                    .map(|field| {
                        Some(crate::types::row::RecordRowField {
                            name: field.name.clone(),
                            ty: self.materialize_for_expected(&field.term, store)?,
                        })
                    })
                    .collect::<Option<Vec<_>>>()?;
                let tail = match row.tail {
                    InferenceRecordTail::Closed => RecordRowTail::Closed,
                    InferenceRecordTail::Parameter(parameter) => RecordRowTail::Parameter(parameter),
                    InferenceRecordTail::Projection(projection) => RecordRowTail::Projection(projection),
                    InferenceRecordTail::Var(_) => return None,
                };
                store.record_row_checked(fields, tail).ok().map(crate::types::store::StaticArgument::RecordRow)
            }
        }
    }

    /// Lowers an application of a canonical type lambda into an inference term.
    ///
    /// Ordinary beta reduction requires canonical arguments. During generic
    /// application, however, the argument may still be a solver variable. The
    /// symbolic form must survive until that variable receives its value so
    /// fixed constructors such as `Either<E, X>` can still constrain `X`.
    pub fn symbolic_beta_reduce(&self, term: &InferenceTerm, store: &TypeStore) -> Option<InferenceTerm> {
        let InferenceTerm::Applied { origin, arguments } = term else { return None };
        let InferenceTerm::Canonical(origin_ty) = origin.as_ref() else { return None };
        let TypeData::Lambda(lambda_id) = store.get(*origin_ty) else { return None };
        let lambda = store.arena().get_lambda(*lambda_id);
        if arguments.len() > lambda.parameter_kinds.len() {
            return None;
        }

        let arguments = arguments
            .iter()
            .map(|argument| argument.term().cloned())
            .collect::<Option<Vec<_>>>()?;
        self.scoped_to_inference(lambda.body, 0, &arguments, store)
    }

    fn scoped_to_inference(&self, scoped: crate::types::id::ScopedTypeId, depth: u32, arguments: &[InferenceTerm], store: &TypeStore) -> Option<InferenceTerm> {
        match store.arena().get_scoped(scoped).clone() {
            ScopedTypeData::Bound { depth: bound_depth, index } => (bound_depth == depth).then(|| arguments.get(index as usize).cloned()).flatten(),
            ScopedTypeData::Free(ty) => Some(self.type_id_to_inference(ty, &HashMap::new(), store)),
            ScopedTypeData::Applied {
                origin,
                arguments: nested_arguments,
            } => Some(InferenceTerm::Applied {
                origin: Box::new(self.scoped_to_inference(origin, depth, arguments, store)?),
                arguments: nested_arguments
                    .iter()
                    .map(|&argument| self.scoped_to_inference(argument, depth, arguments, store).map(InferenceStaticArgument::Term))
                    .collect::<Option<Vec<_>>>()?
                    .into_boxed_slice(),
            }),
            ScopedTypeData::Union(members) => Some(InferenceTerm::Union(
                members
                    .iter()
                    .map(|&member| self.scoped_to_inference(member, depth, arguments, store))
                    .collect::<Option<Vec<_>>>()?
                    .into_boxed_slice(),
            )),
            ScopedTypeData::Tuple(elements) => Some(InferenceTerm::Tuple(
                elements
                    .iter()
                    .map(|element| {
                        Some(InferenceTupleElement {
                            label: element.label.clone(),
                            term: self.scoped_to_inference(element.ty, depth, arguments, store)?,
                        })
                    })
                    .collect::<Option<Vec<_>>>()?
                    .into_boxed_slice(),
            )),
            ScopedTypeData::Record(fields) => Some(InferenceTerm::Record(InferenceRecord {
                fields: fields
                    .iter()
                    .map(|field| {
                        Some(InferenceRecordField {
                            name: field.name.clone(),
                            term: self.scoped_to_inference(field.ty, depth, arguments, store)?,
                        })
                    })
                    .collect::<Option<Vec<_>>>()?
                    .into_boxed_slice(),
                tail: InferenceRecordTail::Closed,
            })),
            ScopedTypeData::OpenRecord(record) => {
                let tail = match record.tail {
                    ScopedRecordTail::FreeParameter(parameter) => InferenceRecordTail::Parameter(parameter),
                    ScopedRecordTail::Bound { .. } => return None,
                };
                Some(InferenceTerm::Record(InferenceRecord {
                    fields: record
                        .fields
                        .iter()
                        .map(|field| {
                            Some(InferenceRecordField {
                                name: field.name.clone(),
                                term: self.scoped_to_inference(field.ty, depth, arguments, store)?,
                            })
                        })
                        .collect::<Option<Vec<_>>>()?
                        .into_boxed_slice(),
                    tail,
                }))
            }
            ScopedTypeData::Callable(callable) => Some(InferenceTerm::Callable(InferenceCallable {
                parameters: callable
                    .parameters
                    .iter()
                    .map(|parameter| {
                        Some(InferenceCallableParameter {
                            label: parameter.label.clone(),
                            term: self.scoped_to_inference(parameter.ty, depth, arguments, store)?,
                            rest: if parameter.rest { RestMode::Positional } else { RestMode::None },
                        })
                    })
                    .collect::<Option<Vec<_>>>()?
                    .into_boxed_slice(),
                return_type: Box::new(self.scoped_to_inference(callable.return_type, depth, arguments, store)?),
            })),
            ScopedTypeData::TypePack(elements) => Some(InferenceTerm::TypePack(
                elements
                    .iter()
                    .map(|&elem| self.scoped_to_inference(elem, depth, arguments, store))
                    .collect::<Option<Vec<_>>>()?
                    .into_boxed_slice(),
            )),
            ScopedTypeData::LabeledTypePack(elements) => Some(InferenceTerm::LabeledTypePack(
                elements
                    .iter()
                    .map(|elem| {
                        Some(InferenceLabeledTypePackElement {
                            label: elem.label.clone(),
                            term: self.scoped_to_inference(elem.element, depth, arguments, store)?,
                        })
                    })
                    .collect::<Option<Vec<_>>>()?
                    .into_boxed_slice(),
            )),
            ScopedTypeData::CompleteTypePack { positionals, labeled } => Some(InferenceTerm::CompleteTypePack {
                positionals: Box::new(self.scoped_to_inference(positionals, depth, arguments, store)?),
                labeled: Box::new(self.scoped_to_inference(labeled, depth, arguments, store)?),
            }),
            ScopedTypeData::TypePackConcat(segments) => Some(InferenceTerm::TypePackConcat(
                segments
                    .iter()
                    .map(|&seg| self.scoped_to_inference(seg, depth, arguments, store))
                    .collect::<Option<Vec<_>>>()?
                    .into_boxed_slice(),
            )),
            ScopedTypeData::LabeledTypePackConcat(segments) => Some(InferenceTerm::LabeledTypePackConcat(
                segments
                    .iter()
                    .map(|&seg| self.scoped_to_inference(seg, depth, arguments, store))
                    .collect::<Option<Vec<_>>>()?
                    .into_boxed_slice(),
            )),
            ScopedTypeData::TupleFromPacks { positionals, labeled } => Some(InferenceTerm::TupleFromPacks {
                positionals: Box::new(self.scoped_to_inference(positionals, depth, arguments, store)?),
                labeled: Box::new(self.scoped_to_inference(labeled, depth, arguments, store)?),
            }),
            ScopedTypeData::CallableFromPacks {
                positionals,
                labeled,
                return_type,
            } => Some(InferenceTerm::CallableFromPacks {
                positionals: Box::new(self.scoped_to_inference(positionals, depth, arguments, store)?),
                labeled: Box::new(self.scoped_to_inference(labeled, depth, arguments, store)?),
                return_type: Box::new(self.scoped_to_inference(return_type, depth, arguments, store)?),
            }),
            ScopedTypeData::PackProjection { subject, lane } => Some(InferenceTerm::PackProjection {
                subject: Box::new(self.scoped_to_inference(subject, depth, arguments, store)?),
                lane: match lane {
                    crate::types::type_lambda::ScopedPackProjectionLane::Positionals => InferencePackProjectionLane::Positionals,
                    crate::types::type_lambda::ScopedPackProjectionLane::Labeled => InferencePackProjectionLane::Labeled,
                    crate::types::type_lambda::ScopedPackProjectionLane::Complete => InferencePackProjectionLane::Complete,
                },
            }),
            // Nested lambdas need their own bound-depth shifting. No current
            // generic-call boundary requires lowering one, so retain it as an
            // unavailable symbolic form rather than inventing a canonical type.
            ScopedTypeData::Lambda(_) => None,
        }
    }

    /// Decomposes one canonical application without choosing an abstraction.
    fn canonical_constructor_view(&self, actual: TypeId, store: &TypeStore) -> Option<CanonicalConstructorView> {
        match store.get(actual) {
            TypeData::Applied { origin, arguments } => Some(CanonicalConstructorView {
                actual,
                origin: *origin,
                arguments: arguments
                    .iter()
                    .map(|argument| argument.as_type_id())
                    .collect::<Option<Vec<_>>>()?
                    .into_boxed_slice(),
                origin_kind: store.kind_of(*origin),
            }),
            TypeData::Nominal { .. } => Some(CanonicalConstructorView {
                actual,
                origin: actual,
                arguments: Box::new([]),
                origin_kind: store.kind_of(actual),
            }),
            _ => None,
        }
    }

    /// Matches an applied term whose constructor is a solver variable against
    /// one canonical constructor view. Formal arguments select actual
    /// positions; no suffix-only correspondence is assumed.
    fn match_applied_constructor_shapes(
        &mut self,
        left_origin: &InferenceTerm,
        left_arguments: &[InferenceTerm],
        right_origin: &InferenceTerm,
        right_arguments: &[InferenceTerm],
        store: &mut TypeStore,
    ) -> Result<Option<SolveEffect>, InferenceFailureReason> {
        let (variable, concrete_origin, concrete_arguments, variable_arguments) = match (left_origin, right_origin) {
            (InferenceTerm::Var(variable), InferenceTerm::Canonical(concrete_origin)) => (*variable, *concrete_origin, right_arguments, left_arguments),
            (InferenceTerm::Canonical(concrete_origin), InferenceTerm::Var(variable)) => (*variable, *concrete_origin, left_arguments, right_arguments),
            _ => return Ok(None),
        };

        if variable_arguments.len() > concrete_arguments.len() {
            return Ok(None);
        }

        // Concrete constructor arguments may themselves be applied types
        // (for example `List<Int>`). Materialize only when fully resolved;
        // unresolved arguments still defer this candidate without inventing
        // a constructor shape.
        let canonical_arguments = concrete_arguments
            .iter()
            .map(|argument| self.materialize(argument, store).ok())
            .collect::<Option<Vec<_>>>();
        let Some(canonical_arguments) = canonical_arguments else {
            return Ok(None);
        };
        let actual = store
            .apply_type_form(concrete_origin, &canonical_arguments)
            .map_err(|_| InferenceFailureReason::StructuralMismatch {
                left: Box::new(InferenceTerm::Applied {
                    origin: Box::new(left_origin.clone()),
                    arguments: inference_arguments_from_terms(left_arguments),
                }),
                right: Box::new(InferenceTerm::Applied {
                    origin: Box::new(right_origin.clone()),
                    arguments: inference_arguments_from_terms(right_arguments),
                }),
            })?;
        let Some(view) = self.canonical_constructor_view(actual, store) else {
            return Ok(None);
        };
        let variable_kind = self
            .variable_by_representative(variable)
            .map(|candidate| candidate.kind)
            .ok_or(InferenceFailureReason::MissingVariableMetadata { var: variable })?;
        let positions = self.select_constructor_positions(variable_arguments, &view.arguments, store);
        let Some(positions) = positions else {
            return Ok(None);
        };
        let constructor = if positions.iter().copied().eq(0..view.arguments.len()) && view.origin_kind == variable_kind {
            view.origin
        } else {
            self.synthesize_constructor_candidate(&view, variable_kind, &positions, store)
                .ok_or_else(|| InferenceFailureReason::StructuralMismatch {
                    left: Box::new(InferenceTerm::Applied {
                        origin: Box::new(left_origin.clone()),
                        arguments: inference_arguments_from_terms(left_arguments),
                    }),
                    right: Box::new(InferenceTerm::Applied {
                        origin: Box::new(right_origin.clone()),
                        arguments: inference_arguments_from_terms(right_arguments),
                    }),
                })?
        };

        let mut changed = self
            .unify_terms(&InferenceTerm::Var(variable), &InferenceTerm::Canonical(constructor), store)?
            .is_changed();
        for (variable_argument, &position) in variable_arguments.iter().zip(positions.iter()) {
            let concrete_argument = InferenceTerm::Canonical(view.arguments[position]);
            changed |= self.unify_terms(variable_argument, &concrete_argument, store)?.is_changed();
        }
        Ok(Some(if changed { SolveEffect::Changed } else { SolveEffect::Unchanged }))
    }

    fn select_constructor_positions(&self, formal_arguments: &[InferenceTerm], actual_arguments: &[TypeId], store: &mut TypeStore) -> Option<Vec<usize>> {
        if formal_arguments.len() > actual_arguments.len() {
            return None;
        }
        let mut candidates = Vec::new();
        Self::enumerate_constructor_positions(formal_arguments.len(), actual_arguments.len(), 0, &mut Vec::new(), &mut candidates);
        candidates.retain(|positions| {
            formal_arguments
                .iter()
                .zip(positions.iter())
                .all(|(formal, position)| self.known_type_for_term(formal, store).is_none_or(|known| known == actual_arguments[*position]))
        });
        if candidates.len() == 1 {
            candidates.pop()
        } else if candidates.len() > 1 && formal_arguments.iter().all(|formal| self.known_type_for_term(formal, store).is_none()) {
            // Preserve established HKT application convention when no formal
            // argument carries position evidence: unspecialized arguments
            // occupy the trailing actual slots. Any known formal evidence
            // above forces structural selection instead of this fallback.
            Some((actual_arguments.len() - formal_arguments.len()..actual_arguments.len()).collect())
        } else {
            None
        }
    }

    fn enumerate_constructor_positions(formal_len: usize, actual_len: usize, next_actual: usize, positions: &mut Vec<usize>, candidates: &mut Vec<Vec<usize>>) {
        if positions.len() == formal_len {
            candidates.push(positions.clone());
            return;
        }
        let remaining = formal_len - positions.len();
        for position in next_actual..=actual_len.saturating_sub(remaining) {
            positions.push(position);
            Self::enumerate_constructor_positions(formal_len, actual_len, position + 1, positions, candidates);
            positions.pop();
        }
    }

    fn known_type_for_term(&self, term: &InferenceTerm, store: &mut TypeStore) -> Option<TypeId> {
        if let Ok(ty) = self.materialize(term, store) {
            return Some(ty);
        }
        let InferenceTerm::Var(variable) = term else { return None };
        let representative = self.find_var(*variable);
        self.constraints.iter().find_map(|constraint| {
            let InferenceRelation::Equivalent(left, right) = &constraint.relation else {
                return None;
            };
            match (left, right) {
                (InferenceTerm::Var(left), InferenceTerm::Canonical(ty)) if self.find_var(*left) == representative => Some(*ty),
                (InferenceTerm::Canonical(ty), InferenceTerm::Var(right)) if self.find_var(*right) == representative => Some(*ty),
                _ => None,
            }
        })
    }

    fn synthesize_constructor_candidate(
        &mut self,
        view: &CanonicalConstructorView,
        variable_kind: KindId,
        positions: &[usize],
        store: &mut TypeStore,
    ) -> Option<TypeId> {
        if store.kind_of(view.actual) != KindId::TYPE {
            return None;
        }
        let KindData::Arrow { parameters, result } = store.get_kind(variable_kind).clone() else {
            return None;
        };
        if parameters.len() != positions.len() {
            return None;
        }
        let actual_kinds = view.arguments.iter().map(|argument| store.kind_of(*argument)).collect::<Vec<_>>();
        if store.apply_kind(view.origin_kind, &actual_kinds).ok()? != KindId::TYPE {
            return None;
        }
        let arena = store.arena_mut();
        let scoped_origin = arena.intern_scoped(ScopedTypeData::Free(view.origin));
        let mut scoped_arguments = Vec::with_capacity(view.arguments.len());
        for (actual_index, _argument) in view.arguments.iter().enumerate() {
            if let Some(formal_index) = positions.iter().position(|position| *position == actual_index) {
                scoped_arguments.push(arena.intern_scoped(ScopedTypeData::Bound {
                    depth: 0,
                    index: formal_index as u32,
                }));
            } else {
                scoped_arguments.push(arena.intern_scoped(ScopedTypeData::Free(view.arguments[actual_index])));
            }
        }
        let scoped_body = arena.intern_scoped(ScopedTypeData::Applied {
            origin: scoped_origin,
            arguments: scoped_arguments.into_boxed_slice(),
        });
        let lambda_id = arena.intern_lambda(parameters, scoped_body, result, None);
        let lambda = store.type_lambda(lambda_id);
        (store.kind_of(lambda) == variable_kind).then_some(lambda)
    }

    /// Rewrites solved inference variables inside a contextual expectation
    /// while retaining unresolved variables for later constraints. This is
    /// intentionally term-preserving: a callable can have a known parameter
    /// type while its return type is still being inferred.
    pub fn term_for_expected(&self, term: &InferenceTerm) -> InferenceTerm {
        match term {
            InferenceTerm::Canonical(ty) => InferenceTerm::Canonical(*ty),
            InferenceTerm::Rigid(rigid) => InferenceTerm::Rigid(*rigid),
            InferenceTerm::Var(variable) => self
                .substitutions
                .get(&self.find_var(*variable))
                .copied()
                .map(InferenceTerm::Canonical)
                .unwrap_or_else(|| InferenceTerm::Var(self.find_var(*variable))),
            InferenceTerm::Applied { origin, arguments } => InferenceTerm::Applied {
                origin: Box::new(self.term_for_expected(origin)),
                arguments: arguments
                    .iter()
                    .map(|argument| self.static_argument_for_expected(argument))
                    .collect::<Vec<_>>()
                    .into_boxed_slice(),
            },
            InferenceTerm::StaticData {
                domain,
                constructor,
                arguments,
            } => InferenceTerm::StaticData {
                domain: *domain,
                constructor: *constructor,
                arguments: arguments
                    .iter()
                    .map(|argument| self.static_argument_for_expected(argument))
                    .collect::<Vec<_>>()
                    .into_boxed_slice(),
            },
            InferenceTerm::ExactCase { variant, enum_type } => InferenceTerm::ExactCase {
                variant: *variant,
                enum_type: Box::new(self.term_for_expected(enum_type)),
            },
            InferenceTerm::Union(members) => InferenceTerm::Union(
                members
                    .iter()
                    .map(|member| self.term_for_expected(member))
                    .collect::<Vec<_>>()
                    .into_boxed_slice(),
            ),
            InferenceTerm::Tuple(elements) => InferenceTerm::Tuple(
                elements
                    .iter()
                    .map(|element| InferenceTupleElement {
                        label: element.label.clone(),
                        term: self.term_for_expected(&element.term),
                    })
                    .collect::<Vec<_>>()
                    .into_boxed_slice(),
            ),
            InferenceTerm::Callable(callable) => InferenceTerm::Callable(InferenceCallable {
                parameters: callable
                    .parameters
                    .iter()
                    .map(|parameter| InferenceCallableParameter {
                        label: parameter.label.clone(),
                        term: self.term_for_expected(&parameter.term),
                        rest: parameter.rest,
                    })
                    .collect::<Vec<_>>()
                    .into_boxed_slice(),
                return_type: Box::new(self.term_for_expected(&callable.return_type)),
            }),
            InferenceTerm::Record(record) => InferenceTerm::Record(InferenceRecord {
                fields: record
                    .fields
                    .iter()
                    .map(|field| InferenceRecordField {
                        name: field.name.clone(),
                        term: self.term_for_expected(&field.term),
                    })
                    .collect::<Vec<_>>()
                    .into_boxed_slice(),
                tail: record.tail,
            }),
            InferenceTerm::Family(family) => InferenceTerm::Family(
                family
                    .iter()
                    .map(|member| InferenceFamilyMember {
                        operation: member.operation.clone(),
                        member_kind: member.member_kind,
                        term: self.term_for_expected(&member.term),
                    })
                    .collect::<Vec<_>>()
                    .into_boxed_slice(),
            ),
            InferenceTerm::AssociatedProjection {
                subject,
                trait_ref,
                requirement,
            } => InferenceTerm::AssociatedProjection {
                subject: Box::new(self.term_for_expected(subject)),
                trait_ref: InferenceTraitRef {
                    declaration: trait_ref.declaration.clone(),
                    arguments: trait_ref
                        .arguments
                        .iter()
                        .map(|arg| self.term_for_expected(arg))
                        .collect::<Vec<_>>()
                        .into_boxed_slice(),
                },
                requirement: requirement.clone(),
            },
            InferenceTerm::TypeMemberProjection {
                subject,
                selector,
                arguments,
                source_declaration,
                result_kind,
            } => InferenceTerm::TypeMemberProjection {
                subject: match subject {
                    InferenceProjectionSubject::Type(term) => InferenceProjectionSubject::Type(Box::new(self.term_for_expected(term))),
                    InferenceProjectionSubject::RecordRow(row) => InferenceProjectionSubject::RecordRow(self.record_for_expected(row)),
                },
                selector: selector.clone(),
                arguments: arguments
                    .iter()
                    .map(|argument| match argument {
                        InferenceTypeMemberArgument::Type(term) => InferenceTypeMemberArgument::Type(self.term_for_expected(term)),
                        InferenceTypeMemberArgument::Const(value) => InferenceTypeMemberArgument::Const(value.clone()),
                        InferenceTypeMemberArgument::RecordRow(row) => {
                            InferenceTypeMemberArgument::RecordRow(self.record_for_expected(row))
                        }
                    })
                    .collect::<Vec<_>>()
                    .into_boxed_slice(),
                source_declaration: source_declaration.clone(),
                result_kind: *result_kind,
            },
            InferenceTerm::TypePack(elements) => {
                InferenceTerm::TypePack(elements.iter().map(|elem| self.term_for_expected(elem)).collect::<Vec<_>>().into_boxed_slice())
            }
            InferenceTerm::LabeledTypePack(elements) => InferenceTerm::LabeledTypePack(
                elements
                    .iter()
                    .map(|elem| InferenceLabeledTypePackElement {
                        label: elem.label.clone(),
                        term: self.term_for_expected(&elem.term),
                    })
                    .collect::<Vec<_>>()
                    .into_boxed_slice(),
            ),
            InferenceTerm::CompleteTypePack { positionals, labeled } => InferenceTerm::CompleteTypePack {
                positionals: Box::new(self.term_for_expected(positionals)),
                labeled: Box::new(self.term_for_expected(labeled)),
            },
            InferenceTerm::TypePackConcat(segments) => {
                InferenceTerm::TypePackConcat(segments.iter().map(|seg| self.term_for_expected(seg)).collect::<Vec<_>>().into_boxed_slice())
            }
            InferenceTerm::LabeledTypePackConcat(segments) => {
                InferenceTerm::LabeledTypePackConcat(segments.iter().map(|seg| self.term_for_expected(seg)).collect::<Vec<_>>().into_boxed_slice())
            }
            InferenceTerm::TupleFromPacks { positionals, labeled } => InferenceTerm::TupleFromPacks {
                positionals: Box::new(self.term_for_expected(positionals)),
                labeled: Box::new(self.term_for_expected(labeled)),
            },
            InferenceTerm::CallableFromPacks {
                positionals,
                labeled,
                return_type,
            } => InferenceTerm::CallableFromPacks {
                positionals: Box::new(self.term_for_expected(positionals)),
                labeled: Box::new(self.term_for_expected(labeled)),
                return_type: Box::new(self.term_for_expected(return_type)),
            },
            InferenceTerm::PackProjection { subject, lane } => InferenceTerm::PackProjection {
                subject: Box::new(self.term_for_expected(subject)),
                lane: *lane,
            },
        }
    }

    fn record_for_expected(&self, record: &InferenceRecord) -> InferenceRecord {
        let normalized = self.normalized_record_row(record);
        InferenceRecord {
            fields: normalized
                .fields
                .iter()
                .map(|field| InferenceRecordField {
                    name: field.name.clone(),
                    term: self.term_for_expected(&field.ty),
                })
                .collect::<Vec<_>>()
                .into_boxed_slice(),
            tail: match normalized.tail {
                RecordRowTermTail::Closed => InferenceRecordTail::Closed,
                RecordRowTermTail::Parameter(parameter) => InferenceRecordTail::Parameter(parameter),
                RecordRowTermTail::Projection(projection) => InferenceRecordTail::Projection(projection),
                RecordRowTermTail::Var(variable) => InferenceRecordTail::Var(variable),
            },
        }
    }

    fn static_argument_for_expected(&self, argument: &InferenceStaticArgument) -> InferenceStaticArgument {
        match argument {
            InferenceStaticArgument::Term(term) => InferenceStaticArgument::Term(self.term_for_expected(term)),
            InferenceStaticArgument::RecordRow(row) => InferenceStaticArgument::RecordRow(InferenceRecord {
                fields: row
                    .fields
                    .iter()
                    .map(|field| InferenceRecordField {
                        name: field.name.clone(),
                        term: self.term_for_expected(&field.term),
                    })
                    .collect::<Vec<_>>()
                    .into_boxed_slice(),
                tail: row.tail,
            }),
        }
    }

    /// Converts a `TypeTerm` to an `InferenceTerm`.
    pub fn type_term_to_inference(
        &self,
        term: &TypeTerm,
        subst: &HashMap<TypeParameterId, InferenceTerm>,
        store: &TypeStore,
    ) -> Result<InferenceTerm, InferenceFailureReason> {
        match term {
            TypeTerm::Canonical(ty) => Ok(self.type_id_to_inference(*ty, subst, store)),
            TypeTerm::SelfType(_) => Err(InferenceFailureReason::UnresolvedSelf),
            TypeTerm::Infer(v) => Ok(InferenceTerm::Var(*v)),
        }
    }

    /// Solves all accumulated constraints.
    pub fn solve(&mut self, store: &mut TypeStore, hierarchy: &dyn TypeHierarchy) -> InferenceOutcome {
        let control = CheckerControl::default();
        self.solve_with_control(store, hierarchy, &control)
    }

    /// Propagates constraints until a fixed point is reached without terminalizing frame/root underconstraint.
    pub fn propagate_with_control(
        &mut self,
        store: &mut TypeStore,
        hierarchy: &dyn TypeHierarchy,
        control: &CheckerControl,
    ) -> Result<bool, Box<InferenceOutcome>> {
        let mut any_changed = false;
        loop {
            if control.is_cancelled() {
                return Err(Box::new(InferenceOutcome::Cancelled));
            }
            if let Err(report) = control.charge_scc_iteration() {
                return Err(Box::new(InferenceOutcome::BudgetExceeded(report)));
            }
            let mut changed = false;
            let constraints = self.constraints.clone();
            for (constraint_index, constraint) in constraints.iter().enumerate() {
                if control.is_cancelled() {
                    return Err(Box::new(InferenceOutcome::Cancelled));
                }
                if let Err(report) = control.charge_step() {
                    return Err(Box::new(InferenceOutcome::BudgetExceeded(report)));
                }
                let constraint_index = constraint_index as u32;
                // Retrying a constraint invalidates its previous residual
                // state. If it is still unresolved, the projection branch
                // records it again during this evaluation.
                self.residual_obligations.remove(&constraint_index);
                self.active_constraint_index = Some(constraint_index);
                let effect = match &constraint.relation {
                    InferenceRelation::Equivalent(left, right) => self.unify_terms(left, right, store),
                    InferenceRelation::Subtype(sub, sup) => self.subtype_terms(sub, sup, store, hierarchy),
                };
                self.active_constraint_index = None;
                match effect {
                    Ok(effect) => {
                        changed |= effect.is_changed();
                        self.record_bound_origin(&constraint.relation, constraint.origin.clone(), constraint_index, store);
                        self.record_variable_constraint_indices(&constraint.relation, constraint_index);
                    }
                    Err(failure) => {
                        let related = self.related_constraint_indices(&constraint.relation);
                        return Err(Box::new(self.failure_outcome_with_related(
                            failure,
                            Some(constraint_index),
                            Some(constraint.origin.clone()),
                            &related,
                        )));
                    }
                }
            }

            let effect = self.propagate_subtype_edges(store, hierarchy, control)?;
            changed |= effect.is_changed();

            // Try to resolve remaining var_terms
            let var_terms = self.var_terms.clone();
            for (var, term) in var_terms {
                if control.is_cancelled() {
                    return Err(Box::new(InferenceOutcome::Cancelled));
                }
                if let Err(report) = control.charge_step() {
                    return Err(Box::new(InferenceOutcome::BudgetExceeded(report)));
                }
                let rep = self.find_var(var);
                if !self.substitutions.contains_key(&rep) && !self.term_has_projection(&term, store) {
                    if let Ok(ty) = self.materialize(&term, store) {
                        match self.bind(rep, ty, store) {
                            Ok(effect) => changed |= effect.is_changed(),
                            Err(failure) => {
                                return Err(Box::new(self.failure_outcome(failure, None, None)));
                            }
                        }
                    }
                }
            }

            // Try to resolve from lower/upper bounds
            let vars_to_check: Vec<InferVarId> = self.variables.iter().map(|v| self.find_var(v.id)).collect();
            for rep in vars_to_check {
                if control.is_cancelled() {
                    return Err(Box::new(InferenceOutcome::Cancelled));
                }
                if let Err(report) = control.charge_step() {
                    return Err(Box::new(InferenceOutcome::BudgetExceeded(report)));
                }
                if !self.substitutions.contains_key(&rep) {
                    if let Some(lowers) = self.lower_bounds.get(&rep).cloned() {
                        if !lowers.is_empty() {
                            let candidate = store.union(&lowers);
                            if let Some(uppers) = self.upper_bounds.get(&rep) {
                                let mut ok = true;
                                let mut failed_upper = None;
                                for &upper in uppers {
                                    if !is_subtype(store, hierarchy, candidate, upper) {
                                        ok = false;
                                        failed_upper = Some(upper);
                                        break;
                                    }
                                }
                                if !ok {
                                    let failure = InferenceFailureReason::ConflictingBounds {
                                        var: rep,
                                        lower: candidate,
                                        upper: failed_upper.expect("failed upper recorded with non-empty failure"),
                                    };
                                    let upper = match &failure {
                                        InferenceFailureReason::ConflictingBounds { upper, .. } => *upper,
                                        _ => unreachable!("bound reconciliation creates ConflictingBounds"),
                                    };
                                    let (constraint_index, origin) = self
                                        .bound_origins
                                        .get(&(rep, upper))
                                        .map(|(index, origin)| (Some(*index), Some(origin.clone())))
                                        .unwrap_or((None, None));
                                    return Err(Box::new(self.failure_outcome(failure, constraint_index, origin)));
                                }
                            }
                            match self.bind(rep, candidate, store) {
                                Ok(effect) => changed |= effect.is_changed(),
                                Err(failure) => {
                                    return Err(Box::new(self.failure_outcome(failure, None, None)));
                                }
                            }
                        }
                    } else if let Some(uppers) = self.upper_bounds.get(&rep).cloned() {
                        if let Some(candidate) = self.context_selection_candidate(rep, &uppers) {
                            if let Some(failed_upper) = uppers
                                .iter()
                                .copied()
                                .find(|upper| *upper != candidate && !is_subtype(store, hierarchy, candidate, *upper))
                            {
                                let failure = InferenceFailureReason::ConflictingBounds {
                                    var: rep,
                                    lower: candidate,
                                    upper: failed_upper,
                                };
                                let (constraint_index, origin) = self
                                    .bound_origins
                                    .get(&(rep, failed_upper))
                                    .map(|(index, origin)| (Some(*index), Some(origin.clone())))
                                    .unwrap_or((None, None));
                                return Err(Box::new(self.failure_outcome(failure, constraint_index, origin)));
                            }
                            match self.bind(rep, candidate, store) {
                                Ok(effect) => changed |= effect.is_changed(),
                                Err(failure) => {
                                    return Err(Box::new(self.failure_outcome(failure, None, None)));
                                }
                            }
                        } else if uppers.len() == 1 && !self.is_declaration_restriction_only(rep, uppers[0]) {
                            match self.bind(rep, uppers[0], store) {
                                Ok(effect) => changed |= effect.is_changed(),
                                Err(failure) => {
                                    return Err(Box::new(self.failure_outcome(failure, None, None)));
                                }
                            }
                        }
                    }
                }
            }

            // A value/equivalence constraint may bind a variable before a
            // declaration restriction is revisited. Validate both directions
            // after every reconciliation pass so source lower bounds such as
            // `Number <: T` cannot be silently bypassed by an argument-derived
            // substitution.
            for rep in self.variables.iter().map(|variable| self.find_var(variable.id)).collect::<Vec<_>>() {
                let Some(candidate) = self.substitutions.get(&rep).copied() else {
                    continue;
                };
                for lower in self.lower_bounds.get(&rep).cloned().unwrap_or_default() {
                    if !is_subtype(store, hierarchy, lower, candidate) {
                        let (constraint_index, origin) = self
                            .bound_origins
                            .get(&(rep, lower))
                            .map(|(index, origin)| (Some(*index), Some(origin.clone())))
                            .unwrap_or((None, None));
                        return Err(Box::new(self.failure_outcome(
                            InferenceFailureReason::ConflictingBounds {
                                var: rep,
                                lower,
                                upper: candidate,
                            },
                            constraint_index,
                            origin,
                        )));
                    }
                }
                for upper in self.upper_bounds.get(&rep).cloned().unwrap_or_default() {
                    if !is_subtype(store, hierarchy, candidate, upper) {
                        let (constraint_index, origin) = self
                            .bound_origins
                            .get(&(rep, upper))
                            .map(|(index, origin)| (Some(*index), Some(origin.clone())))
                            .unwrap_or((None, None));
                        return Err(Box::new(self.failure_outcome(
                            InferenceFailureReason::ConflictingBounds {
                                var: rep,
                                lower: candidate,
                                upper,
                            },
                            constraint_index,
                            origin,
                        )));
                    }
                }
            }

            any_changed |= changed;
            if !changed {
                break;
            }
        }

        // Propagate solutions across alias classes
        for var in &self.variables {
            let rep = self.find_var(var.id);
            if let Some(&ty) = self.substitutions.get(&rep) {
                self.substitutions.insert(var.id, ty);
            }
        }

        Ok(any_changed)
    }

    /// Classifies the completion outcome for variables owned by a specific frame.
    pub fn finish_frame(&self, frame: InferenceFrameId) -> InferenceOutcome {
        let mut unsolved = Vec::new();
        for var in &self.variables {
            if var.frame == frame && !self.substitutions.contains_key(&var.id) {
                unsolved.push(var.id);
            }
        }

        if !unsolved.is_empty() || self.residual_blocks_frame(frame) {
            InferenceOutcome::Underconstrained(UnderconstrainedInference { unsolved_vars: unsolved })
        } else {
            InferenceOutcome::Solved(InferenceSolution {
                substitutions: self.substitutions.clone(),
                support: self
                    .variables
                    .iter()
                    .filter(|variable| variable.frame == frame)
                    .filter_map(|variable| variable.support.map(|support| (variable.id, support)))
                    .collect(),
            })
        }
    }

    /// Classifies the completion outcome across all variables in the graph.
    pub fn finish_root(&self) -> InferenceOutcome {
        let mut unsolved = Vec::new();
        for var in &self.variables {
            if !self.substitutions.contains_key(&var.id) {
                unsolved.push(var.id);
            }
        }

        if !unsolved.is_empty() || !self.residual_obligations.is_empty() {
            InferenceOutcome::Underconstrained(UnderconstrainedInference { unsolved_vars: unsolved })
        } else {
            InferenceOutcome::Solved(InferenceSolution {
                substitutions: self.substitutions.clone(),
                support: self
                    .variables
                    .iter()
                    .filter_map(|variable| variable.support.map(|support| (variable.id, support)))
                    .collect(),
            })
        }
    }

    pub fn term_substitute_variables(&self, term: &InferenceTerm) -> InferenceTerm {
        match term {
            InferenceTerm::Canonical(ty) => InferenceTerm::Canonical(*ty),
            InferenceTerm::Rigid(rigid) => InferenceTerm::Rigid(*rigid),
            InferenceTerm::Var(v) => {
                let rep = self.find_var(*v);
                if let Some(&ty) = self.substitutions.get(&rep) {
                    InferenceTerm::Canonical(ty)
                } else if rep != *v {
                    InferenceTerm::Var(rep)
                } else {
                    InferenceTerm::Var(*v)
                }
            }
            InferenceTerm::ExactCase { variant, enum_type } => InferenceTerm::ExactCase {
                variant: *variant,
                enum_type: Box::new(self.term_substitute_variables(enum_type)),
            },
            InferenceTerm::Applied { origin, arguments } => InferenceTerm::Applied {
                origin: Box::new(self.term_substitute_variables(origin)),
                arguments: arguments
                    .iter()
                    .map(|arg| match arg {
                        InferenceStaticArgument::Term(t) => InferenceStaticArgument::Term(self.term_substitute_variables(t)),
                        InferenceStaticArgument::RecordRow(r) => InferenceStaticArgument::RecordRow(r.clone()),
                    })
                    .collect::<Vec<_>>()
                    .into_boxed_slice(),
            },
            InferenceTerm::StaticData { domain, constructor, arguments } => InferenceTerm::StaticData {
                domain: *domain,
                constructor: *constructor,
                arguments: arguments
                    .iter()
                    .map(|arg| match arg {
                        InferenceStaticArgument::Term(t) => InferenceStaticArgument::Term(self.term_substitute_variables(t)),
                        InferenceStaticArgument::RecordRow(r) => InferenceStaticArgument::RecordRow(r.clone()),
                    })
                    .collect::<Vec<_>>()
                    .into_boxed_slice(),
            },
            InferenceTerm::Union(members) => {
                InferenceTerm::Union(members.iter().map(|m| self.term_substitute_variables(m)).collect::<Vec<_>>().into_boxed_slice())
            }
            InferenceTerm::Tuple(elems) => InferenceTerm::Tuple(
                elems
                    .iter()
                    .map(|e| InferenceTupleElement {
                        label: e.label.clone(),
                        term: self.term_substitute_variables(&e.term),
                    })
                    .collect::<Vec<_>>()
                    .into_boxed_slice(),
            ),
            InferenceTerm::Callable(c) => InferenceTerm::Callable(InferenceCallable {
                parameters: c
                    .parameters
                    .iter()
                    .map(|p| InferenceCallableParameter {
                        label: p.label.clone(),
                        term: self.term_substitute_variables(&p.term),
                        rest: p.rest,
                    })
                    .collect::<Vec<_>>()
                    .into_boxed_slice(),
                return_type: Box::new(self.term_substitute_variables(&c.return_type)),
            }),
            InferenceTerm::Record(r) => InferenceTerm::Record(InferenceRecord {
                fields: r
                    .fields
                    .iter()
                    .map(|f| InferenceRecordField {
                        name: f.name.clone(),
                        term: self.term_substitute_variables(&f.term),
                    })
                    .collect::<Vec<_>>()
                    .into_boxed_slice(),
                tail: r.tail.clone(),
            }),
            InferenceTerm::Family(family) => InferenceTerm::Family(
                family
                    .iter()
                    .map(|m| InferenceFamilyMember {
                        operation: m.operation.clone(),
                        member_kind: m.member_kind,
                        term: self.term_substitute_variables(&m.term),
                    })
                    .collect::<Vec<_>>()
                    .into_boxed_slice(),
            ),
            InferenceTerm::AssociatedProjection { subject, trait_ref, requirement } => {
                InferenceTerm::AssociatedProjection {
                    subject: Box::new(self.term_substitute_variables(subject)),
                    trait_ref: InferenceTraitRef {
                        declaration: trait_ref.declaration.clone(),
                        arguments: trait_ref.arguments.iter().map(|arg| self.term_substitute_variables(arg)).collect::<Vec<_>>().into_boxed_slice(),
                    },
                    requirement: requirement.clone(),
                }
            }
            InferenceTerm::TypeMemberProjection { subject, selector, arguments, source_declaration, result_kind } => {
                let subject = match subject {
                    InferenceProjectionSubject::Type(term) => InferenceProjectionSubject::Type(Box::new(self.term_substitute_variables(term))),
                    InferenceProjectionSubject::RecordRow(row) => InferenceProjectionSubject::RecordRow(row.clone()),
                };
                let arguments = arguments
                    .iter()
                    .map(|arg| match arg {
                        InferenceTypeMemberArgument::Type(term) => InferenceTypeMemberArgument::Type(self.term_substitute_variables(term)),
                        InferenceTypeMemberArgument::Const(c) => InferenceTypeMemberArgument::Const(c.clone()),
                        InferenceTypeMemberArgument::RecordRow(row) => InferenceTypeMemberArgument::RecordRow(row.clone()),
                    })
                    .collect::<Vec<_>>()
                    .into_boxed_slice();
                InferenceTerm::TypeMemberProjection {
                    subject,
                    selector: selector.clone(),
                    arguments,
                    source_declaration: source_declaration.clone(),
                    result_kind: *result_kind,
                }
            }
            InferenceTerm::TypePack(elements) => {
                InferenceTerm::TypePack(elements.iter().map(|e| self.term_substitute_variables(e)).collect::<Vec<_>>().into_boxed_slice())
            }
            InferenceTerm::LabeledTypePack(elements) => InferenceTerm::LabeledTypePack(
                elements
                    .iter()
                    .map(|e| InferenceLabeledTypePackElement {
                        label: e.label.clone(),
                        term: self.term_substitute_variables(&e.term),
                    })
                    .collect::<Vec<_>>()
                    .into_boxed_slice(),
            ),
            InferenceTerm::CompleteTypePack { positionals, labeled } => InferenceTerm::CompleteTypePack {
                positionals: Box::new(self.term_substitute_variables(positionals)),
                labeled: Box::new(self.term_substitute_variables(labeled)),
            },
            InferenceTerm::TypePackConcat(segments) => {
                InferenceTerm::TypePackConcat(segments.iter().map(|s| self.term_substitute_variables(s)).collect::<Vec<_>>().into_boxed_slice())
            }
            InferenceTerm::LabeledTypePackConcat(segments) => {
                InferenceTerm::LabeledTypePackConcat(segments.iter().map(|s| self.term_substitute_variables(s)).collect::<Vec<_>>().into_boxed_slice())
            }
            InferenceTerm::TupleFromPacks { positionals, labeled } => InferenceTerm::TupleFromPacks {
                positionals: Box::new(self.term_substitute_variables(positionals)),
                labeled: Box::new(self.term_substitute_variables(labeled)),
            },
            InferenceTerm::CallableFromPacks { positionals, labeled, return_type } => InferenceTerm::CallableFromPacks {
                positionals: Box::new(self.term_substitute_variables(positionals)),
                labeled: Box::new(self.term_substitute_variables(labeled)),
                return_type: Box::new(self.term_substitute_variables(return_type)),
            },
            InferenceTerm::PackProjection { subject, lane } => InferenceTerm::PackProjection {
                subject: Box::new(self.term_substitute_variables(subject)),
                lane: *lane,
            },
        }
    }

    pub fn term_contains_projection(&self, term: &InferenceTerm) -> bool {
        match term {
            InferenceTerm::AssociatedProjection { .. }
            | InferenceTerm::TypeMemberProjection { .. }
            | InferenceTerm::PackProjection { .. } => true,
            InferenceTerm::ExactCase { enum_type, .. } => self.term_contains_projection(enum_type),
            InferenceTerm::Applied { origin, arguments } => {
                self.term_contains_projection(origin)
                    || arguments.iter().any(|arg| match arg {
                        InferenceStaticArgument::Term(t) => self.term_contains_projection(t),
                        InferenceStaticArgument::RecordRow(_) => false,
                    })
            }
            InferenceTerm::StaticData { arguments, .. } => arguments.iter().any(|arg| match arg {
                InferenceStaticArgument::Term(t) => self.term_contains_projection(t),
                InferenceStaticArgument::RecordRow(_) => false,
            }),
            InferenceTerm::Union(members) => members.iter().any(|m| self.term_contains_projection(m)),
            InferenceTerm::Tuple(elems) => elems.iter().any(|e| self.term_contains_projection(&e.term)),
            InferenceTerm::Callable(c) => {
                c.parameters.iter().any(|p| self.term_contains_projection(&p.term)) || self.term_contains_projection(&c.return_type)
            }
            InferenceTerm::Record(r) => r.fields.iter().any(|f| self.term_contains_projection(&f.term)),
            InferenceTerm::Family(members) => members.iter().any(|m| self.term_contains_projection(&m.term)),
            InferenceTerm::TypePack(elements) => elements.iter().any(|e| self.term_contains_projection(e)),
            InferenceTerm::LabeledTypePack(elements) => elements.iter().any(|e| self.term_contains_projection(&e.term)),
            InferenceTerm::CompleteTypePack { positionals, labeled } => {
                self.term_contains_projection(positionals) || self.term_contains_projection(labeled)
            }
            InferenceTerm::TypePackConcat(segments) => segments.iter().any(|s| self.term_contains_projection(s)),
            InferenceTerm::LabeledTypePackConcat(segments) => segments.iter().any(|s| self.term_contains_projection(s)),
            InferenceTerm::TupleFromPacks { positionals, labeled } => {
                self.term_contains_projection(positionals) || self.term_contains_projection(labeled)
            }
            InferenceTerm::CallableFromPacks { positionals, labeled, return_type } => {
                self.term_contains_projection(positionals)
                    || self.term_contains_projection(labeled)
                    || self.term_contains_projection(return_type)
            }
            _ => false,
        }
    }

    /// Solves all accumulated constraints while consuming the caller's shared
    /// cancellation token and query budget.
    pub fn solve_with_control(&mut self, store: &mut TypeStore, hierarchy: &dyn TypeHierarchy, control: &CheckerControl) -> InferenceOutcome {
        match self.propagate_with_control(store, hierarchy, control) {
            Ok(_) => self.finish_root(),
            Err(outcome) => *outcome,
        }
    }

    pub(crate) fn failure_outcome(
        &mut self,
        failure: InferenceFailureReason,
        constraint_index: Option<u32>,
        origin: Option<ConstraintOrigin>,
    ) -> InferenceOutcome {
        self.failure_outcome_with_related(failure, constraint_index, origin, &[])
    }

    fn failure_outcome_with_related(
        &mut self,
        failure: InferenceFailureReason,
        constraint_index: Option<u32>,
        origin: Option<ConstraintOrigin>,
        related_constraint_indices: &[u32],
    ) -> InferenceOutcome {
        self.mark_failure(&failure);
        if matches!(failure, InferenceFailureReason::MissingVariableMetadata { .. }) {
            InferenceOutcome::InternalFailure(failure)
        } else {
            let mut constraint_indices = related_constraint_indices.to_vec();
            if let Some(index) = constraint_index {
                constraint_indices.push(index);
            }
            if let InferenceFailureReason::ConflictingBounds { var, lower, upper } = &failure {
                let rep = self.find_var(*var);
                for bound in [*lower, *upper] {
                    if let Some((index, _)) = self.bound_origins.get(&(rep, bound)) {
                        constraint_indices.push(*index);
                    }
                }
            }
            constraint_indices.sort_unstable();
            constraint_indices.dedup();
            constraint_indices.truncate(4);
            InferenceOutcome::Conflicting(InferenceConflict {
                constraint_indices: constraint_indices.into_boxed_slice(),
                constraint_index,
                origin,
                failure,
            })
        }
    }

    fn record_variable_constraint_indices(&mut self, relation: &InferenceRelation, constraint_index: u32) {
        let terms = match relation {
            InferenceRelation::Equivalent(left, right) | InferenceRelation::Subtype(left, right) => [left, right],
        };
        for term in terms {
            for variable in self.term_variables(term) {
                let representative = self.find_var(variable);
                let indices = self.variable_constraint_indices.entry(representative).or_default();
                if !indices.contains(&constraint_index) {
                    indices.push(constraint_index);
                }
            }
        }
    }

    fn related_constraint_indices(&self, relation: &InferenceRelation) -> Vec<u32> {
        let terms = match relation {
            InferenceRelation::Equivalent(left, right) | InferenceRelation::Subtype(left, right) => [left, right],
        };
        let mut indices = Vec::new();
        for term in terms {
            for variable in self.term_variables(term) {
                let representative = self.find_var(variable);
                if let Some(recorded) = self.variable_constraint_indices.get(&representative) {
                    indices.extend(recorded.iter().copied());
                }
            }
        }
        indices.sort_unstable();
        indices.dedup();
        indices
    }

    fn record_bound_origin(&mut self, relation: &InferenceRelation, origin: ConstraintOrigin, constraint_index: u32, store: &mut TypeStore) {
        let role = Self::role_for_origin(&origin);
        match relation {
            InferenceRelation::Subtype(InferenceTerm::Var(variable), term) => {
                let representative = self.find_var(*variable);
                if let Ok(ty) = self.materialize(term, store) {
                    self.bound_origins.entry((representative, ty)).or_insert((constraint_index, origin));
                    self.record_bound_role(representative, ty, role);
                }
            }
            InferenceRelation::Subtype(term, InferenceTerm::Var(variable)) => {
                let representative = self.find_var(*variable);
                if let Ok(ty) = self.materialize(term, store) {
                    self.bound_origins.entry((representative, ty)).or_insert((constraint_index, origin));
                    self.record_bound_role(representative, ty, role);
                }
            }
            _ => {}
        }
    }

    fn record_bound_role(&mut self, variable: InferVarId, ty: TypeId, role: InferenceConstraintRole) {
        let roles = self.bound_roles.entry((variable, ty)).or_default();
        if !roles.contains(&role) {
            roles.push(role);
        }
    }

    fn is_declaration_restriction_only(&self, variable: InferVarId, ty: TypeId) -> bool {
        let representative = self.find_var(variable);
        self.bound_roles
            .get(&(representative, ty))
            .is_some_and(|roles| !roles.is_empty() && roles.iter().all(|role| *role == InferenceConstraintRole::DeclarationRestriction))
    }

    fn context_selection_candidate(&self, variable: InferVarId, bounds: &[TypeId]) -> Option<TypeId> {
        let representative = self.find_var(variable);
        bounds.iter().copied().find(|ty| {
            self.bound_roles
                .get(&(representative, *ty))
                .is_some_and(|roles| roles.contains(&InferenceConstraintRole::ContextSelection))
        })
    }

    #[allow(clippy::result_large_err)]
    fn propagate_subtype_edges(
        &mut self,
        store: &mut TypeStore,
        hierarchy: &dyn TypeHierarchy,
        control: &CheckerControl,
    ) -> Result<SolveEffect, Box<InferenceOutcome>> {
        let edges = self.subtype_edges.clone();
        let mut changed = false;
        for edge in edges {
            if control.is_cancelled() {
                return Err(Box::new(InferenceOutcome::Cancelled));
            }
            if let Err(report) = control.charge_step() {
                return Err(Box::new(InferenceOutcome::BudgetExceeded(report)));
            }
            let sub = self.find_var(edge.sub);
            let sup = self.find_var(edge.sup);
            if sub == sup {
                continue;
            }

            let sub_ty = self.substitutions.get(&sub).copied();
            let sup_ty = self.substitutions.get(&sup).copied();
            match (sub_ty, sup_ty) {
                (Some(sub_ty), Some(sup_ty)) => {
                    if !is_subtype(store, hierarchy, sub_ty, sup_ty) {
                        let failure = InferenceFailureReason::StructuralMismatch {
                            left: Box::new(InferenceTerm::Var(sub)),
                            right: Box::new(InferenceTerm::Var(sup)),
                        };
                        return Err(Box::new(self.failure_outcome(failure, None, None)));
                    }
                }
                (Some(sub_ty), None) => {
                    let added = self.add_lower_bound(sup, sub_ty);
                    changed |= added;
                    if added {
                        self.propagate_variable_evidence(sub, sup);
                    }
                }
                (None, Some(sup_ty)) => {
                    changed |= self.add_upper_bound(sub, sup_ty);
                }
                (None, None) => {
                    let lower_bounds = self.lower_bounds.get(&sub).cloned().unwrap_or_default();
                    for lower in lower_bounds {
                        let added = self.add_lower_bound(sup, lower);
                        changed |= added;
                        if added {
                            self.propagate_variable_evidence(sub, sup);
                        }
                    }
                    let upper_bounds = self.upper_bounds.get(&sup).cloned().unwrap_or_default();
                    for upper in upper_bounds {
                        changed |= self.add_upper_bound(sub, upper);
                    }
                }
            }
        }
        Ok(if changed { SolveEffect::Changed } else { SolveEffect::Unchanged })
    }

    fn add_lower_bound(&mut self, variable: InferVarId, ty: TypeId) -> bool {
        let representative = self.find_var(variable);
        let bounds = self.lower_bounds.entry(representative).or_default();
        if bounds.contains(&ty) {
            false
        } else {
            bounds.push(ty);
            true
        }
    }

    fn add_upper_bound(&mut self, variable: InferVarId, ty: TypeId) -> bool {
        let representative = self.find_var(variable);
        let bounds = self.upper_bounds.entry(representative).or_default();
        if bounds.contains(&ty) {
            false
        } else {
            bounds.push(ty);
            true
        }
    }

    fn propagate_variable_evidence(&mut self, from: InferVarId, to: InferVarId) {
        let from_rep = self.find_var(from);
        let support = self.variable_by_representative(from_rep).and_then(|variable| variable.support);
        let proof = self.variable_by_representative(from_rep).and_then(|variable| variable.proof.clone());
        if let Some(support) = support {
            self.record_variable_support(to, support);
        }
        if let Some(proof) = proof {
            self.record_variable_proof(to, proof);
        }
    }

    /// Returns whether an inference term contains a generic variable.
    pub fn term_has_variables(&self, term: &InferenceTerm) -> bool {
        !self.term_variables(term).is_empty()
    }

    /// Solved variables remain in original terms for support/proof publication,
    /// but must not prevent materialized recursive bounds from reaching the
    /// canonical relation engine.
    fn term_has_unresolved_variables(&self, term: &InferenceTerm) -> bool {
        match term {
            InferenceTerm::Var(variable) => {
                let representative = self.find_var(*variable);
                !self.substitutions.contains_key(&representative)
            }
            InferenceTerm::Applied { origin, arguments } => {
                self.term_has_unresolved_variables(origin) || arguments.iter().any(|argument| self.static_argument_has_unresolved_variables(argument))
            }
            InferenceTerm::StaticData { arguments, .. } => arguments.iter().any(|argument| self.static_argument_has_unresolved_variables(argument)),
            InferenceTerm::ExactCase { enum_type, .. } => self.term_has_unresolved_variables(enum_type),
            InferenceTerm::Union(members) => members.iter().any(|member| self.term_has_unresolved_variables(member)),
            InferenceTerm::Tuple(elements) => elements.iter().any(|element| self.term_has_unresolved_variables(&element.term)),
            InferenceTerm::Callable(callable) => {
                callable.parameters.iter().any(|parameter| self.term_has_unresolved_variables(&parameter.term))
                    || self.term_has_unresolved_variables(&callable.return_type)
            }
            InferenceTerm::Record(record) => self.record_row_has_unresolved_variables(record),
            InferenceTerm::Family(family) => family.iter().any(|member| self.term_has_unresolved_variables(&member.term)),
            InferenceTerm::AssociatedProjection { subject, trait_ref, .. } => {
                self.term_has_unresolved_variables(subject) || trait_ref.arguments.iter().any(|arg| self.term_has_unresolved_variables(arg))
            }
            InferenceTerm::TypeMemberProjection { subject, arguments, .. } => {
                let subject_unresolved = match subject {
                    InferenceProjectionSubject::Type(term) => self.term_has_unresolved_variables(term),
                    InferenceProjectionSubject::RecordRow(row) => self.record_row_has_unresolved_variables(row),
                };
                subject_unresolved || arguments.iter().any(|argument| match argument {
                    InferenceTypeMemberArgument::Type(term) => self.term_has_unresolved_variables(term),
                    InferenceTypeMemberArgument::Const(_) => false,
                    InferenceTypeMemberArgument::RecordRow(row) => self.record_row_has_unresolved_variables(row),
                })
            }
            InferenceTerm::TypePack(elements) => elements.iter().any(|elem| self.term_has_unresolved_variables(elem)),
            InferenceTerm::LabeledTypePack(elements) => elements.iter().any(|elem| self.term_has_unresolved_variables(&elem.term)),
            InferenceTerm::CompleteTypePack { positionals, labeled } => {
                self.term_has_unresolved_variables(positionals) || self.term_has_unresolved_variables(labeled)
            }
            InferenceTerm::TypePackConcat(segments) | InferenceTerm::LabeledTypePackConcat(segments) => {
                segments.iter().any(|seg| self.term_has_unresolved_variables(seg))
            }
            InferenceTerm::TupleFromPacks { positionals, labeled } => {
                self.term_has_unresolved_variables(positionals) || self.term_has_unresolved_variables(labeled)
            }
            InferenceTerm::CallableFromPacks {
                positionals,
                labeled,
                return_type,
            } => {
                self.term_has_unresolved_variables(positionals)
                    || self.term_has_unresolved_variables(labeled)
                    || self.term_has_unresolved_variables(return_type)
            }
            InferenceTerm::PackProjection { subject, .. } => self.term_has_unresolved_variables(subject),
            InferenceTerm::Canonical(_) | InferenceTerm::Rigid(_) => false,
        }
    }

    fn static_argument_has_unresolved_variables(&self, argument: &InferenceStaticArgument) -> bool {
        match argument {
            InferenceStaticArgument::Term(term) => self.term_has_unresolved_variables(term),
            InferenceStaticArgument::RecordRow(row) => self.record_row_has_unresolved_variables(row),
        }
    }

    fn normalized_record_row(&self, record: &InferenceRecord) -> RecordRowTerm<InferenceTerm> {
        let term = record_term_for_row_solver(record);
        self.row_solver
            .upgrade()
            .map(|solver| solver.borrow().normalize_term(&term))
            .unwrap_or(term)
    }

    pub(crate) fn term_contains_row_variable(&self, term: &InferenceTerm, target: RecordRowVarId) -> bool {
        let Some(row_solver) = self.row_solver.upgrade() else {
            return false;
        };
        inference_term_contains_row_variable(
            term,
            target,
            &row_solver.borrow(),
            &self.var_terms,
            &self.var_aliases,
            &mut HashSet::new(),
        )
    }

    fn record_row_has_unresolved_variables(&self, record: &InferenceRecord) -> bool {
        let normalized = self.normalized_record_row(record);
        matches!(normalized.tail, RecordRowTermTail::Var(_))
            || normalized.fields.iter().any(|field| self.term_has_unresolved_variables(&field.ty))
    }

    /// Returns aggregate value support for all variables influencing a term.
    /// `None` means at least one influencing variable has no value support.
    pub fn term_support(&self, term: &InferenceTerm) -> Option<InferenceSupport> {
        let variables = self.term_variables(term);
        let mut support: Option<InferenceSupport> = None;
        for variable in variables {
            let current = self
                .variables
                .iter()
                .find(|candidate| self.find_var(candidate.id) == self.find_var(variable))
                .and_then(|candidate| candidate.support)?;
            support = Some(match support {
                Some(previous) => previous.join(current),
                None => current,
            });
        }
        support.or(Some(InferenceSupport::Established))
    }

    fn term_variables(&self, term: &InferenceTerm) -> HashSet<InferVarId> {
        let mut variables = HashSet::new();
        self.collect_term_variables(term, &mut variables);
        variables
    }

    fn collect_term_variables(&self, term: &InferenceTerm, variables: &mut HashSet<InferVarId>) {
        match term {
            InferenceTerm::Var(variable) => {
                variables.insert(self.find_var(*variable));
            }
            InferenceTerm::Applied { origin, arguments } => {
                self.collect_term_variables(origin, variables);
                for argument in arguments.iter() {
                    self.collect_static_argument_variables(argument, variables);
                }
            }
            InferenceTerm::StaticData { arguments, .. } => {
                for argument in arguments.iter() {
                    self.collect_static_argument_variables(argument, variables);
                }
            }
            InferenceTerm::ExactCase { enum_type, .. } => {
                self.collect_term_variables(enum_type, variables);
            }
            InferenceTerm::Union(members) => {
                for member in members.iter() {
                    self.collect_term_variables(member, variables);
                }
            }
            InferenceTerm::Tuple(elements) => {
                for element in elements.iter() {
                    self.collect_term_variables(&element.term, variables);
                }
            }
            InferenceTerm::Callable(callable) => {
                for parameter in callable.parameters.iter() {
                    self.collect_term_variables(&parameter.term, variables);
                }
                self.collect_term_variables(&callable.return_type, variables);
            }
            InferenceTerm::Record(record) => {
                for field in self.normalized_record_row(record).fields.iter() {
                    self.collect_term_variables(&field.ty, variables);
                }
            }
            InferenceTerm::Family(family) => {
                for member in family.iter() {
                    self.collect_term_variables(&member.term, variables);
                }
            }
            InferenceTerm::AssociatedProjection { subject, trait_ref, .. } => {
                self.collect_term_variables(subject, variables);
                for arg in trait_ref.arguments.iter() {
                    self.collect_term_variables(arg, variables);
                }
            }
            InferenceTerm::TypeMemberProjection { subject, arguments, .. } => {
                match subject {
                    InferenceProjectionSubject::Type(term) => self.collect_term_variables(term, variables),
                    InferenceProjectionSubject::RecordRow(row) => self.collect_record_variables(row, variables),
                }
                for argument in arguments.iter() {
                    match argument {
                        InferenceTypeMemberArgument::Type(term) => self.collect_term_variables(term, variables),
                        InferenceTypeMemberArgument::Const(_) => {}
                        InferenceTypeMemberArgument::RecordRow(row) => self.collect_record_variables(row, variables),
                    }
                }
            }
            InferenceTerm::TypePack(elements) => {
                for elem in elements.iter() {
                    self.collect_term_variables(elem, variables);
                }
            }
            InferenceTerm::LabeledTypePack(elements) => {
                for elem in elements.iter() {
                    self.collect_term_variables(&elem.term, variables);
                }
            }
            InferenceTerm::CompleteTypePack { positionals, labeled } => {
                self.collect_term_variables(positionals, variables);
                self.collect_term_variables(labeled, variables);
            }
            InferenceTerm::TypePackConcat(segments) | InferenceTerm::LabeledTypePackConcat(segments) => {
                for seg in segments.iter() {
                    self.collect_term_variables(seg, variables);
                }
            }
            InferenceTerm::TupleFromPacks { positionals, labeled } => {
                self.collect_term_variables(positionals, variables);
                self.collect_term_variables(labeled, variables);
            }
            InferenceTerm::CallableFromPacks {
                positionals,
                labeled,
                return_type,
            } => {
                self.collect_term_variables(positionals, variables);
                self.collect_term_variables(labeled, variables);
                self.collect_term_variables(return_type, variables);
            }
            InferenceTerm::PackProjection { subject, .. } => {
                self.collect_term_variables(subject, variables);
            }
            InferenceTerm::Canonical(_) | InferenceTerm::Rigid(_) => {}
        }
    }

    fn collect_static_argument_variables(&self, argument: &InferenceStaticArgument, variables: &mut HashSet<InferVarId>) {
        match argument {
            InferenceStaticArgument::Term(term) => self.collect_term_variables(term, variables),
            InferenceStaticArgument::RecordRow(row) => self.collect_record_variables(row, variables),
        }
    }

    fn collect_record_variables(&self, row: &InferenceRecord, variables: &mut HashSet<InferVarId>) {
        for field in self.normalized_record_row(row).fields.iter() {
            self.collect_term_variables(&field.ty, variables);
        }
    }

    fn record_relation_support(&mut self, relation: &InferenceRelation, support: InferenceSupport) {
        match relation {
            InferenceRelation::Equivalent(left, right) | InferenceRelation::Subtype(left, right) => {
                self.record_term_support(left, support);
                self.record_term_support(right, support);
            }
        }
    }

    fn record_term_proof_state(&mut self, term: &InferenceTerm, proof: InferenceProofState) {
        match term {
            InferenceTerm::Var(variable) => self.record_variable_proof(*variable, proof),
            InferenceTerm::Applied { origin, arguments } => {
                self.record_term_proof_state(origin, proof.clone());
                for argument in arguments.iter() {
                    self.record_static_argument_proof_state(argument, proof.clone());
                }
            }
            InferenceTerm::StaticData { arguments, .. } => {
                for argument in arguments.iter() {
                    self.record_static_argument_proof_state(argument, proof.clone());
                }
            }
            InferenceTerm::ExactCase { enum_type, .. } => {
                self.record_term_proof_state(enum_type, proof);
            }
            InferenceTerm::Union(members) => {
                for member in members.iter() {
                    self.record_term_proof_state(member, proof.clone());
                }
            }
            InferenceTerm::Tuple(elements) => {
                for element in elements.iter() {
                    self.record_term_proof_state(&element.term, proof.clone());
                }
            }
            InferenceTerm::Callable(callable) => {
                for parameter in callable.parameters.iter() {
                    self.record_term_proof_state(&parameter.term, proof.clone());
                }
                self.record_term_proof_state(&callable.return_type, proof);
            }
            InferenceTerm::Record(record) => {
                for field in self.normalized_record_row(record).fields.iter() {
                    self.record_term_proof_state(&field.ty, proof.clone());
                }
            }
            InferenceTerm::Family(family) => {
                for member in family.iter() {
                    self.record_term_proof_state(&member.term, proof.clone());
                }
            }
            InferenceTerm::AssociatedProjection { subject, trait_ref, .. } => {
                self.record_term_proof_state(subject, proof.clone());
                for arg in trait_ref.arguments.iter() {
                    self.record_term_proof_state(arg, proof.clone());
                }
            }
            InferenceTerm::TypeMemberProjection { subject, arguments, .. } => {
                match subject {
                    InferenceProjectionSubject::Type(term) => self.record_term_proof_state(term, proof.clone()),
                    InferenceProjectionSubject::RecordRow(row) => self.record_record_proof_state(row, proof.clone()),
                }
                for argument in arguments.iter() {
                    match argument {
                        InferenceTypeMemberArgument::Type(term) => self.record_term_proof_state(term, proof.clone()),
                        InferenceTypeMemberArgument::Const(_) => {}
                        InferenceTypeMemberArgument::RecordRow(row) => self.record_record_proof_state(row, proof.clone()),
                    }
                }
            }
            InferenceTerm::TypePack(elements) => {
                for elem in elements.iter() {
                    self.record_term_proof_state(elem, proof.clone());
                }
            }
            InferenceTerm::LabeledTypePack(elements) => {
                for elem in elements.iter() {
                    self.record_term_proof_state(&elem.term, proof.clone());
                }
            }
            InferenceTerm::CompleteTypePack { positionals, labeled } => {
                self.record_term_proof_state(positionals, proof.clone());
                self.record_term_proof_state(labeled, proof);
            }
            InferenceTerm::TypePackConcat(segments) | InferenceTerm::LabeledTypePackConcat(segments) => {
                for seg in segments.iter() {
                    self.record_term_proof_state(seg, proof.clone());
                }
            }
            InferenceTerm::TupleFromPacks { positionals, labeled } => {
                self.record_term_proof_state(positionals, proof.clone());
                self.record_term_proof_state(labeled, proof);
            }
            InferenceTerm::CallableFromPacks {
                positionals,
                labeled,
                return_type,
            } => {
                self.record_term_proof_state(positionals, proof.clone());
                self.record_term_proof_state(labeled, proof.clone());
                self.record_term_proof_state(return_type, proof);
            }
            InferenceTerm::PackProjection { subject, .. } => {
                self.record_term_proof_state(subject, proof);
            }
            InferenceTerm::Canonical(_) | InferenceTerm::Rigid(_) => {}
        }
    }

    fn record_static_argument_proof_state(&mut self, argument: &InferenceStaticArgument, proof: InferenceProofState) {
        match argument {
            InferenceStaticArgument::Term(term) => self.record_term_proof_state(term, proof),
            InferenceStaticArgument::RecordRow(row) => self.record_record_proof_state(row, proof),
        }
    }

    fn record_record_proof_state(&mut self, row: &InferenceRecord, proof: InferenceProofState) {
        for field in self.normalized_record_row(row).fields.iter() {
            self.record_term_proof_state(&field.ty, proof.clone());
        }
    }

    fn record_term_support(&mut self, term: &InferenceTerm, support: InferenceSupport) {
        match term {
            InferenceTerm::Var(variable) => self.record_variable_support(*variable, support),
            InferenceTerm::Applied { origin, arguments } => {
                self.record_term_support(origin, support);
                for argument in arguments.iter() {
                    self.record_static_argument_support(argument, support);
                }
            }
            InferenceTerm::StaticData { arguments, .. } => {
                for argument in arguments.iter() {
                    self.record_static_argument_support(argument, support);
                }
            }
            InferenceTerm::ExactCase { enum_type, .. } => {
                self.record_term_support(enum_type, support);
            }
            InferenceTerm::Union(members) => {
                for member in members.iter() {
                    self.record_term_support(member, support);
                }
            }
            InferenceTerm::Tuple(elements) => {
                for element in elements.iter() {
                    self.record_term_support(&element.term, support);
                }
            }
            InferenceTerm::Callable(callable) => {
                for parameter in callable.parameters.iter() {
                    self.record_term_support(&parameter.term, support);
                }
                self.record_term_support(&callable.return_type, support);
            }
            InferenceTerm::Record(record) => {
                for field in self.normalized_record_row(record).fields.iter() {
                    self.record_term_support(&field.ty, support);
                }
            }
            InferenceTerm::Family(family) => {
                for member in family.iter() {
                    self.record_term_support(&member.term, support);
                }
            }
            InferenceTerm::AssociatedProjection { subject, trait_ref, .. } => {
                self.record_term_support(subject, support);
                for arg in trait_ref.arguments.iter() {
                    self.record_term_support(arg, support);
                }
            }
            InferenceTerm::TypeMemberProjection { subject, arguments, .. } => {
                match subject {
                    InferenceProjectionSubject::Type(term) => self.record_term_support(term, support),
                    InferenceProjectionSubject::RecordRow(row) => self.record_record_support(row, support),
                }
                for argument in arguments.iter() {
                    match argument {
                        InferenceTypeMemberArgument::Type(term) => self.record_term_support(term, support),
                        InferenceTypeMemberArgument::Const(_) => {}
                        InferenceTypeMemberArgument::RecordRow(row) => self.record_record_support(row, support),
                    }
                }
            }
            InferenceTerm::TypePack(elements) => {
                for elem in elements.iter() {
                    self.record_term_support(elem, support);
                }
            }
            InferenceTerm::LabeledTypePack(elements) => {
                for elem in elements.iter() {
                    self.record_term_support(&elem.term, support);
                }
            }
            InferenceTerm::CompleteTypePack { positionals, labeled } => {
                self.record_term_support(positionals, support);
                self.record_term_support(labeled, support);
            }
            InferenceTerm::TypePackConcat(segments) | InferenceTerm::LabeledTypePackConcat(segments) => {
                for seg in segments.iter() {
                    self.record_term_support(seg, support);
                }
            }
            InferenceTerm::TupleFromPacks { positionals, labeled } => {
                self.record_term_support(positionals, support);
                self.record_term_support(labeled, support);
            }
            InferenceTerm::CallableFromPacks {
                positionals,
                labeled,
                return_type,
            } => {
                self.record_term_support(positionals, support);
                self.record_term_support(labeled, support);
                self.record_term_support(return_type, support);
            }
            InferenceTerm::PackProjection { subject, .. } => {
                self.record_term_support(subject, support);
            }
            InferenceTerm::Canonical(_) | InferenceTerm::Rigid(_) => {}
        }
    }

    fn record_static_argument_support(&mut self, argument: &InferenceStaticArgument, support: InferenceSupport) {
        match argument {
            InferenceStaticArgument::Term(term) => self.record_term_support(term, support),
            InferenceStaticArgument::RecordRow(row) => self.record_record_support(row, support),
        }
    }

    fn record_record_support(&mut self, row: &InferenceRecord, support: InferenceSupport) {
        for field in self.normalized_record_row(row).fields.iter() {
            self.record_term_support(&field.ty, support);
        }
    }

    fn record_variable_support(&mut self, variable: InferVarId, support: InferenceSupport) {
        let representative = self.find_var(variable);
        if let Some(candidate) = self.variables.iter_mut().find(|candidate| candidate.id == representative) {
            candidate.support = Some(match candidate.support {
                Some(previous) => previous.join(support),
                None => support,
            });
        }
    }

    fn record_variable_proof(&mut self, variable: InferVarId, proof: InferenceProofState) {
        let representative = self.find_var(variable);
        if let Some(candidate) = self.variables.iter_mut().find(|candidate| candidate.id == representative) {
            candidate.proof = Some(match candidate.proof.take() {
                Some(previous) => previous.meet(proof),
                None => proof,
            });
        }
    }

    fn mark_failure(&mut self, failure: &InferenceFailureReason) {
        let var = match failure {
            InferenceFailureReason::OccursCheck { var }
            | InferenceFailureReason::KindMismatch { var, .. }
            | InferenceFailureReason::ConflictingBounds { var, .. }
            | InferenceFailureReason::MissingVariableMetadata { var } => Some(*var),
            InferenceFailureReason::StructuralMismatch { .. }
            | InferenceFailureReason::RowRejected(_)
            | InferenceFailureReason::UnresolvedSelf
            | InferenceFailureReason::InternalRowSolverFailure { .. } => None,
        };
        if let Some(var) = var {
            let rep = self.find_var(var);
            if let Some(state) = self.variables.iter_mut().find(|candidate| candidate.id == rep) {
                state.state = InferVarState::Failed(failure.clone());
            }
        }
    }

    /// Binds an inference variable to a canonical type, with occurs and kind checks.
    pub fn bind(&mut self, var: InferVarId, ty: TypeId, store: &TypeStore) -> Result<SolveEffect, InferenceFailureReason> {
        let rep = self.find_var(var);
        if self.occurs_in_type(rep, ty, store) {
            return Err(InferenceFailureReason::OccursCheck { var: rep });
        }
        let expected_kind = self
            .variables
            .iter()
            .find(|candidate| candidate.id == rep)
            .map(|candidate| candidate.kind)
            .ok_or(InferenceFailureReason::MissingVariableMetadata { var: rep })?;
        let actual_kind = store.kind_of(ty);
        if expected_kind != actual_kind {
            return Err(InferenceFailureReason::KindMismatch {
                var: rep,
                expected: expected_kind,
                actual: actual_kind,
            });
        }
        if self.substitutions.get(&rep).copied() == Some(ty) {
            return Ok(SolveEffect::Unchanged);
        }
        self.substitutions.insert(rep, ty);
        if let Some(v) = self.variables.iter_mut().find(|v| v.id == rep) {
            v.state = InferVarState::Solved(ty);
        }
        Ok(SolveEffect::Changed)
    }

    /// Checks if `var` occurs in `ty`.
    #[allow(clippy::only_used_in_recursion)]
    pub fn occurs_in_type(&self, var: InferVarId, ty: TypeId, store: &TypeStore) -> bool {
        match store.get(ty) {
            TypeData::Applied { origin, arguments } => {
                self.occurs_in_type(var, *origin, store)
                    || arguments.iter().any(|argument| argument.as_type_id().is_some_and(|a| self.occurs_in_type(var, a, store)))
            }
            TypeData::ExactCase { enum_type, .. } => self.occurs_in_type(var, *enum_type, store),
            TypeData::Union(members) => members.iter().any(|&m| self.occurs_in_type(var, m, store)),
            TypeData::Tuple(elems) => elems.iter().any(|e| self.occurs_in_type(var, e.ty, store)),
            TypeData::Record(row_id) => store.record_row(*row_id).fields.iter().any(|f| self.occurs_in_type(var, f.ty, store)),
            TypeData::Callable(c) => c.parameters.iter().any(|p| self.occurs_in_type(var, p.ty, store)) || self.occurs_in_type(var, c.return_type, store),
            TypeData::Family(fid) => store.get_family(*fid).members.iter().any(|m| self.occurs_in_type(var, m.ty, store)),
            TypeData::AssociatedProjection(proj) => {
                self.occurs_in_type(var, proj.subject, store) || proj.trait_ref.arguments.iter().any(|&a| self.occurs_in_type(var, a, store))
            }
            _ => false,
        }
    }

    /// Checks if `var` occurs in an `InferenceTerm`.
    pub fn occurs_in_term(&self, var: InferVarId, term: &InferenceTerm) -> bool {
        let rep = self.find_var(var);
        match term {
            InferenceTerm::Canonical(_) => false,
            InferenceTerm::Rigid(_) => false,
            InferenceTerm::Var(v) => self.find_var(*v) == rep,
            InferenceTerm::Applied { origin, arguments } => self.occurs_in_term(rep, origin) || arguments.iter().any(|argument| self.occurs_in_static_argument(rep, argument)),
            InferenceTerm::StaticData { arguments, .. } => arguments.iter().any(|argument| self.occurs_in_static_argument(rep, argument)),
            InferenceTerm::ExactCase { enum_type, .. } => self.occurs_in_term(rep, enum_type),
            InferenceTerm::Union(members) => members.iter().any(|m| self.occurs_in_term(rep, m)),
            InferenceTerm::Tuple(elems) => elems.iter().any(|e| self.occurs_in_term(rep, &e.term)),
            InferenceTerm::Callable(c) => c.parameters.iter().any(|p| self.occurs_in_term(rep, &p.term)) || self.occurs_in_term(rep, &c.return_type),
            InferenceTerm::Record(record) => self.normalized_record_row(record).fields.iter().any(|field| self.occurs_in_term(rep, &field.ty)),
            InferenceTerm::Family(family) => family.iter().any(|member| self.occurs_in_term(rep, &member.term)),
            InferenceTerm::AssociatedProjection { subject, trait_ref, .. } => {
                self.occurs_in_term(rep, subject) || trait_ref.arguments.iter().any(|arg| self.occurs_in_term(rep, arg))
            }
            InferenceTerm::TypeMemberProjection { subject, arguments, .. } => {
                let in_subject = match subject {
                    InferenceProjectionSubject::Type(term) => self.occurs_in_term(rep, term),
                    InferenceProjectionSubject::RecordRow(row) => self.occurs_in_record(rep, row),
                };
                in_subject || arguments.iter().any(|argument| match argument {
                    InferenceTypeMemberArgument::Type(term) => self.occurs_in_term(rep, term),
                    InferenceTypeMemberArgument::Const(_) => false,
                    InferenceTypeMemberArgument::RecordRow(row) => self.occurs_in_record(rep, row),
                })
            }
            InferenceTerm::TypePack(elements) => elements.iter().any(|elem| self.occurs_in_term(rep, elem)),
            InferenceTerm::LabeledTypePack(elements) => elements.iter().any(|elem| self.occurs_in_term(rep, &elem.term)),
            InferenceTerm::CompleteTypePack { positionals, labeled } => self.occurs_in_term(rep, positionals) || self.occurs_in_term(rep, labeled),
            InferenceTerm::TypePackConcat(segments) | InferenceTerm::LabeledTypePackConcat(segments) => {
                segments.iter().any(|seg| self.occurs_in_term(rep, seg))
            }
            InferenceTerm::TupleFromPacks { positionals, labeled } => self.occurs_in_term(rep, positionals) || self.occurs_in_term(rep, labeled),
            InferenceTerm::CallableFromPacks {
                positionals,
                labeled,
                return_type,
            } => self.occurs_in_term(rep, positionals) || self.occurs_in_term(rep, labeled) || self.occurs_in_term(rep, return_type),
            InferenceTerm::PackProjection { subject, .. } => self.occurs_in_term(rep, subject),
        }
    }

    fn occurs_in_static_argument(&self, variable: InferVarId, argument: &InferenceStaticArgument) -> bool {
        match argument {
            InferenceStaticArgument::Term(term) => self.occurs_in_term(variable, term),
            InferenceStaticArgument::RecordRow(row) => self.occurs_in_record(variable, row),
        }
    }

    fn occurs_in_record(&self, variable: InferVarId, row: &InferenceRecord) -> bool {
        self.normalized_record_row(row).fields.iter().any(|field| self.occurs_in_term(variable, &field.ty))
    }

    /// Checks if `term` contains an associated type projection.
    pub fn term_has_projection(&self, term: &InferenceTerm, store: &TypeStore) -> bool {
        match term {
            InferenceTerm::AssociatedProjection { .. } | InferenceTerm::TypeMemberProjection { .. } => true,
            InferenceTerm::Canonical(ty) => matches!(store.get(*ty), TypeData::AssociatedProjection(_) | TypeData::TypeMemberProjection(_)),
            InferenceTerm::Var(v) => {
                let rep = self.find_var(*v);
                if let Some(ty) = self.substitutions.get(&rep).copied() {
                    matches!(store.get(ty), TypeData::AssociatedProjection(_) | TypeData::TypeMemberProjection(_))
                } else if let Some(t) = self.var_terms.get(&rep) {
                    self.term_has_projection(t, store)
                } else {
                    false
                }
            }
            InferenceTerm::Applied { origin, arguments } => {
                self.term_has_projection(origin, store) || arguments.iter().any(|argument| self.static_argument_has_projection(argument, store))
            }
            InferenceTerm::StaticData { arguments, .. } => arguments.iter().any(|argument| self.static_argument_has_projection(argument, store)),
            InferenceTerm::ExactCase { enum_type, .. } => self.term_has_projection(enum_type, store),
            InferenceTerm::Union(members) => members.iter().any(|m| self.term_has_projection(m, store)),
            InferenceTerm::Tuple(elems) => elems.iter().any(|e| self.term_has_projection(&e.term, store)),
            InferenceTerm::Callable(c) => {
                c.parameters.iter().any(|p| self.term_has_projection(&p.term, store)) || self.term_has_projection(&c.return_type, store)
            }
            InferenceTerm::Record(record) => self.normalized_record_row(record).fields.iter().any(|field| self.term_has_projection(&field.ty, store)),
            InferenceTerm::Family(family) => family.iter().any(|member| self.term_has_projection(&member.term, store)),
            InferenceTerm::TypePack(elements) => elements.iter().any(|elem| self.term_has_projection(elem, store)),
            InferenceTerm::LabeledTypePack(elements) => elements.iter().any(|elem| self.term_has_projection(&elem.term, store)),
            InferenceTerm::CompleteTypePack { positionals, labeled } => {
                self.term_has_projection(positionals, store) || self.term_has_projection(labeled, store)
            }
            InferenceTerm::TypePackConcat(segments) | InferenceTerm::LabeledTypePackConcat(segments) => {
                segments.iter().any(|seg| self.term_has_projection(seg, store))
            }
            InferenceTerm::TupleFromPacks { positionals, labeled } => self.term_has_projection(positionals, store) || self.term_has_projection(labeled, store),
            InferenceTerm::CallableFromPacks {
                positionals,
                labeled,
                return_type,
            } => self.term_has_projection(positionals, store) || self.term_has_projection(labeled, store) || self.term_has_projection(return_type, store),
            InferenceTerm::PackProjection { subject, .. } => self.term_has_projection(subject, store),
            InferenceTerm::Rigid(_) => false,
        }
    }

    fn static_argument_has_projection(&self, argument: &InferenceStaticArgument, store: &TypeStore) -> bool {
        match argument {
            InferenceStaticArgument::Term(term) => self.term_has_projection(term, store),
            InferenceStaticArgument::RecordRow(row) => self.normalized_record_row(row).fields.iter().any(|field| self.term_has_projection(&field.ty, store)),
        }
    }

    fn inference_tuple_lanes(elements: &[InferenceTupleElement]) -> (InferenceTerm, InferenceTerm) {
        let mut pos = Vec::new();
        let mut lab = Vec::new();
        for elem in elements {
            if let Some(label) = &elem.label {
                lab.push(InferenceLabeledTypePackElement {
                    label: label.clone(),
                    term: elem.term.clone(),
                });
            } else {
                pos.push(elem.term.clone());
            }
        }
        (
            InferenceTerm::TypePack(pos.into_boxed_slice()),
            InferenceTerm::LabeledTypePack(lab.into_boxed_slice()),
        )
    }

    fn inference_callable_lanes(callable: &InferenceCallable) -> Option<(InferenceTerm, InferenceTerm)> {
        let mut pos = Vec::new();
        let mut lab = Vec::new();
        for param in callable.parameters.iter() {
            if param.rest != RestMode::None {
                return None;
            }
            if let Some(label) = &param.label {
                lab.push(InferenceLabeledTypePackElement {
                    label: label.clone(),
                    term: param.term.clone(),
                });
            } else {
                pos.push(param.term.clone());
            }
        }
        Some((
            InferenceTerm::TypePack(pos.into_boxed_slice()),
            InferenceTerm::LabeledTypePack(lab.into_boxed_slice()),
        ))
    }

    fn unify_static_arguments(
        &mut self,
        left: &InferenceStaticArgument,
        right: &InferenceStaticArgument,
        outer_left: &InferenceTerm,
        outer_right: &InferenceTerm,
        store: &mut TypeStore,
    ) -> Result<SolveEffect, InferenceFailureReason> {
        match (left, right) {
            (InferenceStaticArgument::Term(left), InferenceStaticArgument::Term(right)) => self.unify_terms(left, right, store),
            (InferenceStaticArgument::RecordRow(left), InferenceStaticArgument::RecordRow(right)) => {
                Ok(self.enqueue_row_equation(left, right, outer_left, outer_right))
            }
            _ => Err(InferenceFailureReason::StructuralMismatch {
                left: Box::new(outer_left.clone()),
                right: Box::new(outer_right.clone()),
            }),
        }
    }

    fn unify_terms(&mut self, left: &InferenceTerm, right: &InferenceTerm, store: &mut TypeStore) -> Result<SolveEffect, InferenceFailureReason> {
        if let Some(reduced) = self.symbolic_beta_reduce(left, store) {
            return self.unify_terms(&reduced, right, store);
        }
        if let Some(reduced) = self.symbolic_beta_reduce(right, store) {
            return self.unify_terms(left, &reduced, store);
        }
        match (left, right) {
            (InferenceTerm::Canonical(ty), inferred @ InferenceTerm::Applied { .. })
            | (inferred @ InferenceTerm::Applied { .. }, InferenceTerm::Canonical(ty))
                if matches!(store.get(*ty), TypeData::Applied { .. }) =>
            {
                let canonical = self.type_id_to_inference(*ty, &HashMap::new(), store);
                self.unify_terms(&canonical, inferred, store)
            }
            (InferenceTerm::Canonical(ty), inferred @ InferenceTerm::StaticData { .. })
            | (inferred @ InferenceTerm::StaticData { .. }, InferenceTerm::Canonical(ty))
                if matches!(store.get(*ty), TypeData::StaticData(_)) =>
            {
                let canonical = self.type_id_to_inference(*ty, &HashMap::new(), store);
                self.unify_terms(&canonical, inferred, store)
            }
            (InferenceTerm::Rigid(left), InferenceTerm::Rigid(right)) => {
                if left == right {
                    Ok(SolveEffect::Unchanged)
                } else {
                    Err(InferenceFailureReason::StructuralMismatch {
                        left: Box::new(InferenceTerm::Rigid(*left)),
                        right: Box::new(InferenceTerm::Rigid(*right)),
                    })
                }
            }
            (InferenceTerm::Var(v1), InferenceTerm::Var(v2)) => {
                let rep1 = self.find_var(*v1);
                let rep2 = self.find_var(*v2);
                if rep1 == rep2 {
                    return Ok(SolveEffect::Unchanged);
                }
                if let Some(ty1) = self.substitutions.get(&rep1).copied() {
                    let canon = InferenceTerm::Canonical(ty1);
                    return self.unify_terms(&canon, right, store);
                }
                if let Some(ty2) = self.substitutions.get(&rep2).copied() {
                    let canon = InferenceTerm::Canonical(ty2);
                    return self.unify_terms(left, &canon, store);
                }
                let kind1 = self
                    .variables
                    .iter()
                    .find(|candidate| candidate.id == rep1)
                    .map(|candidate| candidate.kind)
                    .ok_or(InferenceFailureReason::MissingVariableMetadata { var: rep1 })?;
                let kind2 = self
                    .variables
                    .iter()
                    .find(|candidate| candidate.id == rep2)
                    .map(|candidate| candidate.kind)
                    .ok_or(InferenceFailureReason::MissingVariableMetadata { var: rep2 })?;
                if kind1 != kind2 {
                    return Err(InferenceFailureReason::KindMismatch {
                        var: rep1,
                        expected: kind1,
                        actual: kind2,
                    });
                }
                let support = self
                    .variables
                    .iter()
                    .find(|candidate| candidate.id == rep1)
                    .and_then(|candidate| candidate.support)
                    .into_iter()
                    .chain(
                        self.variables
                            .iter()
                            .find(|candidate| candidate.id == rep2)
                            .and_then(|candidate| candidate.support),
                    )
                    .reduce(InferenceSupport::join);
                let proof = self
                    .variables
                    .iter()
                    .filter(|candidate| candidate.id == rep1 || candidate.id == rep2)
                    .filter_map(|candidate| candidate.proof.clone())
                    .reduce(InferenceProofState::meet);
                self.var_aliases.insert(rep1, rep2);
                if let Some(support) = support {
                    self.record_variable_support(rep2, support);
                }
                if let Some(proof) = proof {
                    if let Some(candidate) = self.variables.iter_mut().find(|candidate| candidate.id == rep2) {
                        candidate.proof = Some(proof);
                    }
                }
                // Merge bounds
                if let Some(lowers) = self.lower_bounds.remove(&rep1) {
                    self.lower_bounds.entry(rep2).or_default().extend(lowers);
                }
                if let Some(uppers) = self.upper_bounds.remove(&rep1) {
                    self.upper_bounds.entry(rep2).or_default().extend(uppers);
                }
                let origins = std::mem::take(&mut self.bound_origins);
                for ((variable, ty), origin) in origins {
                    let representative = if variable == rep1 { rep2 } else { variable };
                    self.bound_origins.entry((representative, ty)).or_insert(origin);
                }
                let roles = std::mem::take(&mut self.bound_roles);
                for ((variable, ty), roles_for_bound) in roles {
                    let representative = if variable == rep1 { rep2 } else { variable };
                    let target = self.bound_roles.entry((representative, ty)).or_default();
                    for role in roles_for_bound {
                        if !target.contains(&role) {
                            target.push(role);
                        }
                    }
                }
                if let Some(indices) = self.variable_constraint_indices.remove(&rep1) {
                    let target = self.variable_constraint_indices.entry(rep2).or_default();
                    target.extend(indices);
                    target.sort_unstable();
                    target.dedup();
                }
                Ok(SolveEffect::Changed)
            }
            (InferenceTerm::Var(v), term) | (term, InferenceTerm::Var(v)) => {
                let rep = self.find_var(*v);
                if let Some(ty) = self.substitutions.get(&rep).copied() {
                    let canon = InferenceTerm::Canonical(ty);
                    self.unify_terms(&canon, term, store)
                } else if self.occurs_in_term(rep, term) {
                    Err(InferenceFailureReason::OccursCheck { var: rep })
                } else if !self.term_has_projection(term, store)
                    && let Ok(ty) = self.materialize(term, store)
                {
                    self.bind(rep, ty, store)
                } else {
                    // Constraint replay is expected to revisit the same
                    // deferred term. Only publish progress for a new term.
                    if self.var_terms.get(&rep) == Some(term) {
                        Ok(SolveEffect::Unchanged)
                    } else {
                        self.var_terms.insert(rep, term.clone());
                        Ok(SolveEffect::Changed)
                    }
                }
            }
            (InferenceTerm::Canonical(t1), InferenceTerm::Canonical(t2)) => {
                if t1 == t2 {
                    Ok(SolveEffect::Unchanged)
                } else {
                    Err(InferenceFailureReason::StructuralMismatch {
                        left: Box::new(left.clone()),
                        right: Box::new(right.clone()),
                    })
                }
            }
            (InferenceTerm::ExactCase { variant: v1, enum_type: e1 }, InferenceTerm::ExactCase { variant: v2, enum_type: e2 }) => {
                if v1 != v2 {
                    return Err(InferenceFailureReason::StructuralMismatch {
                        left: Box::new(left.clone()),
                        right: Box::new(right.clone()),
                    });
                }
                self.unify_terms(e1, e2, store)
            }
            (InferenceTerm::Canonical(ty), InferenceTerm::ExactCase { variant, enum_type })
            | (InferenceTerm::ExactCase { variant, enum_type }, InferenceTerm::Canonical(ty)) => {
                if let TypeData::ExactCase {
                    variant: canon_var,
                    enum_type: canon_enum,
                } = store.get(*ty).clone()
                {
                    if canon_var != *variant {
                        return Err(InferenceFailureReason::StructuralMismatch {
                            left: Box::new(left.clone()),
                            right: Box::new(right.clone()),
                        });
                    }
                    let canon_enum_term = InferenceTerm::Canonical(canon_enum);
                    self.unify_terms(enum_type, &canon_enum_term, store)
                } else {
                    Err(InferenceFailureReason::StructuralMismatch {
                        left: Box::new(left.clone()),
                        right: Box::new(right.clone()),
                    })
                }
            }
            (InferenceTerm::Applied { origin: o1, arguments: a1 }, InferenceTerm::Applied { origin: o2, arguments: a2 }) => {
                if let (Some(left_terms), Some(right_terms)) = (static_terms(a1), static_terms(a2)) {
                    if let Some(effect) = self.match_applied_constructor_shapes(o1, &left_terms, o2, &right_terms, store)? {
                        return Ok(effect);
                    }
                }
                if a1.len() != a2.len() {
                    return Err(InferenceFailureReason::StructuralMismatch {
                        left: Box::new(left.clone()),
                        right: Box::new(right.clone()),
                    });
                }
                let mut changed = self.unify_terms(o1, o2, store)?.is_changed();
                for (arg1, arg2) in a1.iter().zip(a2.iter()) {
                    changed |= self.unify_static_arguments(arg1, arg2, left, right, store)?.is_changed();
                }
                Ok(if changed { SolveEffect::Changed } else { SolveEffect::Unchanged })
            }
            (
                InferenceTerm::StaticData {
                    domain: d1,
                    constructor: c1,
                    arguments: a1,
                },
                InferenceTerm::StaticData {
                    domain: d2,
                    constructor: c2,
                    arguments: a2,
                },
            ) => {
                if d1 != d2 || c1 != c2 || a1.len() != a2.len() {
                    return Err(InferenceFailureReason::StructuralMismatch {
                        left: Box::new(left.clone()),
                        right: Box::new(right.clone()),
                    });
                }
                let mut changed = false;
                for (left_argument, right_argument) in a1.iter().zip(a2.iter()) {
                    changed |= self.unify_static_arguments(left_argument, right_argument, left, right, store)?.is_changed();
                }
                Ok(if changed { SolveEffect::Changed } else { SolveEffect::Unchanged })
            }
            (InferenceTerm::Canonical(ty), InferenceTerm::Applied { origin, arguments })
            | (InferenceTerm::Applied { origin, arguments }, InferenceTerm::Canonical(ty)) => {
                if let TypeData::ExactCase { enum_type, .. } = store.get(*ty).clone() {
                    let applied = InferenceTerm::Applied {
                        origin: origin.clone(),
                        arguments: arguments.clone(),
                    };
                    return self.unify_terms(&InferenceTerm::Canonical(enum_type), &applied, store);
                }
                Err(InferenceFailureReason::StructuralMismatch {
                    left: Box::new(left.clone()),
                    right: Box::new(right.clone()),
                })
            }
            (InferenceTerm::Canonical(ty), InferenceTerm::Callable(callable)) | (InferenceTerm::Callable(callable), InferenceTerm::Canonical(ty)) => {
                let TypeData::Callable(canonical) = store.get(*ty).clone() else {
                    return Err(InferenceFailureReason::StructuralMismatch {
                        left: Box::new(left.clone()),
                        right: Box::new(right.clone()),
                    });
                };
                let canonical_term = InferenceTerm::Callable(InferenceCallable {
                    parameters: canonical
                        .parameters
                        .iter()
                        .map(|parameter| InferenceCallableParameter {
                            label: parameter.label.clone(),
                            term: InferenceTerm::Canonical(parameter.ty),
                            rest: parameter.rest,
                        })
                        .collect::<Vec<_>>()
                        .into_boxed_slice(),
                    return_type: Box::new(InferenceTerm::Canonical(canonical.return_type)),
                });
                self.unify_terms(&canonical_term, &InferenceTerm::Callable(callable.clone()), store)
            }
            (InferenceTerm::Canonical(ty), InferenceTerm::Tuple(tuple)) | (InferenceTerm::Tuple(tuple), InferenceTerm::Canonical(ty)) => {
                let TypeData::Tuple(canonical) = store.get(*ty).clone() else {
                    return Err(InferenceFailureReason::StructuralMismatch {
                        left: Box::new(left.clone()),
                        right: Box::new(right.clone()),
                    });
                };
                if canonical.len() != tuple.len()
                    || canonical
                        .iter()
                        .zip(tuple.iter())
                        .any(|(canonical, inferred)| canonical.label != inferred.label)
                {
                    return Err(InferenceFailureReason::StructuralMismatch {
                        left: Box::new(left.clone()),
                        right: Box::new(right.clone()),
                    });
                }
                let mut changed = false;
                for (canonical, inferred) in canonical.iter().zip(tuple.iter()) {
                    changed |= self.unify_terms(&InferenceTerm::Canonical(canonical.ty), &inferred.term, store)?.is_changed();
                }
                Ok(if changed { SolveEffect::Changed } else { SolveEffect::Unchanged })
            }
            (InferenceTerm::ExactCase { enum_type, .. }, other) | (other, InferenceTerm::ExactCase { enum_type, .. }) => {
                self.unify_terms(enum_type, other, store)
            }
            (InferenceTerm::Callable(left_callable), InferenceTerm::Callable(right_callable)) => {
                if left_callable.parameters.len() != right_callable.parameters.len() {
                    return Err(InferenceFailureReason::StructuralMismatch {
                        left: Box::new(left.clone()),
                        right: Box::new(right.clone()),
                    });
                }
                let mut changed = false;
                for (left_parameter, right_parameter) in left_callable.parameters.iter().zip(right_callable.parameters.iter()) {
                    if left_parameter.label != right_parameter.label || left_parameter.rest != right_parameter.rest {
                        return Err(InferenceFailureReason::StructuralMismatch {
                            left: Box::new(left.clone()),
                            right: Box::new(right.clone()),
                        });
                    }
                    changed |= self.unify_terms(&left_parameter.term, &right_parameter.term, store)?.is_changed();
                }
                changed |= self.unify_terms(&left_callable.return_type, &right_callable.return_type, store)?.is_changed();
                Ok(if changed { SolveEffect::Changed } else { SolveEffect::Unchanged })
            }
            (InferenceTerm::Record(left_record), InferenceTerm::Record(right_record)) => {
                if left_record.tail != right_record.tail || left_record.fields.len() != right_record.fields.len() {
                    return Err(InferenceFailureReason::StructuralMismatch {
                        left: Box::new(left.clone()),
                        right: Box::new(right.clone()),
                    });
                }
                let mut changed = false;
                for (left_field, right_field) in left_record.fields.iter().zip(right_record.fields.iter()) {
                    if left_field.name != right_field.name {
                        return Err(InferenceFailureReason::StructuralMismatch {
                            left: Box::new(left.clone()),
                            right: Box::new(right.clone()),
                        });
                    }
                    changed |= self.unify_terms(&left_field.term, &right_field.term, store)?.is_changed();
                }
                Ok(if changed { SolveEffect::Changed } else { SolveEffect::Unchanged })
            }
            (InferenceTerm::Family(left_family), InferenceTerm::Family(right_family)) => {
                if left_family.len() != right_family.len() {
                    return Err(InferenceFailureReason::StructuralMismatch {
                        left: Box::new(left.clone()),
                        right: Box::new(right.clone()),
                    });
                }
                let mut changed = false;
                for (left_member, right_member) in left_family.iter().zip(right_family.iter()) {
                    if left_member.operation != right_member.operation || left_member.member_kind != right_member.member_kind {
                        return Err(InferenceFailureReason::StructuralMismatch {
                            left: Box::new(left.clone()),
                            right: Box::new(right.clone()),
                        });
                    }
                    changed |= self.unify_terms(&left_member.term, &right_member.term, store)?.is_changed();
                }
                Ok(if changed { SolveEffect::Changed } else { SolveEffect::Unchanged })
            }
            (
                InferenceTerm::AssociatedProjection { .. },
                InferenceTerm::AssociatedProjection { .. },
            ) => {
                if left == right {
                    return Ok(SolveEffect::Unchanged);
                }
                // Associated projections are non-injective by default. Do not
                // unify subjects or arguments backwards; preserve the required
                // equality as a residual proof obligation until normalization
                // can prove or refute it.
                self.defer_active_constraint();
                Ok(SolveEffect::Unchanged)
            }
            (
                InferenceTerm::Canonical(ty),
                other @ (InferenceTerm::Record(_)
                | InferenceTerm::Family(_)
                | InferenceTerm::AssociatedProjection { .. }
                | InferenceTerm::TypeMemberProjection { .. }
                | InferenceTerm::TypePack(_)
                | InferenceTerm::LabeledTypePack(_)
                | InferenceTerm::CompleteTypePack { .. }
                | InferenceTerm::TypePackConcat(_)
                | InferenceTerm::LabeledTypePackConcat(_)
                | InferenceTerm::TupleFromPacks { .. }
                | InferenceTerm::CallableFromPacks { .. }
                | InferenceTerm::PackProjection { .. }),
            )
            | (
                other @ (InferenceTerm::Record(_)
                | InferenceTerm::Family(_)
                | InferenceTerm::AssociatedProjection { .. }
                | InferenceTerm::TypeMemberProjection { .. }
                | InferenceTerm::TypePack(_)
                | InferenceTerm::LabeledTypePack(_)
                | InferenceTerm::CompleteTypePack { .. }
                | InferenceTerm::TypePackConcat(_)
                | InferenceTerm::LabeledTypePackConcat(_)
                | InferenceTerm::TupleFromPacks { .. }
                | InferenceTerm::CallableFromPacks { .. }
                | InferenceTerm::PackProjection { .. }),
                InferenceTerm::Canonical(ty),
            ) => {
                let canonical = self.type_id_to_inference(*ty, &HashMap::new(), store);
                if canonical == InferenceTerm::Canonical(*ty) {
                    if matches!(other, InferenceTerm::AssociatedProjection { .. } | InferenceTerm::TypeMemberProjection { .. } | InferenceTerm::PackProjection { .. }) {
                        self.defer_active_constraint();
                        return Ok(SolveEffect::Unchanged);
                    }
                    return Err(InferenceFailureReason::StructuralMismatch {
                        left: Box::new(left.clone()),
                        right: Box::new(right.clone()),
                    });
                }
                self.unify_terms(&canonical, other, store)
            }
            (InferenceTerm::TypePack(left_elems), InferenceTerm::TypePack(right_elems)) => {
                if left_elems.len() != right_elems.len() {
                    return Err(InferenceFailureReason::StructuralMismatch {
                        left: Box::new(left.clone()),
                        right: Box::new(right.clone()),
                    });
                }
                let mut changed = false;
                for (left_elem, right_elem) in left_elems.iter().zip(right_elems.iter()) {
                    changed |= self.unify_terms(left_elem, right_elem, store)?.is_changed();
                }
                Ok(if changed { SolveEffect::Changed } else { SolveEffect::Unchanged })
            }
            (InferenceTerm::LabeledTypePack(left_elems), InferenceTerm::LabeledTypePack(right_elems)) => {
                if left_elems.len() != right_elems.len() {
                    return Err(InferenceFailureReason::StructuralMismatch {
                        left: Box::new(left.clone()),
                        right: Box::new(right.clone()),
                    });
                }
                let mut changed = false;
                for (left_elem, right_elem) in left_elems.iter().zip(right_elems.iter()) {
                    if left_elem.label != right_elem.label {
                        return Err(InferenceFailureReason::StructuralMismatch {
                            left: Box::new(left.clone()),
                            right: Box::new(right.clone()),
                        });
                    }
                    changed |= self.unify_terms(&left_elem.term, &right_elem.term, store)?.is_changed();
                }
                Ok(if changed { SolveEffect::Changed } else { SolveEffect::Unchanged })
            }
            (
                InferenceTerm::CompleteTypePack {
                    positionals: left_pos,
                    labeled: left_lab,
                },
                InferenceTerm::CompleteTypePack {
                    positionals: right_pos,
                    labeled: right_lab,
                },
            ) => {
                let mut changed = self.unify_terms(left_pos, right_pos, store)?.is_changed();
                changed |= self.unify_terms(left_lab, right_lab, store)?.is_changed();
                Ok(if changed { SolveEffect::Changed } else { SolveEffect::Unchanged })
            }
            (InferenceTerm::TypePackConcat(left_segs), InferenceTerm::TypePackConcat(right_segs)) => {
                if left_segs.len() != right_segs.len() {
                    return Err(InferenceFailureReason::StructuralMismatch {
                        left: Box::new(left.clone()),
                        right: Box::new(right.clone()),
                    });
                }
                let mut changed = false;
                for (left_seg, right_seg) in left_segs.iter().zip(right_segs.iter()) {
                    changed |= self.unify_terms(left_seg, right_seg, store)?.is_changed();
                }
                Ok(if changed { SolveEffect::Changed } else { SolveEffect::Unchanged })
            }
            (InferenceTerm::LabeledTypePackConcat(left_segs), InferenceTerm::LabeledTypePackConcat(right_segs)) => {
                if left_segs.len() != right_segs.len() {
                    return Err(InferenceFailureReason::StructuralMismatch {
                        left: Box::new(left.clone()),
                        right: Box::new(right.clone()),
                    });
                }
                let mut changed = false;
                for (left_seg, right_seg) in left_segs.iter().zip(right_segs.iter()) {
                    changed |= self.unify_terms(left_seg, right_seg, store)?.is_changed();
                }
                Ok(if changed { SolveEffect::Changed } else { SolveEffect::Unchanged })
            }
            (
                InferenceTerm::TupleFromPacks {
                    positionals: left_pos,
                    labeled: left_lab,
                },
                InferenceTerm::TupleFromPacks {
                    positionals: right_pos,
                    labeled: right_lab,
                },
            ) => {
                let mut changed = self.unify_terms(left_pos, right_pos, store)?.is_changed();
                changed |= self.unify_terms(left_lab, right_lab, store)?.is_changed();
                Ok(if changed { SolveEffect::Changed } else { SolveEffect::Unchanged })
            }
            (InferenceTerm::TupleFromPacks { positionals, labeled }, InferenceTerm::Tuple(elements))
            | (InferenceTerm::Tuple(elements), InferenceTerm::TupleFromPacks { positionals, labeled }) => {
                let (tuple_pos, tuple_lab) = Self::inference_tuple_lanes(elements);
                let mut changed = self.unify_terms(positionals, &tuple_pos, store)?.is_changed();
                changed |= self.unify_terms(labeled, &tuple_lab, store)?.is_changed();
                Ok(if changed { SolveEffect::Changed } else { SolveEffect::Unchanged })
            }
            (
                InferenceTerm::CallableFromPacks {
                    positionals: left_pos,
                    labeled: left_lab,
                    return_type: left_ret,
                },
                InferenceTerm::CallableFromPacks {
                    positionals: right_pos,
                    labeled: right_lab,
                    return_type: right_ret,
                },
            ) => {
                let mut changed = self.unify_terms(left_pos, right_pos, store)?.is_changed();
                changed |= self.unify_terms(left_lab, right_lab, store)?.is_changed();
                changed |= self.unify_terms(left_ret, right_ret, store)?.is_changed();
                Ok(if changed { SolveEffect::Changed } else { SolveEffect::Unchanged })
            }
            (
                InferenceTerm::CallableFromPacks {
                    positionals,
                    labeled,
                    return_type,
                },
                InferenceTerm::Callable(callable),
            )
            | (
                InferenceTerm::Callable(callable),
                InferenceTerm::CallableFromPacks {
                    positionals,
                    labeled,
                    return_type,
                },
            ) => {
                let Some((call_pos, call_lab)) = Self::inference_callable_lanes(callable) else {
                    return Err(InferenceFailureReason::StructuralMismatch {
                        left: Box::new(left.clone()),
                        right: Box::new(right.clone()),
                    });
                };
                let mut changed = self.unify_terms(positionals, &call_pos, store)?.is_changed();
                changed |= self.unify_terms(labeled, &call_lab, store)?.is_changed();
                changed |= self.unify_terms(return_type, &callable.return_type, store)?.is_changed();
                Ok(if changed { SolveEffect::Changed } else { SolveEffect::Unchanged })
            }
            (
                InferenceTerm::PackProjection {
                    subject: left_subject,
                    lane: left_lane,
                },
                InferenceTerm::PackProjection {
                    subject: right_subject,
                    lane: right_lane,
                },
            ) => {
                if left_lane != right_lane {
                    return Err(InferenceFailureReason::StructuralMismatch {
                        left: Box::new(left.clone()),
                        right: Box::new(right.clone()),
                    });
                }
                if left_subject == right_subject {
                    return Ok(SolveEffect::Unchanged);
                }
                if *left_lane == InferencePackProjectionLane::Complete {
                    // Complete is the identity lane. Reduce it forward rather
                    // than treating generic projections as injective.
                    return self.unify_terms(left_subject, right_subject, store);
                }
                if let (Ok(left_ty), Ok(right_ty)) = (
                    self.materialize(left_subject, store),
                    self.materialize(right_subject, store),
                ) {
                    let left_projected = match left_lane {
                        InferencePackProjectionLane::Positionals => store.project_positionals(left_ty),
                        InferencePackProjectionLane::Labeled => store.project_labeled(left_ty),
                        InferencePackProjectionLane::Complete => store.project_complete(left_ty),
                    }
                    .map_err(|_| InferenceFailureReason::StructuralMismatch {
                        left: Box::new(left.clone()),
                        right: Box::new(right.clone()),
                    })?;
                    let right_projected = match right_lane {
                        InferencePackProjectionLane::Positionals => store.project_positionals(right_ty),
                        InferencePackProjectionLane::Labeled => store.project_labeled(right_ty),
                        InferencePackProjectionLane::Complete => store.project_complete(right_ty),
                    }
                    .map_err(|_| InferenceFailureReason::StructuralMismatch {
                        left: Box::new(left.clone()),
                        right: Box::new(right.clone()),
                    })?;
                    return self.unify_terms(
                        &InferenceTerm::Canonical(left_projected),
                        &InferenceTerm::Canonical(right_projected),
                        store,
                    );
                }
                // Positionals(P) == Positionals(Q) or Labeled(P) == Labeled(Q)
                // does NOT imply P == Q. Keep the equality mandatory and retry
                // after independent constraints solve the subjects.
                self.defer_active_constraint();
                Ok(SolveEffect::Unchanged)
            }
            (InferenceTerm::PackProjection { subject, lane }, other) | (other, InferenceTerm::PackProjection { subject, lane }) => {
                if *lane == InferencePackProjectionLane::Complete {
                    self.unify_terms(subject, other, store)
                } else if let Ok(subj_ty) = self.materialize(subject, store) {
                    let projected = match lane {
                        InferencePackProjectionLane::Positionals => store.project_positionals(subj_ty),
                        InferencePackProjectionLane::Labeled => store.project_labeled(subj_ty),
                        InferencePackProjectionLane::Complete => store.project_complete(subj_ty),
                    }
                    .map_err(InferenceMaterializationFailure::Pack)
                    .map_err(|_| InferenceFailureReason::StructuralMismatch {
                        left: Box::new(left.clone()),
                        right: Box::new(right.clone()),
                    })?;
                    self.unify_terms(&InferenceTerm::Canonical(projected), other, store)
                } else {
                    self.defer_active_constraint();
                    Ok(SolveEffect::Unchanged)
                }
            }
            // Type-member operations are not injective. Their subjects and
            // arguments may become canonical after other constraints solve,
            // but projection equality must not solve those children backwards.
            (InferenceTerm::TypeMemberProjection { .. }, InferenceTerm::TypeMemberProjection { .. }) if left == right => Ok(SolveEffect::Unchanged),
            (InferenceTerm::TypeMemberProjection { .. }, _) | (_, InferenceTerm::TypeMemberProjection { .. }) => {
                self.defer_active_constraint();
                Ok(SolveEffect::Unchanged)
            }
            _ => Err(InferenceFailureReason::StructuralMismatch {
                left: Box::new(left.clone()),
                right: Box::new(right.clone()),
            }),
        }
    }

    fn project_supertype_term(
        &self,
        origin: &InferenceTerm,
        arguments: &[InferenceTerm],
        store: &TypeStore,
        hier: &dyn TypeHierarchy,
    ) -> Option<InferenceTerm> {
        let InferenceTerm::Canonical(origin_ty) = origin else { return None };
        let TypeData::Nominal { declaration } = store.get(*origin_ty) else {
            return None;
        };
        let template = hier.supertype_template(declaration)?;
        let mut substitution = HashMap::new();
        for (index, argument) in arguments.iter().enumerate() {
            if let Some(parameter) = store.find_type_parameter_id(&crate::types::parameter::TypeParameterOwner::Declaration(declaration.clone()), index as u32)
            {
                substitution.insert(parameter, argument.clone());
            }
        }
        Some(self.type_id_to_inference(template.supertype, &substitution, store))
    }

    fn subtype_terms(
        &mut self,
        sub: &InferenceTerm,
        sup: &InferenceTerm,
        store: &mut TypeStore,
        hier: &dyn TypeHierarchy,
    ) -> Result<SolveEffect, InferenceFailureReason> {
        if let Some(reduced) = self.symbolic_beta_reduce(sub, store) {
            return self.subtype_terms(&reduced, sup, store, hier);
        }
        if let Some(reduced) = self.symbolic_beta_reduce(sup, store) {
            return self.subtype_terms(sub, &reduced, store, hier);
        }
        // Once both terms are canonical, use the single bounded relation
        // semantics. This keeps solver-local inference from inventing a second
        // subtype algebra for nominal, union, and structural types.
        if !self.term_has_unresolved_variables(sub)
            && !self.term_has_unresolved_variables(sup)
            && !self.term_has_projection(sub, store)
            && !self.term_has_projection(sup, store)
            && let (Ok(sub_ty), Ok(sup_ty)) = (self.materialize(sub, store), self.materialize(sup, store))
        {
            return if is_subtype(store, hier, sub_ty, sup_ty) {
                Ok(SolveEffect::Unchanged)
            } else {
                Err(InferenceFailureReason::StructuralMismatch {
                    left: Box::new(sub.clone()),
                    right: Box::new(sup.clone()),
                })
            };
        }

        match (sub, sup) {
            (InferenceTerm::ExactCase { enum_type, .. }, sup) => self.subtype_terms(enum_type, sup, store, hier),
            (InferenceTerm::Var(sub_var), InferenceTerm::Var(sup_var)) => {
                let sub_rep = self.find_var(*sub_var);
                let sup_rep = self.find_var(*sup_var);
                if sub_rep == sup_rep {
                    return Ok(SolveEffect::Unchanged);
                }
                if let Some(sub_ty) = self.substitutions.get(&sub_rep).copied() {
                    return self.subtype_terms(&InferenceTerm::Canonical(sub_ty), sup, store, hier);
                }
                if let Some(sup_ty) = self.substitutions.get(&sup_rep).copied() {
                    return self.subtype_terms(sub, &InferenceTerm::Canonical(sup_ty), store, hier);
                }
                if self
                    .subtype_edges
                    .iter()
                    .any(|edge| self.find_var(edge.sub) == sub_rep && self.find_var(edge.sup) == sup_rep)
                {
                    Ok(SolveEffect::Unchanged)
                } else {
                    self.subtype_edges.push(InferenceSubtypeEdge { sub: sub_rep, sup: sup_rep });
                    Ok(SolveEffect::Changed)
                }
            }
            (InferenceTerm::Var(v), term) => {
                let rep = self.find_var(*v);
                if let Some(ty) = self.substitutions.get(&rep).copied() {
                    self.subtype_terms(&InferenceTerm::Canonical(ty), term, store, hier)
                } else if let Ok(ty) = self.materialize(term, store) {
                    Ok(if self.add_upper_bound(rep, ty) {
                        SolveEffect::Changed
                    } else {
                        SolveEffect::Unchanged
                    })
                } else {
                    // A compound upper term with unresolved variables is a
                    // relation obligation, not an equality fallback. It will
                    // be revisited when its nested variables become concrete.
                    Ok(SolveEffect::Unchanged)
                }
            }
            (term, InferenceTerm::Var(v)) => {
                let rep = self.find_var(*v);
                if let Some(ty) = self.substitutions.get(&rep).copied() {
                    self.subtype_terms(term, &InferenceTerm::Canonical(ty), store, hier)
                } else if let Ok(ty) = self.materialize(term, store) {
                    Ok(if self.add_lower_bound(rep, ty) {
                        SolveEffect::Changed
                    } else {
                        SolveEffect::Unchanged
                    })
                } else {
                    Ok(SolveEffect::Unchanged)
                }
            }
            (InferenceTerm::Union(members), sup) => {
                let mut changed = false;
                for member in members.iter() {
                    changed |= self.subtype_terms(member, sup, store, hier)?.is_changed();
                }
                Ok(if changed { SolveEffect::Changed } else { SolveEffect::Unchanged })
            }
            (_sub, InferenceTerm::Union(_)) => {
                // A non-materializable right union has alternatives. Do not
                // choose an arm speculatively; canonical relation handles the
                // finite arm rule once the left term is materializable.
                Ok(SolveEffect::Unchanged)
            }
            (InferenceTerm::StaticData { .. }, InferenceTerm::StaticData { .. }) => self.unify_terms(sub, sup, store),
            (
                InferenceTerm::Applied {
                    origin: left_origin,
                    arguments: left_arguments,
                },
                InferenceTerm::Applied {
                    origin: right_origin,
                    arguments: right_arguments,
                },
            ) => {
                if let (Some(left_terms), Some(right_terms)) = (static_terms(left_arguments), static_terms(right_arguments)) {
                    if let Some(effect) = self.match_applied_constructor_shapes(left_origin, &left_terms, right_origin, &right_terms, store)? {
                        return Ok(effect);
                    }
                }
                if left_origin == right_origin {
                    if left_arguments.len() != right_arguments.len() {
                        return Err(InferenceFailureReason::StructuralMismatch {
                            left: Box::new(sub.clone()),
                            right: Box::new(sup.clone()),
                        });
                    }
                    let declaration = match left_origin.as_ref() {
                        InferenceTerm::Canonical(ty) => store.nominal_origin_declaration(*ty).cloned(),
                        _ => None,
                    };
                    let mut changed = false;
                    for (index, (left, right)) in left_arguments.iter().zip(right_arguments.iter()).enumerate() {
                        if matches!((left, right), (InferenceStaticArgument::RecordRow(_), InferenceStaticArgument::RecordRow(_))) {
                            changed |= self.unify_static_arguments(left, right, sub, sup, store)?.is_changed();
                            continue;
                        }
                        let variance = declaration
                            .as_ref()
                            .and_then(|decl| store.get_parameter_variance(decl, index as u32))
                            .unwrap_or(Variance::Invariant);
                        changed |= match variance {
                            Variance::Covariant => match (left, right) {
                                (InferenceStaticArgument::Term(left), InferenceStaticArgument::Term(right)) => self.subtype_terms(left, right, store, hier)?.is_changed(),
                                _ => self.unify_static_arguments(left, right, sub, sup, store)?.is_changed(),
                            },
                            Variance::Contravariant => match (left, right) {
                                (InferenceStaticArgument::Term(left), InferenceStaticArgument::Term(right)) => self.subtype_terms(right, left, store, hier)?.is_changed(),
                                _ => self.unify_static_arguments(left, right, sub, sup, store)?.is_changed(),
                            },
                            Variance::Invariant => self.unify_static_arguments(left, right, sub, sup, store)?.is_changed(),
                        };
                    }
                    Ok(if changed { SolveEffect::Changed } else { SolveEffect::Unchanged })
                } else if let Some(left_terms) = static_terms(left_arguments)
                    && let Some(projected) = self.project_supertype_term(left_origin, &left_terms, store, hier)
                {
                    if projected == *sub {
                        return Ok(SolveEffect::Unchanged);
                    }
                    self.subtype_terms(&projected, sup, store, hier)
                } else {
                    Ok(SolveEffect::Unchanged)
                }
            }
            (InferenceTerm::Canonical(ty), other) => {
                let expanded = self.type_id_to_inference(*ty, &HashMap::new(), store);
                // Caller-owned parameters are rigid. If canonical expansion
                // leaves this type atomic, this relation is pending until
                // the other side becomes materializable; re-entering with
                // the same terms would never make structural progress.
                if expanded == InferenceTerm::Canonical(*ty) {
                    return Ok(SolveEffect::Unchanged);
                }
                if expanded == *other {
                    return Ok(SolveEffect::Unchanged);
                }
                self.subtype_terms(&expanded, other, store, hier)
            }
            (other, InferenceTerm::Canonical(ty)) => {
                let expanded = self.type_id_to_inference(*ty, &HashMap::new(), store);
                // Keep subtype direction intact when expanding the right
                // canonical term. Rigid canonical types cannot be solved by
                // the opposite compound term.
                if expanded == InferenceTerm::Canonical(*ty) {
                    return Ok(SolveEffect::Unchanged);
                }
                if expanded == *other {
                    return Ok(SolveEffect::Unchanged);
                }
                self.subtype_terms(other, &expanded, store, hier)
            }
            (InferenceTerm::Callable(left), InferenceTerm::Callable(right)) => {
                if left.parameters.len() != right.parameters.len() {
                    return Err(InferenceFailureReason::StructuralMismatch {
                        left: Box::new(sub.clone()),
                        right: Box::new(sup.clone()),
                    });
                }
                let mut changed = false;
                for (left_parameter, right_parameter) in left.parameters.iter().zip(right.parameters.iter()) {
                    if left_parameter.label != right_parameter.label || left_parameter.rest != right_parameter.rest {
                        return Err(InferenceFailureReason::StructuralMismatch {
                            left: Box::new(sub.clone()),
                            right: Box::new(sup.clone()),
                        });
                    }
                    changed |= self.subtype_terms(&right_parameter.term, &left_parameter.term, store, hier)?.is_changed();
                }
                changed |= self.subtype_terms(&left.return_type, &right.return_type, store, hier)?.is_changed();
                Ok(if changed { SolveEffect::Changed } else { SolveEffect::Unchanged })
            }
            (InferenceTerm::Tuple(left), InferenceTerm::Tuple(right)) => {
                if left.len() != right.len() {
                    return Err(InferenceFailureReason::StructuralMismatch {
                        left: Box::new(sub.clone()),
                        right: Box::new(sup.clone()),
                    });
                }
                let mut changed = false;
                for (left, right) in left.iter().zip(right.iter()) {
                    if left.label != right.label {
                        return Err(InferenceFailureReason::StructuralMismatch {
                            left: Box::new(sub.clone()),
                            right: Box::new(sup.clone()),
                        });
                    }
                    changed |= self.subtype_terms(&left.term, &right.term, store, hier)?.is_changed();
                }
                Ok(if changed { SolveEffect::Changed } else { SolveEffect::Unchanged })
            }
            (InferenceTerm::Record(left), InferenceTerm::Record(right)) => {
                let mut changed = false;
                for right_field in right.fields.iter() {
                    let Some(left_field) = left.fields.iter().find(|field| field.name == right_field.name) else {
                        return Err(InferenceFailureReason::StructuralMismatch {
                            left: Box::new(sub.clone()),
                            right: Box::new(sup.clone()),
                        });
                    };
                    changed |= self.subtype_terms(&left_field.term, &right_field.term, store, hier)?.is_changed();
                }
                Ok(if changed { SolveEffect::Changed } else { SolveEffect::Unchanged })
            }
            (InferenceTerm::Family(left), InferenceTerm::Family(right)) => {
                let mut changed = false;
                for right_member in right.iter() {
                    let Some(left_member) = left.iter().find(|member| member.operation == right_member.operation) else {
                        return Err(InferenceFailureReason::StructuralMismatch {
                            left: Box::new(sub.clone()),
                            right: Box::new(sup.clone()),
                        });
                    };
                    if left_member.member_kind != right_member.member_kind {
                        return Err(InferenceFailureReason::StructuralMismatch {
                            left: Box::new(sub.clone()),
                            right: Box::new(sup.clone()),
                        });
                    }
                    changed |= self.subtype_terms(&left_member.term, &right_member.term, store, hier)?.is_changed();
                }
                Ok(if changed { SolveEffect::Changed } else { SolveEffect::Unchanged })
            }
            (
                InferenceTerm::AssociatedProjection {
                    subject: left_subject,
                    trait_ref: left_trait_ref,
                    requirement: left_requirement,
                },
                InferenceTerm::AssociatedProjection {
                    subject: right_subject,
                    trait_ref: right_trait_ref,
                    requirement: right_requirement,
                },
            ) => {
                if left_requirement != right_requirement
                    || left_trait_ref.declaration != right_trait_ref.declaration
                    || left_trait_ref.arguments.len() != right_trait_ref.arguments.len()
                {
                    return Err(InferenceFailureReason::StructuralMismatch {
                        left: Box::new(sub.clone()),
                        right: Box::new(sup.clone()),
                    });
                }
                let mut changed = self.unify_terms(left_subject, right_subject, store)?.is_changed();
                for (left_arg, right_arg) in left_trait_ref.arguments.iter().zip(right_trait_ref.arguments.iter()) {
                    changed |= self.unify_terms(left_arg, right_arg, store)?.is_changed();
                }
                Ok(if changed { SolveEffect::Changed } else { SolveEffect::Unchanged })
            }
            (
                InferenceTerm::TupleFromPacks {
                    positionals: sub_pos,
                    labeled: sub_lab,
                },
                InferenceTerm::Tuple(elements),
            ) => {
                let (tuple_pos, tuple_lab) = Self::inference_tuple_lanes(elements);
                let mut changed = self.unify_terms(sub_pos, &tuple_pos, store)?.is_changed();
                changed |= self.unify_terms(sub_lab, &tuple_lab, store)?.is_changed();
                Ok(if changed { SolveEffect::Changed } else { SolveEffect::Unchanged })
            }
            (
                InferenceTerm::Tuple(sub_elements),
                InferenceTerm::TupleFromPacks {
                    positionals: sup_pos,
                    labeled: sup_lab,
                },
            ) => {
                let (tuple_pos, tuple_lab) = Self::inference_tuple_lanes(sub_elements);
                let mut changed = self.unify_terms(&tuple_pos, sup_pos, store)?.is_changed();
                changed |= self.unify_terms(&tuple_lab, sup_lab, store)?.is_changed();
                Ok(if changed { SolveEffect::Changed } else { SolveEffect::Unchanged })
            }
            (
                InferenceTerm::CallableFromPacks {
                    positionals: sub_pos,
                    labeled: sub_lab,
                    return_type: sub_ret,
                },
                InferenceTerm::Callable(sup_callable),
            ) => {
                let Some((call_pos, call_lab)) = Self::inference_callable_lanes(sup_callable) else {
                    return Err(InferenceFailureReason::StructuralMismatch {
                        left: Box::new(sub.clone()),
                        right: Box::new(sup.clone()),
                    });
                };
                let mut changed = self.unify_terms(sub_pos, &call_pos, store)?.is_changed();
                changed |= self.unify_terms(sub_lab, &call_lab, store)?.is_changed();
                changed |= self.subtype_terms(sub_ret, &sup_callable.return_type, store, hier)?.is_changed();
                Ok(if changed { SolveEffect::Changed } else { SolveEffect::Unchanged })
            }
            (
                InferenceTerm::Callable(sub_callable),
                InferenceTerm::CallableFromPacks {
                    positionals: sup_pos,
                    labeled: sup_lab,
                    return_type: sup_ret,
                },
            ) => {
                let Some((call_pos, call_lab)) = Self::inference_callable_lanes(sub_callable) else {
                    return Err(InferenceFailureReason::StructuralMismatch {
                        left: Box::new(sub.clone()),
                        right: Box::new(sup.clone()),
                    });
                };
                let mut changed = self.unify_terms(&call_pos, sup_pos, store)?.is_changed();
                changed |= self.unify_terms(&call_lab, sup_lab, store)?.is_changed();
                changed |= self.subtype_terms(&sub_callable.return_type, sup_ret, store, hier)?.is_changed();
                Ok(if changed { SolveEffect::Changed } else { SolveEffect::Unchanged })
            }
            (
                InferenceTerm::TupleFromPacks {
                    positionals: sub_pos,
                    labeled: sub_lab,
                },
                InferenceTerm::TupleFromPacks {
                    positionals: sup_pos,
                    labeled: sup_lab,
                },
            ) => {
                let mut changed = self.unify_terms(sub_pos, sup_pos, store)?.is_changed();
                changed |= self.unify_terms(sub_lab, sup_lab, store)?.is_changed();
                Ok(if changed { SolveEffect::Changed } else { SolveEffect::Unchanged })
            }
            (
                InferenceTerm::CallableFromPacks {
                    positionals: sub_pos,
                    labeled: sub_lab,
                    return_type: sub_ret,
                },
                InferenceTerm::CallableFromPacks {
                    positionals: sup_pos,
                    labeled: sup_lab,
                    return_type: sup_ret,
                },
            ) => {
                let mut changed = self.unify_terms(sub_pos, sup_pos, store)?.is_changed();
                changed |= self.unify_terms(sub_lab, sup_lab, store)?.is_changed();
                changed |= self.subtype_terms(sub_ret, sup_ret, store, hier)?.is_changed();
                Ok(if changed { SolveEffect::Changed } else { SolveEffect::Unchanged })
            }
            _ => {
                if self.term_has_projection(sub, store) || self.term_has_projection(sup, store) {
                    Ok(SolveEffect::Unchanged)
                } else {
                    Err(InferenceFailureReason::StructuralMismatch {
                        left: Box::new(sub.clone()),
                        right: Box::new(sup.clone()),
                    })
                }
            }
        }
    }

    /// Materializes an `InferenceTerm` into a canonical `TypeId`, preserving
    /// whether failure came from an unsolved variable or invalid application.
    pub fn materialize(&self, term: &InferenceTerm, store: &mut TypeStore) -> Result<TypeId, InferenceMaterializationFailure> {
        match term {
            InferenceTerm::Canonical(ty) => Ok(*ty),
            InferenceTerm::Rigid(rigid) => Err(InferenceMaterializationFailure::Rigid(*rigid)),
            InferenceTerm::Var(v) => {
                let rep = self.find_var(*v);
                if let Some(&ty) = self.substitutions.get(&rep) {
                    Ok(ty)
                } else {
                    Err(InferenceMaterializationFailure::Unsolved(UnderconstrainedInference { unsolved_vars: vec![*v] }))
                }
            }
            InferenceTerm::ExactCase { variant, enum_type } => {
                let enum_ty = self.materialize(enum_type, store)?;
                let variant_id = store.variant_identity(*variant).clone();
                store
                    .exact_case_type(&variant_id, enum_ty)
                    .map_err(|_| InferenceMaterializationFailure::InvalidExactCase)
            }
            InferenceTerm::Applied { origin, arguments } => {
                let orig_ty = self.materialize(origin, store)?;
                let mut static_arguments = Vec::with_capacity(arguments.len());
                for argument in arguments.iter() {
                    static_arguments.push(self.materialize_static_argument(argument, store)?);
                }
                store
                    .apply_static_form(orig_ty, &static_arguments)
                    .map_err(InferenceMaterializationFailure::TypeApplication)
            }
            InferenceTerm::StaticData {
                domain,
                constructor,
                arguments,
            } => {
                let arguments = arguments
                    .iter()
                    .map(|argument| self.materialize_static_argument(argument, store))
                    .collect::<Result<Vec<_>, _>>()?;
                store
                    .static_data(*domain, *constructor, arguments.into_boxed_slice())
                    .map_err(|_| InferenceMaterializationFailure::InternalInvariant)
            }
            InferenceTerm::Union(members) => {
                let mut member_tys = Vec::with_capacity(members.len());
                for m in members.iter() {
                    member_tys.push(self.materialize(m, store)?);
                }
                Ok(store.union(&member_tys))
            }
            InferenceTerm::Tuple(elems) => {
                let mut tuple_elems = Vec::with_capacity(elems.len());
                for e in elems.iter() {
                    tuple_elems.push(TupleTypeElement {
                        label: e.label.clone(),
                        ty: self.materialize(&e.term, store)?,
                    });
                }
                Ok(store.tuple(tuple_elems.into_boxed_slice()))
            }
            InferenceTerm::Callable(c) => {
                let mut params = Vec::with_capacity(c.parameters.len());
                for p in c.parameters.iter() {
                    params.push(CallableParameterType {
                        label: p.label.clone(),
                        ty: self.materialize(&p.term, store)?,
                        rest: p.rest,
                    });
                }
                let ret = self.materialize(&c.return_type, store)?;
                Ok(store.callable(CallableType {
                    parameters: params.into_boxed_slice(),
                    return_type: ret,
                }))
            }
            InferenceTerm::Record(record) => {
                let normalized = self.normalized_record_row(record);
                let fields = normalized
                    .fields
                    .iter()
                    .map(|field| {
                        Ok(crate::types::row::RecordRowField {
                            name: field.name.clone(),
                            ty: self.materialize(&field.ty, store)?,
                        })
                    })
                    .collect::<Result<Vec<_>, InferenceMaterializationFailure>>()?;
                store
                    .record_row_type_checked(
                        fields,
                        match normalized.tail {
                            RecordRowTermTail::Closed => RecordRowTail::Closed,
                            RecordRowTermTail::Parameter(parameter) => RecordRowTail::Parameter(parameter),
                            RecordRowTermTail::Projection(proj) => RecordRowTail::Projection(proj),
                            RecordRowTermTail::Var(_) => return Err(InferenceMaterializationFailure::UnsupportedDomain),
                        },
                    )
                    .map_err(|_| InferenceMaterializationFailure::InternalInvariant)
            }
            InferenceTerm::Family(family) => {
                let members = family
                    .iter()
                    .map(|member| {
                        Ok(FamilyMemberType {
                            operation: member.operation.clone(),
                            member_kind: member.member_kind,
                            ty: self.materialize(&member.term, store)?,
                        })
                    })
                    .collect::<Result<Vec<_>, InferenceMaterializationFailure>>()?;
                store.family_type(members).map_err(|_| InferenceMaterializationFailure::InternalInvariant)
            }
            InferenceTerm::AssociatedProjection {
                subject,
                trait_ref,
                requirement,
            } => {
                let subject_ty = self.materialize(subject, store)?;
                let mut arg_tys = Vec::with_capacity(trait_ref.arguments.len());
                for arg in trait_ref.arguments.iter() {
                    arg_tys.push(self.materialize(arg, store)?);
                }
                let trait_ref = crate::traits::TraitRef::new(trait_ref.declaration.clone(), arg_tys);
                Ok(store.associated_projection(subject_ty, trait_ref, requirement.clone()))
            }
            InferenceTerm::TypeMemberProjection {
                subject,
                selector,
                arguments,
                source_declaration,
                result_kind,
            } => {
                let projection = self.materialize_type_member_projection(subject, selector, arguments, source_declaration.as_ref(), store)?;
                Ok(store.type_member_projection(projection, *result_kind))
            }
            InferenceTerm::TypePack(elements) => {
                let mut elem_tys = Vec::with_capacity(elements.len());
                for e in elements.iter() {
                    elem_tys.push(self.materialize(e, store)?);
                }
                store.type_pack(elem_tys.into_boxed_slice()).map_err(InferenceMaterializationFailure::Pack)
            }
            InferenceTerm::LabeledTypePack(elements) => {
                let mut elem_tys = Vec::with_capacity(elements.len());
                for e in elements.iter() {
                    elem_tys.push(crate::types::store::LabeledTypePackElement {
                        label: e.label.clone(),
                        element: self.materialize(&e.term, store)?,
                    });
                }
                store
                    .labeled_type_pack(elem_tys.into_boxed_slice())
                    .map_err(InferenceMaterializationFailure::Pack)
            }
            InferenceTerm::CompleteTypePack { positionals, labeled } => {
                let pos = self.materialize(positionals, store)?;
                let lab = self.materialize(labeled, store)?;
                store.complete_type_pack(pos, lab).map_err(InferenceMaterializationFailure::Pack)
            }
            InferenceTerm::TypePackConcat(segments) => {
                let mut seg_tys = Vec::with_capacity(segments.len());
                for s in segments.iter() {
                    seg_tys.push(self.materialize(s, store)?);
                }
                store.type_pack_concat(&seg_tys).map_err(InferenceMaterializationFailure::Pack)
            }
            InferenceTerm::LabeledTypePackConcat(segments) => {
                let mut seg_tys = Vec::with_capacity(segments.len());
                for s in segments.iter() {
                    seg_tys.push(self.materialize(s, store)?);
                }
                store.labeled_type_pack_concat(&seg_tys).map_err(InferenceMaterializationFailure::Pack)
            }
            InferenceTerm::TupleFromPacks { positionals, labeled } => {
                let pos = self.materialize(positionals, store)?;
                let lab = self.materialize(labeled, store)?;
                store.tuple_from_packs(pos, lab).map_err(InferenceMaterializationFailure::Pack)
            }
            InferenceTerm::CallableFromPacks {
                positionals,
                labeled,
                return_type,
            } => {
                let pos = self.materialize(positionals, store)?;
                let lab = self.materialize(labeled, store)?;
                let ret = self.materialize(return_type, store)?;
                store.callable_from_packs(pos, lab, ret).map_err(InferenceMaterializationFailure::Pack)
            }
            InferenceTerm::PackProjection { subject, lane } => {
                let subj = self.materialize(subject, store)?;
                match lane {
                    InferencePackProjectionLane::Positionals => store.project_positionals(subj),
                    InferencePackProjectionLane::Labeled => store.project_labeled(subj),
                    InferencePackProjectionLane::Complete => store.project_complete(subj),
                }
                .map_err(InferenceMaterializationFailure::Pack)
            }
        }
    }

    fn materialize_static_argument(
        &self,
        argument: &InferenceStaticArgument,
        store: &mut TypeStore,
    ) -> Result<crate::types::store::StaticArgument, InferenceMaterializationFailure> {
        match argument {
            InferenceStaticArgument::Term(term) => self.materialize(term, store).map(crate::types::store::StaticArgument::Term),
            InferenceStaticArgument::RecordRow(row) => {
                let normalized = self.normalized_record_row(row);
                let fields = normalized
                    .fields
                    .iter()
                    .map(|field| {
                        Ok(crate::types::row::RecordRowField {
                            name: field.name.clone(),
                            ty: self.materialize(&field.ty, store)?,
                        })
                    })
                    .collect::<Result<Vec<_>, InferenceMaterializationFailure>>()?;
                let tail = match normalized.tail {
                    RecordRowTermTail::Closed => RecordRowTail::Closed,
                    RecordRowTermTail::Parameter(parameter) => RecordRowTail::Parameter(parameter),
                    RecordRowTermTail::Projection(projection) => RecordRowTail::Projection(projection),
                    RecordRowTermTail::Var(_) => return Err(InferenceMaterializationFailure::UnsupportedDomain),
                };
                store
                    .record_row_checked(fields, tail)
                    .map(crate::types::store::StaticArgument::RecordRow)
                    .map_err(|_| InferenceMaterializationFailure::UnsupportedDomain)
            }
        }
    }

    fn materialize_type_member_projection(
        &self,
        subject: &InferenceProjectionSubject,
        selector: &TypeMemberSelector,
        arguments: &[InferenceTypeMemberArgument],
        source_declaration: Option<&TypeMemberDeclarationId>,
        store: &mut TypeStore,
    ) -> Result<TypeLevelProjection, InferenceMaterializationFailure> {
        let subject = match subject {
            InferenceProjectionSubject::Type(term) => ProjectionSubject::Type(self.materialize(term, store)?),
            InferenceProjectionSubject::RecordRow(row) => match self
                .materialize_static_argument(&InferenceStaticArgument::RecordRow(row.clone()), store)?
            {
                crate::types::store::StaticArgument::RecordRow(row) => ProjectionSubject::RecordRow(row),
                crate::types::store::StaticArgument::Term(_) => return Err(InferenceMaterializationFailure::InternalInvariant),
            },
        };
        let arguments = arguments
            .iter()
            .map(|argument| match argument {
                InferenceTypeMemberArgument::Type(term) => self.materialize(term, store).map(TypeMemberArgument::Type),
                InferenceTypeMemberArgument::Const(value) => Ok(TypeMemberArgument::Const(value.clone())),
                InferenceTypeMemberArgument::RecordRow(row) => match self
                    .materialize_static_argument(&InferenceStaticArgument::RecordRow(row.clone()), store)?
                {
                    crate::types::store::StaticArgument::RecordRow(row) => Ok(TypeMemberArgument::RecordRow(row)),
                    crate::types::store::StaticArgument::Term(_) => Err(InferenceMaterializationFailure::InternalInvariant),
                },
            })
            .collect::<Result<Vec<_>, _>>()?
            .into_boxed_slice();
        Ok(TypeLevelProjection {
            subject,
            selector: selector.clone(),
            arguments,
            source_declaration: source_declaration.cloned(),
        })
    }

    /// Extracts materialized projections from constraints and var_terms, normalizes them via
    /// the provided normalizer closure, and unifies the normalized types.
    pub fn extract_and_normalize_projections<F>(&mut self, store: &mut TypeStore, mut normalizer: F) -> Result<bool, InferenceFailureReason>
    where
        F: FnMut(&mut TypeStore, TypeId) -> Option<TypeId>,
    {
        let mut progress = false;

        // Check var_terms for materialized projections
        let var_terms = self.var_terms.clone();
        for (var, term) in var_terms {
            let rep = self.find_var(var);
            if !self.substitutions.contains_key(&rep) {
                if let Ok(ty) = self.materialize(&term, store) {
                    if let Some(norm_ty) = normalizer(store, ty) {
                        if norm_ty != ty || !self.term_has_unresolved_variables(&term) {
                            let effect = self.bind(rep, norm_ty, store)?;
                            progress |= effect.is_changed();
                        }
                    }
                }
            }
        }

        // Check constraints for projections where one side is materialized & normalizable
        let mut constraints = self.constraints.clone();
        for constraint in &mut constraints {
            match &mut constraint.relation {
                InferenceRelation::Equivalent(left, right) => {
                    let mut new_left = None;
                    let mut new_right = None;
                    if let Ok(left_ty) = self.materialize(left, store) {
                        if let Some(norm_ty) = normalizer(store, left_ty) {
                            if norm_ty != left_ty {
                                new_left = Some(InferenceTerm::Canonical(norm_ty));
                            }
                        }
                    }
                    if let Ok(right_ty) = self.materialize(right, store) {
                        if let Some(norm_ty) = normalizer(store, right_ty) {
                            if norm_ty != right_ty {
                                new_right = Some(InferenceTerm::Canonical(norm_ty));
                            }
                        }
                    }
                    if let Some(nl) = new_left {
                        let effect = self.unify_terms(&nl, right, store)?;
                        progress |= effect.is_changed();
                        *left = nl;
                    }
                    if let Some(nr) = new_right {
                        let effect = self.unify_terms(left, &nr, store)?;
                        progress |= effect.is_changed();
                        *right = nr;
                    }
                }
                InferenceRelation::Subtype(sub, sup) => {
                    let mut new_sub = None;
                    let mut new_sup = None;
                    if let Ok(sub_ty) = self.materialize(sub, store) {
                        if let Some(norm_ty) = normalizer(store, sub_ty) {
                            if norm_ty != sub_ty {
                                new_sub = Some(InferenceTerm::Canonical(norm_ty));
                            }
                        }
                    }
                    if let Ok(sup_ty) = self.materialize(sup, store) {
                        if let Some(norm_ty) = normalizer(store, sup_ty) {
                            if norm_ty != sup_ty {
                                new_sup = Some(InferenceTerm::Canonical(norm_ty));
                            }
                        }
                    }
                    if let Some(ns) = new_sub {
                        *sub = ns;
                        progress = true;
                    }
                    if let Some(ns) = new_sup {
                        *sup = ns;
                        progress = true;
                    }
                }
            }
        }
        self.constraints = constraints;

        Ok(progress)
    }
}

/// Query-context owner for the ordinary type solver and the distinct record
/// row solver. It coordinates their fixed point without merging identities.
pub(crate) struct InferenceCoordinator {
    types: InferenceSession,
    rows: Rc<RefCell<RecordRowSolver<InferenceTerm>>>,
    row_variable_frames: HashMap<RecordRowVarId, InferenceFrameId>,
    row_parameter_bindings: HashMap<InferenceFrameId, HashMap<TypeParameterId, RecordRowVarId>>,
}

impl InferenceCoordinator {
    pub(crate) fn new() -> Self {
        let rows = Rc::new(RefCell::new(RecordRowSolver::new()));
        let types = InferenceSession {
            row_solver: Rc::downgrade(&rows),
            ..InferenceSession::default()
        };
        Self {
            types,
            rows,
            row_variable_frames: HashMap::new(),
            row_parameter_bindings: HashMap::new(),
        }
    }

    pub(crate) fn fresh_record_row_variable_in_frame(&mut self, frame: InferenceFrameId) -> RecordRowVarId {
        let variable = self.rows.borrow_mut().fresh_var();
        self.row_variable_frames.insert(variable, frame);
        variable
    }

    pub(crate) fn bind_record_row_parameter(&mut self, frame: InferenceFrameId, parameter: TypeParameterId, variable: RecordRowVarId) {
        self.row_parameter_bindings.entry(frame).or_default().insert(parameter, variable);
    }

    pub(crate) fn row_parameter_terms_for_frame(&self, frame: InferenceFrameId) -> HashMap<TypeParameterId, RecordRowVarId> {
        let mut chain = Vec::new();
        let mut current = Some(frame);
        while let Some(candidate) = current {
            chain.push(candidate);
            current = self.types.frame_parent(candidate);
        }
        let mut bindings = HashMap::new();
        for ancestor in chain.into_iter().rev() {
            bindings.extend(self.row_parameter_bindings.get(&ancestor).into_iter().flatten().map(|(&parameter, &variable)| (parameter, variable)));
        }
        bindings
    }

    pub(crate) fn add_record_row_lacks(
        &mut self,
        variable: RecordRowVarId,
        field: Box<str>,
    ) -> Result<(), crate::types::row_solver::RecordRowFailure<InferenceTerm>> {
        self.rows.borrow_mut().add_lacks(variable, field)
    }

    pub(crate) fn row_solution_for_roots(
        &self,
        frame: InferenceFrameId,
        roots: &[RecordRowVarId],
        budget: &mut QueryBudget,
        cancellation: &CancellationToken,
    ) -> RecordRowSolveResult<InferenceTerm> {
        let mut solver = self.rows.borrow().clone();
        match solver.solve_shapes_for_roots(&[], &[], budget, cancellation) {
            RecordRowShapeSolveResult::Solved { solution, .. } => {
                let mut unresolved = Vec::new();
                for root in roots {
                    let normalized = solver.normalize_term(&RecordRowTerm {
                        fields: Box::new([]),
                        tail: RecordRowTermTail::Var(*root),
                    });
                    if let RecordRowTermTail::Var(tail) = normalized.tail
                        && !self.row_variable_is_owned_by_ancestor_frame(tail, frame)
                    {
                        unresolved.push(*root);
                    }
                }
                unresolved.sort_unstable();
                unresolved.dedup();
                if unresolved.is_empty() {
                    RecordRowSolveResult::Solved(solution)
                } else {
                    RecordRowSolveResult::Underconstrained(crate::types::row_solver::RecordRowUnderconstrained {
                        variables: unresolved.into_boxed_slice(),
                    })
                }
            }
            RecordRowShapeSolveResult::Underconstrained(result) => RecordRowSolveResult::Underconstrained(result),
            RecordRowShapeSolveResult::Rejected(error) => RecordRowSolveResult::Rejected(error),
            RecordRowShapeSolveResult::Blocked(reason) => RecordRowSolveResult::Blocked(reason),
            RecordRowShapeSolveResult::Cancelled => RecordRowSolveResult::Cancelled,
            RecordRowShapeSolveResult::BudgetExceeded(report) => RecordRowSolveResult::BudgetExceeded(report),
            RecordRowShapeSolveResult::InternalFailure(id) => RecordRowSolveResult::InternalFailure(id),
        }
    }

    pub(crate) fn row_variable_is_owned_by_ancestor_frame(&self, variable: RecordRowVarId, frame: InferenceFrameId) -> bool {
        let Some(owner) = self.row_variable_frames.get(&variable).copied() else {
            return false;
        };
        let mut current = self.types.frame_parent(frame);
        while let Some(candidate) = current {
            if candidate == owner {
                return true;
            }
            current = self.types.frame_parent(candidate);
        }
        false
    }

    pub(crate) fn term_has_unresolved_ancestor_row(
        &self,
        term: &InferenceTerm,
        solution: &RecordRowSolution<InferenceTerm>,
        frame: InferenceFrameId,
    ) -> bool {
        self.row_variable_frames.keys().copied().any(|root| {
            if !self.types.term_contains_row_variable(term, root) {
                return false;
            }
            let mut current = root;
            let mut seen = HashSet::new();
            loop {
                if !seen.insert(current) {
                    return false;
                }
                match solution.term_for(current) {
                    Some(row) => match row.tail {
                        RecordRowTermTail::Var(next) => current = next,
                        RecordRowTermTail::Closed | RecordRowTermTail::Parameter(_) | RecordRowTermTail::Projection(_) => return false,
                    },
                    None => return self.row_variable_is_owned_by_ancestor_frame(current, frame),
                }
            }
        })
    }

    pub(crate) fn enqueue_record_row_equation(&mut self, left: &InferenceRecord, right: &InferenceRecord) -> SolveEffect {
        self.types.enqueue_record_row_equation(left, right)
    }

    pub(crate) fn propagate_with_control(
        &mut self,
        store: &mut TypeStore,
        hierarchy: &dyn TypeHierarchy,
        control: &CheckerControl,
    ) -> Result<bool, Box<InferenceOutcome>> {
        let mut any_changed = false;
        loop {
            let ordinary_changed = self.types.propagate_with_control(store, hierarchy, control)?;
            any_changed |= ordinary_changed;
            let pending_rows = self.types.take_pending_row_equations();
            if pending_rows.is_empty() {
                return Ok(any_changed);
            }

            let equations = pending_rows
                .iter()
                .map(|equation| (record_term_for_row_solver(&equation.left), record_term_for_row_solver(&equation.right)))
                .collect::<Vec<_>>();
            let row_state = self.rows.borrow().clone();
            let var_terms = &self.types.var_terms;
            let aliases = &self.types.var_aliases;
            let rows = self.rows.clone();
            let row_result = control.relation(|budget, cancellation| {
                rows.borrow_mut()
                    .solve_shapes_for_roots_with_payload_occurs(&equations, &[], budget, cancellation, |target, payload| {
                        inference_term_contains_row_variable(
                            payload,
                            target,
                            &row_state,
                            var_terms,
                            aliases,
                            &mut HashSet::new(),
                        )
                    })
            });
            match row_result {
                RecordRowShapeSolveResult::Solved { field_equations, .. } => {
                    for (_, left, right) in field_equations {
                        self.types.add_constraint(InferenceRelation::Equivalent(left, right), ConstraintOrigin::Explicit, None);
                    }
                    any_changed = true;
                }
                RecordRowShapeSolveResult::Underconstrained(_) => {
                    unreachable!("row shape solving without completion roots cannot be underconstrained")
                }
                RecordRowShapeSolveResult::Rejected(failure) => {
                    return Err(Box::new(self.types.failure_outcome(
                        InferenceFailureReason::RowRejected(failure),
                        None,
                        None,
                    )));
                }
                RecordRowShapeSolveResult::Blocked(reason) => return Err(Box::new(InferenceOutcome::Blocked(match reason {
                    crate::types::row_solver::RecordRowBlockedReason::UnboundVariable(_) => BlockReason::RecursiveFixpoint,
                    crate::types::row_solver::RecordRowBlockedReason::AmbiguousExtension => BlockReason::RecursiveFixpoint,
                }))),
                RecordRowShapeSolveResult::Cancelled => return Err(Box::new(InferenceOutcome::Cancelled)),
                RecordRowShapeSolveResult::BudgetExceeded(report) => return Err(Box::new(InferenceOutcome::BudgetExceeded(report))),
                RecordRowShapeSolveResult::InternalFailure(id) => {
                    return Err(Box::new(InferenceOutcome::InternalFailure(InferenceFailureReason::InternalRowSolverFailure {
                        incident: id.0,
                    })));
                }
            }
        }
    }

    pub(crate) fn solve_with_control(
        &mut self,
        store: &mut TypeStore,
        hierarchy: &dyn TypeHierarchy,
        control: &CheckerControl,
    ) -> InferenceOutcome {
        match self.propagate_with_control(store, hierarchy, control) {
            Ok(_) => self.types.finish_root(),
            Err(outcome) => *outcome,
        }
    }
}

impl Deref for InferenceCoordinator {
    type Target = InferenceSession;

    fn deref(&self) -> &Self::Target {
        &self.types
    }
}

impl DerefMut for InferenceCoordinator {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.types
    }
}

#[cfg(test)]
mod tests {
    use super::{
        ConstraintOrigin, InferenceCallable, InferenceCallableParameter, InferenceFailureReason, InferenceOutcome, InferenceRecord, InferenceRelation,
        InferenceSession, InferenceTerm,
    };
    use crate::identity::{DeclarationId, InferVarId};
    use crate::types::id::{KindId, RigidTypeVariableId};
    use crate::types::store::TypeStore;
    use phalcom_modules::identity::ModuleId;

    fn test_decl(name: &str) -> DeclarationId {
        DeclarationId::new(ModuleId::universe_root(), name.into())
    }

    #[test]
    fn missing_variable_metadata_is_an_internal_failure() {
        let mut session = InferenceSession::new();
        let outcome = session.failure_outcome(
            InferenceFailureReason::MissingVariableMetadata {
                var: InferVarId::from_index(99),
            },
            None,
            None,
        );

        assert!(matches!(
            outcome,
            InferenceOutcome::InternalFailure(InferenceFailureReason::MissingVariableMetadata { .. })
        ));
    }

    #[test]
    fn nested_frames_share_allocator_without_reusing_variables() {
        let mut session = InferenceSession::new();
        let root = session.root_frame();
        let child = session.begin_frame(Some(root));
        let sibling = session.begin_frame(Some(root));
        let root_var = session.fresh_variable_in_frame(root, KindId::TYPE);
        let child_var = session.fresh_variable_in_frame(child, KindId::TYPE);
        let sibling_var = session.fresh_variable_in_frame(sibling, KindId::TYPE);

        assert_ne!(root_var, child_var);
        assert_ne!(child_var, sibling_var);
        assert_eq!(
            session.variables.iter().find(|variable| variable.id == root_var).map(|variable| variable.frame),
            Some(root)
        );
        assert_eq!(
            session
                .variables
                .iter()
                .find(|variable| variable.id == child_var)
                .map(|variable| variable.frame),
            Some(child)
        );
        assert_eq!(session.frame_parent(child), Some(root));
        assert!(!session.frame_is_closed(child));
        session.close_frame(child);
        assert!(session.frame_is_closed(child));
    }

    #[test]
    fn compound_subtype_uses_declared_variance() {
        use crate::types::parameter::{TypeParameterData, TypeParameterOwner};
        use crate::types::variance::Variance;
        use phalcom_modules::identity::ModuleId;

        let mut store = TypeStore::new();
        let mut hierarchy = crate::types::relation::MapTypeHierarchy::new();
        let int = store.nominal(DeclarationId::new(ModuleId::universe_root(), "Int".into()));
        let number_decl = DeclarationId::new(ModuleId::universe_root(), "Number".into());
        let number = store.nominal(number_decl.clone());
        hierarchy.insert(DeclarationId::new(ModuleId::universe_root(), "Int".into()), number_decl);
        let producer_decl = DeclarationId::new(ModuleId::universe_root(), "Producer".into());
        let owner = TypeParameterOwner::Declaration(producer_decl.clone());
        let _parameter = store.intern_type_parameter(TypeParameterData::new(owner, 0, "T", KindId::TYPE).with_variance(Variance::Covariant));
        let producer_kind = store.arrow_kind(vec![KindId::TYPE].into_boxed_slice(), KindId::TYPE);
        let form = store.nominal_form(producer_decl, producer_kind);
        let mut session = InferenceSession::new();
        let inferred = session.fresh_variable(KindId::TYPE);
        let left = InferenceTerm::Applied {
            origin: Box::new(InferenceTerm::Canonical(form)),
            arguments: super::inference_arguments_from_terms(&[InferenceTerm::Var(inferred)]),
        };
        let right = InferenceTerm::Applied {
            origin: Box::new(InferenceTerm::Canonical(form)),
            arguments: super::inference_arguments_from_terms(&[InferenceTerm::Canonical(number)]),
        };
        session.add_constraint(InferenceRelation::Subtype(left, right), ConstraintOrigin::Explicit, None);
        session.add_constraint(
            InferenceRelation::Equivalent(InferenceTerm::Var(inferred), InferenceTerm::Canonical(int)),
            ConstraintOrigin::Explicit,
            None,
        );
        assert!(session.solve(&mut store, &hierarchy).is_solved());
    }

    #[test]
    fn compound_subtype_matches_canonical_variance_directions() {
        use crate::types::parameter::{TypeParameterData, TypeParameterOwner};
        use crate::types::variance::Variance;
        use phalcom_modules::identity::ModuleId;

        let mut store = TypeStore::new();
        let mut hierarchy = crate::types::relation::MapTypeHierarchy::new();
        let int_decl = DeclarationId::new(ModuleId::universe_root(), "Int".into());
        let number_decl = DeclarationId::new(ModuleId::universe_root(), "Number".into());
        let int = store.nominal(int_decl.clone());
        let number = store.nominal(number_decl.clone());
        hierarchy.insert(int_decl, number_decl);

        let kind = store.arrow_kind(vec![KindId::TYPE].into_boxed_slice(), KindId::TYPE);

        let producer_decl = test_decl("Producer");
        let producer_owner = TypeParameterOwner::Declaration(producer_decl.clone());
        store.intern_type_parameter(TypeParameterData::new(producer_owner, 0, "T", KindId::TYPE).with_variance(Variance::Covariant));
        let producer = store.nominal_form(producer_decl, kind);
        let producer_int = store.apply_type_form(producer, &[int]).unwrap();
        let producer_number = store.apply_type_form(producer, &[number]).unwrap();

        let consumer_decl = test_decl("Consumer");
        let consumer_owner = TypeParameterOwner::Declaration(consumer_decl.clone());
        store.intern_type_parameter(TypeParameterData::new(consumer_owner, 0, "T", KindId::TYPE).with_variance(Variance::Contravariant));
        let consumer = store.nominal_form(consumer_decl, kind);
        let consumer_int = store.apply_type_form(consumer, &[int]).unwrap();
        let consumer_number = store.apply_type_form(consumer, &[number]).unwrap();

        let mut covariant = InferenceSession::new();
        let covariant_var = covariant.fresh_variable(KindId::TYPE);
        covariant.add_constraint(
            InferenceRelation::Subtype(
                InferenceTerm::Applied {
                    origin: Box::new(InferenceTerm::Canonical(producer)),
                    arguments: super::inference_arguments_from_terms(&[InferenceTerm::Var(covariant_var)]),
                },
                InferenceTerm::Applied {
                    origin: Box::new(InferenceTerm::Canonical(producer)),
                    arguments: super::inference_arguments_from_terms(&[InferenceTerm::Canonical(number)]),
                },
            ),
            ConstraintOrigin::Explicit,
            None,
        );
        covariant.add_constraint(
            InferenceRelation::Equivalent(InferenceTerm::Var(covariant_var), InferenceTerm::Canonical(int)),
            ConstraintOrigin::Explicit,
            None,
        );
        assert!(covariant.solve(&mut store, &hierarchy).is_solved());
        assert!(crate::types::relation::is_subtype(&mut store, &hierarchy, producer_int, producer_number));

        let mut contravariant = InferenceSession::new();
        let contravariant_var = contravariant.fresh_variable(KindId::TYPE);
        contravariant.add_constraint(
            InferenceRelation::Subtype(
                InferenceTerm::Applied {
                    origin: Box::new(InferenceTerm::Canonical(consumer)),
                    arguments: super::inference_arguments_from_terms(&[InferenceTerm::Canonical(number)]),
                },
                InferenceTerm::Applied {
                    origin: Box::new(InferenceTerm::Canonical(consumer)),
                    arguments: super::inference_arguments_from_terms(&[InferenceTerm::Var(contravariant_var)]),
                },
            ),
            ConstraintOrigin::Explicit,
            None,
        );
        contravariant.add_constraint(
            InferenceRelation::Equivalent(InferenceTerm::Var(contravariant_var), InferenceTerm::Canonical(int)),
            ConstraintOrigin::Explicit,
            None,
        );
        assert!(contravariant.solve(&mut store, &hierarchy).is_solved());
        assert!(crate::types::relation::is_subtype(&mut store, &hierarchy, consumer_number, consumer_int));
    }

    #[test]
    fn callable_subtype_is_contravariant_in_parameters_and_covariant_in_return() {
        use phalcom_ast::ast::RestMode;

        let mut store = TypeStore::new();
        let mut hierarchy = crate::types::relation::MapTypeHierarchy::new();
        let int_decl = test_decl("Int");
        let number_decl = test_decl("Number");
        hierarchy.insert(int_decl.clone(), number_decl.clone());
        let int = store.nominal(int_decl);
        let number = store.nominal(number_decl);
        let mut session = InferenceSession::new();
        let parameter = session.fresh_variable(KindId::TYPE);
        let result = session.fresh_variable(KindId::TYPE);
        let sub = InferenceTerm::Callable(InferenceCallable {
            parameters: vec![InferenceCallableParameter {
                label: None,
                term: InferenceTerm::Canonical(number),
                rest: RestMode::None,
            }]
            .into_boxed_slice(),
            return_type: Box::new(InferenceTerm::Canonical(int)),
        });
        let sup = InferenceTerm::Callable(InferenceCallable {
            parameters: vec![InferenceCallableParameter {
                label: None,
                term: InferenceTerm::Var(parameter),
                rest: RestMode::None,
            }]
            .into_boxed_slice(),
            return_type: Box::new(InferenceTerm::Var(result)),
        });
        session.add_constraint(InferenceRelation::Subtype(sub, sup), ConstraintOrigin::Explicit, None);
        session.add_constraint(
            InferenceRelation::Equivalent(InferenceTerm::Var(parameter), InferenceTerm::Canonical(int)),
            ConstraintOrigin::Explicit,
            None,
        );
        session.add_constraint(
            InferenceRelation::Equivalent(InferenceTerm::Var(result), InferenceTerm::Canonical(number)),
            ConstraintOrigin::Explicit,
            None,
        );
        assert!(session.solve(&mut store, &hierarchy).is_solved());
    }

    #[test]
    fn higher_kinded_application_binds_constructor_and_argument() {
        let mut store = TypeStore::new();
        let list_decl = test_decl("List");
        let list_kind = store.arrow_kind(vec![KindId::TYPE].into_boxed_slice(), KindId::TYPE);
        let list = store.nominal_form(list_decl, list_kind);
        let int = store.nominal(test_decl("Int"));
        let mut session = InferenceSession::new();
        let constructor = session.fresh_variable(list_kind);
        let argument = session.fresh_variable(KindId::TYPE);
        let applied = InferenceTerm::Applied {
            origin: Box::new(InferenceTerm::Var(constructor)),
            arguments: super::inference_arguments_from_terms(&[InferenceTerm::Var(argument)]),
        };
        let concrete = InferenceTerm::Applied {
            origin: Box::new(InferenceTerm::Canonical(list)),
            arguments: super::inference_arguments_from_terms(&[InferenceTerm::Canonical(int)]),
        };
        session.add_constraint(InferenceRelation::Equivalent(applied, concrete), ConstraintOrigin::Explicit, None);
        assert!(session.solve(&mut store, &crate::types::relation::MapTypeHierarchy::new()).is_solved());
        assert_eq!(session.materialize(&InferenceTerm::Var(constructor), &mut store), Ok(list));
        assert_eq!(session.materialize(&InferenceTerm::Var(argument), &mut store), Ok(int));
    }

    #[test]
    fn invalid_application_is_not_reported_as_underconstraint() {
        let mut store = TypeStore::new();
        let int = store.nominal(test_decl("Int"));
        let error = InferenceSession::new()
            .materialize(
                &InferenceTerm::Applied {
                    origin: Box::new(InferenceTerm::Canonical(int)),
                    arguments: super::inference_arguments_from_terms(&[InferenceTerm::Canonical(int)]),
                },
                &mut store,
            )
            .expect_err("proper type cannot be applied");
        assert!(matches!(error, super::InferenceMaterializationFailure::TypeApplication(_)));
    }

    #[test]
    fn static_data_constructor_unification_solves_const_int_payload() {
        let mut store = TypeStore::new();
        let shape_decl = test_decl("Shape");
        let shape_domain = store.nominal_static_domain(shape_decl.clone());
        let width = store.const_int(3);
        let constructor = store.intern_static_constructor(
            shape_domain,
            shape_decl,
            "Vector",
            vec![store.kind_of(width)].into_boxed_slice(),
        );
        let mut session = InferenceSession::new();
        let width_var = session.fresh_variable(store.kind_of(width));
        let symbolic = InferenceTerm::StaticData {
            domain: shape_domain,
            constructor,
            arguments: super::inference_arguments_from_terms(&[InferenceTerm::Var(width_var)]),
        };
        let concrete = InferenceTerm::StaticData {
            domain: shape_domain,
            constructor,
            arguments: super::inference_arguments_from_terms(&[InferenceTerm::Canonical(width)]),
        };

        session.add_constraint(InferenceRelation::Equivalent(symbolic, concrete), ConstraintOrigin::Explicit, None);
        assert!(session.solve(&mut store, &crate::types::relation::MapTypeHierarchy::new()).is_solved());
        assert_eq!(session.materialize(&InferenceTerm::Var(width_var), &mut store), Ok(width));
    }

    #[test]
    fn static_data_constructor_unification_refutes_domain_and_constructor_mismatch() {
        let mut store = TypeStore::new();
        let shape_decl = test_decl("Shape");
        let other_decl = test_decl("OtherShape");
        let shape_domain = store.nominal_static_domain(shape_decl.clone());
        let other_domain = store.nominal_static_domain(other_decl.clone());
        let vector = store.intern_static_constructor(shape_domain, shape_decl, "Vector", Box::new([]));
        let scalar = store.intern_static_constructor(shape_domain, test_decl("Shape"), "Scalar", Box::new([]));
        let other_vector = store.intern_static_constructor(other_domain, other_decl, "Vector", Box::new([]));
        let mut session = InferenceSession::new();

        let vector_term = InferenceTerm::StaticData {
            domain: shape_domain,
            constructor: vector,
            arguments: Box::new([]),
        };
        let scalar_term = InferenceTerm::StaticData {
            domain: shape_domain,
            constructor: scalar,
            arguments: Box::new([]),
        };
        let other_term = InferenceTerm::StaticData {
            domain: other_domain,
            constructor: other_vector,
            arguments: Box::new([]),
        };

        assert!(matches!(
            session.unify_terms(&vector_term, &scalar_term, &mut store),
            Err(InferenceFailureReason::StructuralMismatch { .. })
        ));
        assert!(matches!(
            session.unify_terms(&vector_term, &other_term, &mut store),
            Err(InferenceFailureReason::StructuralMismatch { .. })
        ));
    }

    #[test]
    fn repeated_deferred_unification_is_idempotent() {
        let mut store = TypeStore::new();
        let form = store.nominal_form(test_decl("Applied"), KindId::TYPE);
        let mut session = InferenceSession::new();
        let outer = session.fresh_variable(KindId::TYPE);
        let nested = session.fresh_variable(KindId::TYPE);
        let term = InferenceTerm::Applied {
            origin: Box::new(InferenceTerm::Canonical(form)),
            arguments: super::inference_arguments_from_terms(&[InferenceTerm::Var(nested)]),
        };

        assert_eq!(
            session.unify_terms(&InferenceTerm::Var(outer), &term, &mut store),
            Ok(super::SolveEffect::Changed)
        );
        assert_eq!(
            session.unify_terms(&InferenceTerm::Var(outer), &term, &mut store),
            Ok(super::SolveEffect::Unchanged)
        );
    }

    #[test]
    fn rigid_parameter_unification_against_structural_term_terminates() {
        use super::InferenceRecordTail;
        use crate::types::parameter::{TypeParameterData, TypeParameterOwner};

        let mut store = TypeStore::new();
        let owner = TypeParameterOwner::Declaration(test_decl("Caller"));
        let parameter = store.intern_type_parameter(TypeParameterData::new(owner, 0, "P", KindId::TYPE));
        let rigid = store.parameter_form(parameter);
        let record = InferenceTerm::Record(InferenceRecord {
            fields: Box::new([]),
            tail: InferenceRecordTail::Closed,
        });

        let mut session = InferenceSession::new();
        let result = session.unify_terms(&InferenceTerm::Canonical(rigid), &record, &mut store);
        assert!(matches!(result, Err(InferenceFailureReason::StructuralMismatch { .. })));
    }

    #[test]
    fn rigid_terms_are_identity_equal_but_never_assignment_targets() {
        let mut store = TypeStore::new();
        let int = store.nominal(test_decl("Int"));
        let same = InferenceTerm::Rigid(RigidTypeVariableId::from_index(0));
        let other = InferenceTerm::Rigid(RigidTypeVariableId::from_index(1));
        let mut session = InferenceSession::new();

        assert_eq!(session.unify_terms(&same, &same, &mut store), Ok(super::SolveEffect::Unchanged));
        assert!(matches!(
            session.unify_terms(&same, &other, &mut store),
            Err(InferenceFailureReason::StructuralMismatch { .. })
        ));
        assert!(matches!(
            session.unify_terms(&same, &InferenceTerm::Canonical(int), &mut store),
            Err(InferenceFailureReason::StructuralMismatch { .. })
        ));
        assert!(session.substitutions.is_empty());
    }

    #[test]
    fn flexible_terms_can_defer_rigid_equality_without_solving_the_rigid() {
        let mut store = TypeStore::new();
        let mut session = InferenceSession::new();
        let variable = session.fresh_variable(KindId::TYPE);
        let rigid = InferenceTerm::Rigid(RigidTypeVariableId::from_index(0));

        assert_eq!(
            session.unify_terms(&InferenceTerm::Var(variable), &rigid, &mut store),
            Ok(super::SolveEffect::Changed)
        );
        assert!(session.substitutions.is_empty());
        assert_eq!(session.var_terms.get(&variable), Some(&rigid));
    }

    #[test]
    fn expected_term_rewrites_solved_nested_variables() {
        let mut store = TypeStore::new();
        let int = store.nominal(test_decl("Int"));
        let mut session = InferenceSession::new();
        let parameter = session.fresh_variable(KindId::TYPE);
        let result = session.fresh_variable(KindId::TYPE);
        let callable = InferenceTerm::Callable(InferenceCallable {
            parameters: Box::new([InferenceCallableParameter {
                label: None,
                term: InferenceTerm::Var(parameter),
                rest: phalcom_ast::ast::RestMode::None,
            }]),
            return_type: Box::new(InferenceTerm::Var(result)),
        });

        session.bind(parameter, int, &store).expect("parameter binding should be valid");

        assert_eq!(
            session.term_for_expected(&callable),
            InferenceTerm::Callable(InferenceCallable {
                parameters: Box::new([InferenceCallableParameter {
                    label: None,
                    term: InferenceTerm::Canonical(int),
                    rest: phalcom_ast::ast::RestMode::None,
                }]),
                return_type: Box::new(InferenceTerm::Var(result)),
            })
        );
    }
}

#[cfg(test)]
mod coordinated_row_completion_tests {
    use super::{InferenceCoordinator, InferenceRecord, InferenceRecordField, InferenceRecordTail, InferenceStaticArgument, InferenceTerm};
    use crate::checker::context::CheckerControl;
    use crate::identity::DeclarationId;
    use crate::types::id::{KindId, TypeId};
    use crate::types::outcome::{CancellationToken, QueryBudget};
    use crate::types::parameter::{TypeParameterData, TypeParameterOwner};
    use crate::types::row::RecordRowField;
    use crate::types::row_solver::{RecordRowShapeSolveResult, RecordRowSolveResult, RecordRowTerm, RecordRowTermTail};
    use crate::types::store::TypeStore;
    use crate::types::relation::MapTypeHierarchy;
    use phalcom_modules::identity::ModuleId;

    fn nominal(store: &mut TypeStore, name: &str) -> TypeId {
        store.nominal(DeclarationId::new(ModuleId::universe_root(), name.into()))
    }

    fn row(fields: Vec<(&str, TypeId)>, tail: RecordRowTermTail) -> RecordRowTerm<InferenceTerm> {
        RecordRowTerm::new(
            fields
                .into_iter()
                .map(|(name, ty)| RecordRowField {
                    name: name.into(),
                    ty: InferenceTerm::Canonical(ty),
                })
                .collect(),
            tail,
        )
        .expect("test row fields are unique")
    }

    fn solve_shape_equations(coordinator: &mut InferenceCoordinator, equations: &[(RecordRowTerm<InferenceTerm>, RecordRowTerm<InferenceTerm>)]) {
        let mut budget = QueryBudget::default();
        let cancellation = CancellationToken::new();
        assert!(matches!(
            coordinator.rows.borrow_mut().solve_shapes_for_roots(equations, &[], &mut budget, &cancellation),
            RecordRowShapeSolveResult::Solved { .. }
        ));
    }

    fn propagate(coordinator: &mut InferenceCoordinator, store: &mut TypeStore) -> Result<bool, Box<super::InferenceOutcome>> {
        coordinator.propagate_with_control(store, &MapTypeHierarchy::new(), &CheckerControl::default())
    }

    fn solved_row_for(
        coordinator: &InferenceCoordinator,
        frame: super::InferenceFrameId,
        variable: crate::types::row_solver::RecordRowVarId,
    ) -> crate::types::row_solver::RecordRowSolution<InferenceTerm> {
        let mut budget = QueryBudget::default();
        let cancellation = CancellationToken::new();
        let RecordRowSolveResult::Solved(solution) = coordinator.row_solution_for_roots(frame, &[variable], &mut budget, &cancellation) else {
            panic!("expected completed row");
        };
        solution
    }

    #[test]
    fn direct_local_row_root_without_evidence_is_underconstrained() {
        let mut coordinator = InferenceCoordinator::new();
        let frame = coordinator.root_frame();
        let root = coordinator.fresh_record_row_variable_in_frame(frame);
        let mut budget = QueryBudget::default();
        let cancellation = CancellationToken::new();

        let result = coordinator.row_solution_for_roots(frame, &[root], &mut budget, &cancellation);
        assert!(matches!(result, RecordRowSolveResult::Underconstrained(value) if value.variables.as_ref() == [root]));
    }

    #[test]
    fn transitive_solver_internal_row_tail_is_underconstrained() {
        let mut coordinator = InferenceCoordinator::new();
        let frame = coordinator.root_frame();
        let root = coordinator.fresh_record_row_variable_in_frame(frame);
        let residual = coordinator.rows.borrow_mut().fresh_var();
        let mut store = TypeStore::new();
        let int = nominal(&mut store, "Int");
        solve_shape_equations(
            &mut coordinator,
            &[(
                RecordRowTerm::new(Vec::new(), RecordRowTermTail::Var(root)).unwrap(),
                row(vec![("a", int)], RecordRowTermTail::Var(residual)),
            )],
        );
        let mut budget = QueryBudget::default();
        let cancellation = CancellationToken::new();

        let result = coordinator.row_solution_for_roots(frame, &[root], &mut budget, &cancellation);
        assert!(matches!(result, RecordRowSolveResult::Underconstrained(value) if value.variables.as_ref() == [root]));
    }

    #[test]
    fn child_row_retains_unresolved_ancestor_dependency_until_parent_solves() {
        let mut coordinator = InferenceCoordinator::new();
        let parent = coordinator.root_frame();
        let ancestor = coordinator.fresh_record_row_variable_in_frame(parent);
        let child = coordinator.begin_frame(Some(parent));
        let local = coordinator.fresh_record_row_variable_in_frame(child);
        let mut store = TypeStore::new();
        let bool_ty = nominal(&mut store, "Bool");
        solve_shape_equations(
            &mut coordinator,
            &[(
                RecordRowTerm::new(Vec::new(), RecordRowTermTail::Var(local)).unwrap(),
                row(vec![("mapped", bool_ty)], RecordRowTermTail::Var(ancestor)),
            )],
        );

        let mut budget = QueryBudget::default();
        let cancellation = CancellationToken::new();
        let child_solution = coordinator.row_solution_for_roots(child, &[local], &mut budget, &cancellation);
        let RecordRowSolveResult::Solved(child_solution) = child_solution else {
            panic!("ancestor dependency must remain symbolic")
        };
        let child_term = InferenceTerm::Record(InferenceRecord {
            fields: Box::new([]),
            tail: super::InferenceRecordTail::Var(local),
        });
        assert!(coordinator.term_has_unresolved_ancestor_row(&child_term, &child_solution, child));

        let string_ty = nominal(&mut store, "String");
        solve_shape_equations(
            &mut coordinator,
            &[(
                RecordRowTerm::new(Vec::new(), RecordRowTermTail::Var(ancestor)).unwrap(),
                row(vec![("cached", bool_ty), ("name", string_ty)], RecordRowTermTail::Closed),
            )],
        );
        let mut budget = QueryBudget::default();
        let resolved = coordinator.row_solution_for_roots(child, &[local], &mut budget, &cancellation);
        let RecordRowSolveResult::Solved(resolved) = resolved else {
            panic!("parent evidence closes child row")
        };
        assert!(!coordinator.term_has_unresolved_ancestor_row(&child_term, &resolved, child));
        let term = resolved.term_for(local).expect("child root has a normalized solution");
        assert_eq!(term.tail, RecordRowTermTail::Closed);
        assert_eq!(
            term.fields.iter().map(|field| field.name.as_ref()).collect::<Vec<_>>(),
            ["cached", "mapped", "name"]
        );
    }

    #[test]
    fn row_occurs_check_finds_ordinary_type_that_already_contains_the_row() {
        let mut coordinator = InferenceCoordinator::new();
        let frame = coordinator.root_frame();
        let row_var = coordinator.fresh_record_row_variable_in_frame(frame);
        let type_var = coordinator.fresh_variable_in_frame(frame, KindId::TYPE);
        let mut store = TypeStore::new();
        let recursive_type = InferenceTerm::Record(InferenceRecord {
            fields: Box::new([]),
            tail: InferenceRecordTail::Var(row_var),
        });
        coordinator
            .unify_terms(&InferenceTerm::Var(type_var), &recursive_type, &mut store)
            .expect("ordinary term first retains the row reference");

        let row_state = coordinator.rows.borrow().clone();
        let var_terms = &coordinator.types.var_terms;
        let aliases = &coordinator.types.var_aliases;
        let equation = (
            RecordRowTerm::new(Vec::new(), RecordRowTermTail::Var(row_var)).unwrap(),
            RecordRowTerm {
                fields: vec![RecordRowField {
                    name: "x".into(),
                    ty: InferenceTerm::Var(type_var),
                }]
                .into_boxed_slice(),
                tail: RecordRowTermTail::Closed,
            },
        );
        let mut budget = QueryBudget::default();
        let cancellation = CancellationToken::new();
        let outcome =
            coordinator
                .rows
                .borrow_mut()
                .solve_shapes_for_roots_with_payload_occurs(&[equation], &[], &mut budget, &cancellation, |target, payload| {
                    super::inference_term_contains_row_variable(payload, target, &row_state, var_terms, aliases, &mut std::collections::HashSet::new())
                });
        assert!(
            matches!(outcome, RecordRowShapeSolveResult::Rejected(crate::types::row_solver::RecordRowFailure::OccursCheckFailed { var, .. }) if var == row_var)
        );
    }

    #[test]
    fn ordinary_occurs_check_finds_row_payload_type_in_later_heterogeneous_application() {
        let mut coordinator = InferenceCoordinator::new();
        let frame = coordinator.root_frame();
        let row_var = coordinator.fresh_record_row_variable_in_frame(frame);
        let type_var = coordinator.fresh_variable_in_frame(frame, KindId::TYPE);
        let mut store = TypeStore::new();
        let payload = InferenceTerm::Var(type_var);
        let equation = (
            RecordRowTerm::new(Vec::new(), RecordRowTermTail::Var(row_var)).unwrap(),
            RecordRowTerm {
                fields: vec![RecordRowField { name: "x".into(), ty: payload }].into_boxed_slice(),
                tail: RecordRowTermTail::Closed,
            },
        );
        solve_shape_equations(&mut coordinator, &[equation]);

        let declaration = DeclarationId::new(ModuleId::universe_root(), "RowCarrier".into());
        let owner = TypeParameterOwner::Declaration(declaration.clone());
        let _row_parameter = store.intern_type_parameter(TypeParameterData::new(owner, 0, "R", KindId::RECORD_ROW));
        let constructor_kind = store.arrow_kind(vec![KindId::RECORD_ROW].into_boxed_slice(), KindId::TYPE);
        let constructor = store.nominal_form(declaration, constructor_kind);
        let application = InferenceTerm::Applied {
            origin: Box::new(InferenceTerm::Canonical(constructor)),
            arguments: Box::new([InferenceStaticArgument::RecordRow(InferenceRecord {
                fields: Box::new([]),
                tail: InferenceRecordTail::Var(row_var),
            })]),
        };

        assert!(matches!(
            coordinator.unify_terms(&InferenceTerm::Var(type_var), &application, &mut store),
            Err(super::InferenceFailureReason::OccursCheck { var }) if var == type_var
        ));
    }

    #[test]
    fn heterogeneous_applied_routes_term_row_and_static_data_children() {
        let mut coordinator = InferenceCoordinator::new();
        let frame = coordinator.root_frame();
        let type_var = coordinator.fresh_variable_in_frame(frame, KindId::TYPE);
        let field_var = coordinator.fresh_variable_in_frame(frame, KindId::TYPE);
        let row_var = coordinator.fresh_record_row_variable_in_frame(frame);
        let mut store = TypeStore::new();
        let int = nominal(&mut store, "Int");
        let string = nominal(&mut store, "String");
        let shape_decl = DeclarationId::new(ModuleId::universe_root(), "Shape".into());
        let shape = store.nominal_static_domain(shape_decl.clone());
        let width = store.const_int(3);
        let width_kind = store.kind_of(width);
        let vector = store.intern_static_constructor(shape, shape_decl, "Vector", vec![width_kind].into_boxed_slice());
        let width_var = coordinator.fresh_variable_in_frame(frame, width_kind);
        let applied_decl = DeclarationId::new(ModuleId::universe_root(), "AppliedCarrier".into());
        let shape_kind = store.static_domain_kind(shape);
        let applied_kind = store.arrow_kind(vec![KindId::TYPE, KindId::RECORD_ROW, shape_kind].into_boxed_slice(), KindId::TYPE);
        let applied_form = store.nominal_form(applied_decl, applied_kind);
        let symbolic_static = InferenceTerm::StaticData {
            domain: shape,
            constructor: vector,
            arguments: vec![InferenceStaticArgument::Term(InferenceTerm::Var(width_var))].into_boxed_slice(),
        };
        let concrete_static = InferenceTerm::StaticData {
            domain: shape,
            constructor: vector,
            arguments: vec![InferenceStaticArgument::Term(InferenceTerm::Canonical(width))].into_boxed_slice(),
        };
        let left = InferenceTerm::Applied {
            origin: Box::new(InferenceTerm::Canonical(applied_form)),
            arguments: vec![
                InferenceStaticArgument::Term(InferenceTerm::Var(type_var)),
                InferenceStaticArgument::RecordRow(InferenceRecord {
                    fields: vec![InferenceRecordField { name: "x".into(), term: InferenceTerm::Var(field_var) }].into_boxed_slice(),
                    tail: InferenceRecordTail::Var(row_var),
                }),
                InferenceStaticArgument::Term(symbolic_static),
            ]
            .into_boxed_slice(),
        };
        let right = InferenceTerm::Applied {
            origin: Box::new(InferenceTerm::Canonical(applied_form)),
            arguments: vec![
                InferenceStaticArgument::Term(InferenceTerm::Canonical(int)),
                InferenceStaticArgument::RecordRow(InferenceRecord {
                    fields: vec![
                        InferenceRecordField { name: "x".into(), term: InferenceTerm::Canonical(int) },
                        InferenceRecordField { name: "y".into(), term: InferenceTerm::Canonical(string) },
                    ]
                    .into_boxed_slice(),
                    tail: InferenceRecordTail::Closed,
                }),
                InferenceStaticArgument::Term(concrete_static),
            ]
            .into_boxed_slice(),
        };

        coordinator.unify_terms(&left, &right, &mut store).expect("heterogeneous applied terms have matching categories");
        assert!(propagate(&mut coordinator, &mut store).expect("coordinated solving"));
        assert!(coordinator.finish_root().is_solved());
        assert_eq!(coordinator.materialize(&InferenceTerm::Var(type_var), &mut store), Ok(int));
        assert_eq!(coordinator.materialize(&InferenceTerm::Var(field_var), &mut store), Ok(int));
        assert_eq!(coordinator.materialize(&InferenceTerm::Var(width_var), &mut store), Ok(width));
        let solution = solved_row_for(&coordinator, frame, row_var);
        let remainder = solution.term_for(row_var).expect("row root solution");
        assert_eq!(remainder.tail, RecordRowTermTail::Closed);
        assert_eq!(remainder.fields.as_ref(), &[RecordRowField { name: "y".into(), ty: InferenceTerm::Canonical(string) }]);
    }

    #[test]
    fn heterogeneous_static_data_routes_row_and_term_children() {
        let mut coordinator = InferenceCoordinator::new();
        let frame = coordinator.root_frame();
        let type_var = coordinator.fresh_variable_in_frame(frame, KindId::TYPE);
        let field_var = coordinator.fresh_variable_in_frame(frame, KindId::TYPE);
        let row_var = coordinator.fresh_record_row_variable_in_frame(frame);
        let mut store = TypeStore::new();
        let int = nominal(&mut store, "Int");
        let string = nominal(&mut store, "String");
        let declaration = DeclarationId::new(ModuleId::universe_root(), "RecordShape".into());
        let domain = store.nominal_static_domain(declaration.clone());
        let constructor = store.intern_static_constructor(domain, declaration, "Tagged", vec![KindId::RECORD_ROW, KindId::TYPE].into_boxed_slice());
        let left = InferenceTerm::StaticData {
            domain,
            constructor,
            arguments: vec![
                InferenceStaticArgument::RecordRow(InferenceRecord {
                    fields: vec![InferenceRecordField { name: "x".into(), term: InferenceTerm::Var(field_var) }].into_boxed_slice(),
                    tail: InferenceRecordTail::Var(row_var),
                }),
                InferenceStaticArgument::Term(InferenceTerm::Var(type_var)),
            ]
            .into_boxed_slice(),
        };
        let right = InferenceTerm::StaticData {
            domain,
            constructor,
            arguments: vec![
                InferenceStaticArgument::RecordRow(InferenceRecord {
                    fields: vec![
                        InferenceRecordField { name: "x".into(), term: InferenceTerm::Canonical(int) },
                        InferenceRecordField { name: "y".into(), term: InferenceTerm::Canonical(string) },
                    ]
                    .into_boxed_slice(),
                    tail: InferenceRecordTail::Closed,
                }),
                InferenceStaticArgument::Term(InferenceTerm::Canonical(int)),
            ]
            .into_boxed_slice(),
        };

        coordinator.unify_terms(&left, &right, &mut store).expect("static constructor children have matching categories");
        propagate(&mut coordinator, &mut store).expect("coordinated solving");
        assert!(coordinator.finish_root().is_solved());
        assert_eq!(coordinator.materialize(&InferenceTerm::Var(type_var), &mut store), Ok(int));
        assert_eq!(coordinator.materialize(&InferenceTerm::Var(field_var), &mut store), Ok(int));
        let solution = solved_row_for(&coordinator, frame, row_var);
        let remainder = solution.term_for(row_var).expect("row root solution");
        assert_eq!(remainder.tail, RecordRowTermTail::Closed);
        assert_eq!(remainder.fields.as_ref(), &[RecordRowField { name: "y".into(), ty: InferenceTerm::Canonical(string) }]);
    }

    #[test]
    fn row_extension_payload_materializes_after_ordinary_variable_solves() {
        let mut coordinator = InferenceCoordinator::new();
        let frame = coordinator.root_frame();
        let payload = coordinator.fresh_variable_in_frame(frame, KindId::TYPE);
        let row_var = coordinator.fresh_record_row_variable_in_frame(frame);
        let mut store = TypeStore::new();
        let string = nominal(&mut store, "String");
        let applied_kind = store.arrow_kind(vec![KindId::TYPE, KindId::RECORD_ROW].into_boxed_slice(), KindId::TYPE);
        let applied_form = store.nominal_form(DeclarationId::new(ModuleId::universe_root(), "RowPayloadCarrier".into()), applied_kind);
        let left = InferenceTerm::Applied {
            origin: Box::new(InferenceTerm::Canonical(applied_form)),
            arguments: vec![
                InferenceStaticArgument::Term(InferenceTerm::Var(payload)),
                InferenceStaticArgument::RecordRow(InferenceRecord {
                    fields: Box::new([]),
                    tail: InferenceRecordTail::Var(row_var),
                }),
            ]
            .into_boxed_slice(),
        };
        let right = InferenceTerm::Applied {
            origin: Box::new(InferenceTerm::Canonical(applied_form)),
            arguments: vec![
                InferenceStaticArgument::Term(InferenceTerm::Canonical(string)),
                InferenceStaticArgument::RecordRow(InferenceRecord {
                    fields: vec![InferenceRecordField { name: "extra".into(), term: InferenceTerm::Var(payload) }].into_boxed_slice(),
                    tail: InferenceRecordTail::Closed,
                }),
            ]
            .into_boxed_slice(),
        };

        coordinator.unify_terms(&left, &right, &mut store).expect("row extension and ordinary payload are compatible");
        propagate(&mut coordinator, &mut store).expect("coordinated solving");
        assert!(coordinator.finish_root().is_solved());
        let materialized = coordinator
            .materialize(
                &InferenceTerm::Record(InferenceRecord { fields: Box::new([]), tail: InferenceRecordTail::Var(row_var) }),
                &mut store,
            )
            .expect("solved residual row materializes");
        let crate::types::store::TypeData::Record(row) = store.get(materialized) else {
            panic!("expected materialized residual record");
        };
        assert_eq!(store.record_row(*row).fields.as_ref(), &[crate::types::row::RecordRowField { name: "extra".into(), ty: string }]);
    }

    #[test]
    fn conflicting_same_label_payloads_are_semantic_inference_conflicts() {
        let mut coordinator = InferenceCoordinator::new();
        coordinator.root_frame();
        let mut store = TypeStore::new();
        let int = nominal(&mut store, "Int");
        let string = nominal(&mut store, "String");
        let conflict_kind = store.arrow_kind(vec![KindId::RECORD_ROW].into_boxed_slice(), KindId::TYPE);
        let form = store.nominal_form(DeclarationId::new(ModuleId::universe_root(), "ConflictCarrier".into()), conflict_kind);
        let left = InferenceTerm::Applied {
            origin: Box::new(InferenceTerm::Canonical(form)),
            arguments: Box::new([InferenceStaticArgument::RecordRow(InferenceRecord {
                fields: vec![InferenceRecordField { name: "x".into(), term: InferenceTerm::Canonical(int) }].into_boxed_slice(),
                tail: InferenceRecordTail::Closed,
            })]),
        };
        let right = InferenceTerm::Applied {
            origin: Box::new(InferenceTerm::Canonical(form)),
            arguments: Box::new([InferenceStaticArgument::RecordRow(InferenceRecord {
                fields: vec![InferenceRecordField { name: "x".into(), term: InferenceTerm::Canonical(string) }].into_boxed_slice(),
                tail: InferenceRecordTail::Closed,
            })]),
        };

        coordinator.unify_terms(&left, &right, &mut store).expect("row payload equality is deferred to ordinary inference");
        let outcome = propagate(&mut coordinator, &mut store).expect_err("incompatible payloads reject semantically");
        assert!(matches!(*outcome, super::InferenceOutcome::Conflicting(_)));
    }
}
