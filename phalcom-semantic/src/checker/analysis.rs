//! Formal expression and callable analysis product models (Spec 04.5).

use super::analysis_map::AnalysisMap;
use crate::checker::causal::CausalInvalidity;
use crate::checker::incident::InternalSemanticIncident;
use crate::diagnostic::SemanticDiagnostic;
use crate::identity::{
    AnalysisIncidentId, BindingId, CallResolutionId, CallableId, CallableParameterId, DiagnosticCauseId, ExplanationId, ExpressionId, FieldId, ImplId,
    SourceOwner,
};
use crate::types::denotation::SemanticDenotation;
use crate::types::evidence::{DynamicReason, TypeKnowledge};
use crate::types::id::{TypeId, TypeParameterId};
use crate::types::outcome::{BlockReason, BudgetReport};
use phalcom_common::range::SourceRange;
use phalcom_type_meta::allocation::{AllocationBudget, AllocationError};
use std::collections::{BTreeSet, HashMap};
use std::ops::Range;
use std::sync::Arc;

/// Analysis outcome status for a single expression.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AnalysisStatus {
    Ready,
    Invalid(DiagnosticCauseId),
    Suppressed(crate::checker::causal::SuppressionCause),
    Blocked(BlockReason),
    DynamicBoundary(DynamicReason),
    Cancelled,
    BudgetExceeded(BudgetReport),
    InternalFailure(AnalysisIncidentId),
}

impl AnalysisStatus {
    pub fn is_ready(&self) -> bool {
        matches!(self, Self::Ready)
    }

    pub fn is_invalid(&self) -> bool {
        matches!(self, Self::Invalid(_))
    }
}

/// Comprehensive analysis product for an analyzed expression.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExpressionAnalysis {
    pub id: ExpressionId,
    pub range: SourceRange,
    pub knowledge: TypeKnowledge,
    pub callable: Option<CallableId>,
    /// Canonical generic arguments selected for this call site. Lowering may
    /// project this product into a runtime recipe but must not redo inference.
    pub call_specialization: Option<CallSpecialization>,
    /// Canonical per-expression evidence that dispatch selected a conditional
    /// inherent implementation. Lowering consumes this directly and must not
    /// reconstruct the decision from target indexes.
    pub conditional_dispatch: Option<crate::dispatch::ConditionalDispatchSelection>,
    /// Proof-bearing semantic capability for the selected conditional call
    /// edge. Runtime lowering projects this product and never re-solves it.
    pub conditional_capability: Option<crate::dispatch::ConditionalSelectedCallCapability>,
    /// Exact per-expression trait evidence used by semantic-to-core lowering.
    pub trait_dispatch: Option<crate::trait_dispatch::TraitDispatchSite>,
    /// Candidate provenance retained when trait dispatch is ambiguous. This
    /// keeps requirement/evidence origins available to diagnostics and editor
    /// consumers without pretending one candidate was selected.
    pub trait_dispatch_candidates: Option<Box<[crate::trait_dispatch::TraitEvidencedMemberCandidate]>>,
    /// Canonical target-side algebra operation selected for a contextualized
    /// basis value, imaginary literal, or real scalar term.
    pub contextual_realization: Option<Arc<ContextualRealization>>,
    pub denotation: Option<SemanticDenotation>,
    pub status: AnalysisStatus,
    pub causal_invalidity: CausalInvalidity,
    pub explanation: Option<ExplanationId>,
    pub call: Option<CallResolutionId>,
    pub binding_read_safety: Option<crate::checker::flow::BindingReadSafety>,
    /// Exact lexical binding selected for a local variable read.
    pub binding_read: Option<BindingId>,
    /// Exact lexical binding selected as an assignment target.
    pub binding_write: Option<BindingId>,
}

impl ExpressionAnalysis {
    pub fn ready(id: ExpressionId, range: SourceRange, knowledge: TypeKnowledge) -> Self {
        Self {
            id,
            range,
            knowledge,
            callable: None,
            call_specialization: None,
            conditional_dispatch: None,
            conditional_capability: None,
            trait_dispatch: None,
            trait_dispatch_candidates: None,
            contextual_realization: None,
            denotation: None,
            status: AnalysisStatus::Ready,
            causal_invalidity: CausalInvalidity::Clean,
            explanation: None,
            call: None,
            binding_read_safety: None,
            binding_read: None,
            binding_write: None,
        }
    }

    pub fn invalid(id: ExpressionId, range: SourceRange, cause: DiagnosticCauseId) -> Self {
        Self {
            id,
            range,
            knowledge: TypeKnowledge::Unknown(crate::types::evidence::UnknownReason::SyntaxError),
            callable: None,
            call_specialization: None,
            conditional_dispatch: None,
            conditional_capability: None,
            trait_dispatch: None,
            trait_dispatch_candidates: None,
            contextual_realization: None,
            denotation: None,
            status: AnalysisStatus::Invalid(cause),
            causal_invalidity: CausalInvalidity::One(cause),
            explanation: None,
            call: None,
            binding_read_safety: None,
            binding_read: None,
            binding_write: None,
        }
    }

    pub fn with_denotation(mut self, denotation: Option<SemanticDenotation>) -> Self {
        self.denotation = denotation;
        self
    }

    pub fn with_explanation(mut self, explanation: ExplanationId) -> Self {
        self.explanation = Some(explanation);
        self
    }

    pub fn with_call(mut self, call: CallResolutionId) -> Self {
        self.call = Some(call);
        self
    }

    pub fn with_status(mut self, status: AnalysisStatus) -> Self {
        self.status = status;
        self
    }
}

/// Canonical axis carried by a source basis value or imaginary suffix.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum AlgebraicBasisAxis {
    I,
    J,
    K,
}

/// How a basis-component coefficient is supplied to the selected capability.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RealizationCoefficientSource {
    /// A source basis unit supplies the multiplicative identity of the
    /// capability's scalar type.
    One { scalar_type: TypeId, one: RealizationCapabilitySelection },
    /// Exact integer spelling and radix from a native imaginary literal.
    IntegerLiteral { digits: Box<str>, radix: u32 },
    /// Exact IEEE-754 payload from a native imaginary literal.
    FloatLiteral { bits: u64 },
    /// A scalar expression selected by a future bounded arithmetic region.
    Expression { expression: ExpressionId, scalar_type: TypeId },
}

/// Durable semantic evidence that one basis value was realized in a target
/// algebra. All type identities are canonical; inference-local variables are
/// deliberately excluded.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContextualRealization {
    /// Expression whose value is replaced by the realization operation.
    pub source_expression: ExpressionId,
    pub source_range: SourceRange,
    /// Basis type for a basis realization, or scalar type for real embedding.
    pub source_type: TypeId,
    pub target_type: TypeId,
    pub operation: ContextualRealizationOperation,
    /// Selected class-side `BasisRealizable` or `RealEmbeddable` requirement.
    pub realization: RealizationCapabilitySelection,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ContextualRealizationOperation {
    Basis {
        basis_type: TypeId,
        axis: AlgebraicBasisAxis,
        coefficient: RealizationCoefficientSource,
        evaluation: RealizationEvaluationPlan,
    },
    RealEmbedding {
        value: ExpressionId,
        scalar_type: TypeId,
    },
}

/// Source-order obligations for constructing a contextual realization.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RealizationEvaluationPlan {
    /// Native suffix coefficient is a literal and the axis value is
    /// materialized after it.
    LiteralCoefficientThenAxis,
    /// For a bare basis value, evaluate the selected `One` requirement, then
    /// evaluate the original basis expression exactly once.
    MultiplicativeIdentityThenSourceBasis,
    /// Arithmetic-region elaboration records both source operands and their
    /// original evaluation order explicitly.
    SourceOperands {
        first: ExpressionId,
        second: ExpressionId,
        coefficient: ExpressionId,
        basis: ExpressionId,
    },
}

/// Executable selection for a class-side realization capability.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RealizationCapabilitySelection {
    /// Exact or derived source conformance evidence.
    Evidenced(crate::trait_dispatch::TraitDispatchSelection),
    /// A direct generic conformance bound carried by the active caller.
    Assumed(crate::trait_dispatch::AssumedTraitDispatchSelection),
}

/// Provenance for one callee generic conformance bound discharged at a call site.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GenericCallConformanceSource {
    Exact {
        evidence: Arc<crate::impls::ConformanceEvidence>,
    },
    Forwarded {
        caller_conformance: crate::generic_evidence::GenericConformanceKey,
    },
    Derived {
        evidence: Arc<crate::generic_evidence::DerivedConformanceEvidence>,
    },
    CompilerDerived {
        evidence: Arc<crate::generic_evidence::CompilerDerivedConformanceEvidence>,
    },
}

/// Binding for one callee generic conformance obligation discharged during callable application.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GenericCallConformanceBinding {
    pub callee_conformance: crate::generic_evidence::GenericConformanceKey,
    pub source: GenericCallConformanceSource,
}

/// Solved generic arguments and discharged conformance bindings for one canonical callable application.
///
/// Each binding retains its canonical static-argument category. Dynamic is an
/// explicit boundary, not a missing binding or a proper `TypeId`. Inference-
/// local variables never cross this publication boundary.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CallSpecialization {
    pub callable: CallableId,
    pub bindings: Box<[(TypeParameterId, crate::types::store::StaticArgument)]>,
    pub conformances: Box<[GenericCallConformanceBinding]>,
}

/// Evidence-backed semantic resolution for one `for` lane.
///
/// The source expression is the stable key.  The compiler consumes the
/// selected requirements and evidence from this product; it must not infer
/// iteration from selector names or repeat associated-type normalization.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ForIterationLaneResolution {
    pub source_expression: ExpressionId,
    pub source_range: SourceRange,
    pub source_type: TypeId,
    pub item_type: TypeId,
    pub iterator_type: TypeId,
    pub iterator: ForIterationRequirementResolution,
    pub next: ForIterationRequirementResolution,
}

/// Executable authority for one implicit iteration requirement.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ForIterationRequirementResolution {
    /// A requirement selected by exact, assumed, or derived conformance
    /// evidence.  Keeping the evidence variant intact is important for
    /// generic callers and for derived `Iterator => Iterable` dispatch.
    Evidence {
        conformance: crate::generic_evidence::GenericConformanceKey,
        evidence: crate::generic_evidence::GenericConformanceEvidence,
        requirement: crate::traits::TraitRequirementId,
        signature: crate::signature::CallableSemanticSignature,
    },
    /// A trait default body is checked under an active abstract requirement.
    AbstractRequirement {
        requirement: crate::traits::TraitRequirementId,
        signature: crate::signature::CallableSemanticSignature,
    },
}

/// Deterministic index of canonical semantic `for` lane resolutions.
pub type ForIterationLaneResolutionIndex = AnalysisMap<ExpressionId, ForIterationLaneResolution>;

pub use crate::checker::static_iteration::{StaticIterationItemDomain, StaticIterationLaneResolution, StaticIterationLaneResolutionIndex, StaticIterationMode};

/// Evidence-backed semantic resolution for one positional expansion site (`*expr`).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PositionalExpansionIterationResolution {
    pub source_expression: ExpressionId,
    pub source_range: SourceRange,
    pub source_type: TypeId,
    pub item_type: TypeId,
    pub iterator_type: TypeId,
    pub iterator: ForIterationRequirementResolution,
    pub next: ForIterationRequirementResolution,
}

/// Deterministic index of canonical semantic positional expansion resolutions.
pub type PositionalExpansionIterationResolutionIndex = AnalysisMap<ExpressionId, PositionalExpansionIterationResolution>;

/// A statically known positional expansion source for which canonical
/// Iterable/Iterator evidence was not available. Lowering consumes this
/// product to fail closed without re-solving the semantic question.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KnownInvalidPositionalExpansion {
    pub source_expression: ExpressionId,
    pub source_range: SourceRange,
    pub source_type: TypeId,
}

pub type KnownInvalidPositionalExpansionIndex = AnalysisMap<ExpressionId, KnownInvalidPositionalExpansion>;

/// Binding flow state tracking persistent declared constraint vs. current path knowledge.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BindingState {
    pub binding: BindingId,
    pub name: String,
    pub parameter: Option<CallableParameterId>,
    pub range: SourceRange,
    pub contract: Option<super::binding::BindingContract>,
    pub current: TypeKnowledge,
    pub denotation: Option<SemanticDenotation>,
    pub consistency: super::binding::BindingConsistency,
    pub causal_invalidity: super::causal::CausalInvalidity,
    pub mutable: bool,
    pub initialization: crate::checker::flow::InitializationState,
    pub version: u32,
    pub explanation: Option<ExplanationId>,
}

impl BindingState {
    pub(crate) fn clone_with_allocation_budget(&self, allocations: &mut AllocationBudget) -> Result<Self, AllocationError> {
        let name = allocations.string(&self.name)?;
        let parameter = self.parameter.as_ref().map(|p| p.clone_with_allocation_budget(allocations)).transpose()?;
        let contract = self.contract.clone();
        let current = self.current.clone_with_allocation_budget(allocations)?;
        let denotation = self.denotation.as_ref().map(|d| d.clone_with_allocation_budget(allocations)).transpose()?;
        let consistency = self.consistency.clone_with_allocation_budget(allocations)?;
        Ok(Self {
            binding: self.binding,
            name,
            parameter,
            range: self.range,
            contract,
            current,
            denotation,
            consistency,
            causal_invalidity: self.causal_invalidity,
            mutable: self.mutable,
            initialization: self.initialization,
            version: self.version,
            explanation: self.explanation,
        })
    }

    pub fn declared_type(&self) -> Option<crate::types::id::TypeId> {
        self.contract.as_ref().map(|contract| contract.ty)
    }

    pub fn from_seed(binding: BindingId, seed: super::binding::BindingSeed, current: TypeKnowledge, consistency: super::binding::BindingConsistency) -> Self {
        Self {
            binding,
            name: seed.name,
            parameter: seed.parameter,
            range: seed.range,
            contract: seed.contract,
            current,
            denotation: seed.denotation,
            consistency,
            causal_invalidity: seed.causal_invalidity,
            mutable: seed.mutable,
            initialization: seed.initialization,
            version: 0,
            explanation: None,
        }
    }

    pub fn new(
        binding: BindingId,
        name: impl Into<String>,
        range: SourceRange,
        declared: Option<crate::types::id::TypeId>,
        current: TypeKnowledge,
        mutable: bool,
    ) -> Self {
        let contract = declared.map(|ty| super::binding::BindingContract {
            ty,
            origin: super::binding::BindingContractOrigin::SourceAnnotation,
            source: Some(range),
        });
        Self::new_with_contract(
            binding,
            name,
            range,
            contract,
            current,
            None,
            mutable,
            crate::checker::flow::InitializationState::DefinitelyInitialized,
        )
    }

    pub fn new_with_contract(
        binding: BindingId,
        name: impl Into<String>,
        range: SourceRange,
        contract: Option<super::binding::BindingContract>,
        current: TypeKnowledge,
        denotation: Option<SemanticDenotation>,
        mutable: bool,
        initialization: crate::checker::flow::InitializationState,
    ) -> Self {
        Self {
            binding,
            name: name.into(),
            parameter: None,
            range,
            contract,
            current,
            denotation,
            consistency: super::binding::BindingConsistency::Unconstrained,
            causal_invalidity: super::causal::CausalInvalidity::Clean,
            mutable,
            initialization,
            version: 0,
            explanation: None,
        }
    }
}

/// Summary of flow state at entry or exit boundary.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FlowBindingSummary {
    pub knowledge: TypeKnowledge,
    pub contract: Option<super::binding::BindingContract>,
    pub consistency: super::binding::BindingConsistency,
    pub mutable: bool,
    pub initialization: crate::checker::flow::InitializationState,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FlowFieldSummary {
    pub contract: TypeKnowledge,
    pub current: TypeKnowledge,
    pub initialization: crate::checker::flow::FieldInitialization,
    pub validity: crate::checker::flow::FieldContractValidity,
    pub causal_invalidity: CausalInvalidity,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct FlowStateSummary {
    pub bindings: AnalysisMap<BindingId, FlowBindingSummary>,
    pub fields: AnalysisMap<FieldId, FlowFieldSummary>,
    pub fact_count: usize,
}

/// A single recorded normal exit fact.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NormalReturnFact {
    pub knowledge: TypeKnowledge,
    pub flow: FlowStateSummary,
    pub status: AnalysisStatus,
    pub causal_invalidity: CausalInvalidity,
    /// Canonical callable whose result is this exit's root call, when the
    /// normal-exit expression itself is a resolved call. This intentionally
    /// excludes calls nested under other result-producing expressions.
    pub direct_result_dependency: Option<crate::identity::CallableId>,
}

pub(crate) fn direct_result_dependency(
    expression: &phalcom_ast::ast::Expr,
    callable: Option<&crate::identity::CallableId>,
) -> Option<crate::identity::CallableId> {
    matches!(expression, phalcom_ast::ast::Expr::UnqualifiedCall(_) | phalcom_ast::ast::Expr::MethodCall(_))
        .then(|| callable.cloned())
        .flatten()
}

impl NormalReturnFact {
    pub fn publication_knowledge(&self) -> TypeKnowledge {
        if self.causal_invalidity != CausalInvalidity::Clean {
            return TypeKnowledge::Unknown(crate::types::evidence::UnknownReason::SuppressedByInvalidCause);
        }
        match &self.status {
            AnalysisStatus::Ready => self.knowledge.clone(),
            AnalysisStatus::Invalid(_) | AnalysisStatus::Suppressed(_) => {
                TypeKnowledge::Unknown(crate::types::evidence::UnknownReason::SuppressedByInvalidCause)
            }
            AnalysisStatus::Blocked(_) => TypeKnowledge::Unknown(crate::types::evidence::UnknownReason::InferenceBlocked),
            AnalysisStatus::DynamicBoundary(reason) => match &self.knowledge {
                TypeKnowledge::Dynamic(_) => self.knowledge.clone(),
                _ => TypeKnowledge::Dynamic(reason.clone()),
            },
            AnalysisStatus::Cancelled => TypeKnowledge::Unknown(crate::types::evidence::UnknownReason::InferenceCancelled),
            AnalysisStatus::BudgetExceeded(_) => TypeKnowledge::Unknown(crate::types::evidence::UnknownReason::InferenceBudgetExceeded),
            AnalysisStatus::InternalFailure(_) => TypeKnowledge::Unknown(crate::types::evidence::UnknownReason::InferenceBlocked),
        }
    }
}

/// Recorded exit facts across returns, throws, and unreachable points.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct BodyExitFacts {
    pub normal_returns: Vec<NormalReturnFact>,
    pub throws: Vec<FlowStateSummary>,
    pub unreachable: bool,
}

/// Joins values from all normal callable exits into one published return
/// knowledge fact. Abrupt-only bodies produce `Never`; incomplete knowledge
/// keeps the summary incomplete instead of inventing a return type.
pub fn normal_return_summary(store: &mut crate::types::store::TypeStore, exits: &[NormalReturnFact]) -> TypeKnowledge {
    use crate::types::evidence::{EvidenceOrigin, TypeKnowledge, join_type_knowledge};

    if exits.is_empty() {
        return TypeKnowledge::established(store.never(), EvidenceOrigin::Flow);
    }

    join_type_knowledge(store, exits.iter().map(NormalReturnFact::publication_knowledge))
}

/// Index of expression analysis products within a body.
pub type ExpressionAnalysisIndex = AnalysisMap<ExpressionId, ExpressionAnalysis>;

/// Index of binding states within a body.
pub type BindingAnalysisIndex = AnalysisMap<BindingId, BindingState>;

/// Storage authority for one binding handle retained by a callable analysis.
///
/// Body-local handles are qualified by the owning [`CallableAnalysis`]. A
/// module value keeps its canonical linker symbol and, for a selective
/// import, the exact module-local import binding used to reach it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CallableBindingOrigin {
    /// A lexical binding whose storage belongs to this callable body.
    BodyLocal,
    /// A module-owned global value seeded into this callable's lexical scope.
    ModuleValue {
        /// Canonical storage identity shared by declarations and imported aliases.
        symbol: phalcom_modules::linker::SymbolId,
        /// Exact linked import slot, absent for a declaration owned by this module.
        import: Option<phalcom_modules::linker::ImportBindingId>,
    },
}

/// Storage-origin map keyed by body-local analysis handles.
pub type CallableBindingOriginIndex = AnalysisMap<BindingId, CallableBindingOrigin>;

use crate::identity::{DeclarationId, ModuleId};

/// Semantic dependency representing query-invalidating semantic consumption by a callable body.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum SemanticDependency {
    DeclarationShell(DeclarationId),
    TraitHeader(DeclarationId),
    CallableSignature(CallableId),
    FieldSignature(FieldId),
    DeclarationSurface(DeclarationId),
    TraitSurface(DeclarationId),
    ConformanceDispatch(ImplId),
    TraitDispatchBucket(crate::trait_dispatch::TraitDispatchBucketKey),
    /// Context-independent exact proof state for one canonical proposition.
    /// Contextual assumed/derived results remain embedded in their owner.
    GenericConformance(crate::generic_evidence::GenericConformanceKey),
    HierarchyEdge(DeclarationId),
    LinkedInterface(ModuleId),
    DataDeclaration(DeclarationId),
    EnumDeclaration(DeclarationId),
    AssociatedSurface(DeclarationId),
    TypeMemberDeclaration(crate::types::type_member::TypeMemberDeclarationId),
    StaticDomainCaseCatalog(crate::identity::StaticDomainDeclarationId),
    StaticCaseSignature(crate::identity::StaticCaseId),
    StaticDefinition(DeclarationId),
    StaticConstant(crate::static_constant::StaticConstantId),
    ModuleStaticBindingCatalog(ModuleId),
    ConstructorInitialization(DeclarationId),
    ResolvedImport(phalcom_modules::identity::ImportSiteId),
    LinkedName(ModuleId, String),
    PublicExport(ModuleId, String),
    ModuleStaticBindings(ModuleId),
}

/// Status of callable-body analysis.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum CallableAnalysisStatus {
    Complete,
    Partial,
    Blocked,
    Cancelled,
    BudgetExceeded,
    InternalFailure(AnalysisIncidentId),
}

impl CallableAnalysisStatus {
    /// Returns whether this body analysis has reached the complete status.
    ///
    /// Artifact publication separately validates that the required body facts
    /// are present and represented.
    pub const fn is_complete(self) -> bool {
        matches!(self, Self::Complete)
    }
}

#[cfg(test)]
mod callable_analysis_status_tests {
    use super::CallableAnalysisStatus;
    use crate::identity::InternalSemanticIncidentId;

    #[test]
    fn only_complete_callable_analysis_is_publishable() {
        assert!(CallableAnalysisStatus::Complete.is_complete());
        assert!(!CallableAnalysisStatus::Partial.is_complete());
        assert!(!CallableAnalysisStatus::Blocked.is_complete());
        assert!(!CallableAnalysisStatus::Cancelled.is_complete());
        assert!(!CallableAnalysisStatus::BudgetExceeded.is_complete());
        assert!(!CallableAnalysisStatus::InternalFailure(InternalSemanticIncidentId(0)).is_complete());
    }
}

/// Exact semantic link from an explicit constructor-chain call to its target.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct ConstructorSuperCallResolution {
    /// Exact source expression resolved by semantic analysis.
    pub call_range: SourceRange,
    /// Canonical direct-parent instance initializer selected for that call.
    pub initializer: CallableId,
}

/// Published semantic analysis product for a single callable body.
/// Canonical collection family selected for a bracket literal.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum SequenceLiteralFamily {
    List,
    Array,
    Vector,
}

/// Construction plan for literal lowering.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum SequenceConstructionPlan {
    Fixed {
        element_count: usize,
    },
    Expanded,
    /// A lone native Vector spread with canonically identical symbolic extent.
    /// This proof authorizes copying storage without interpreting a runtime N.
    VectorIdentitySpread {
        source_range: SourceRange,
        extent: TypeId,
    },
}

/// Checked semantic resolution product for a bracket literal (COLL006.C1.P1).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SequenceLiteralResolution {
    pub source_expression: ExpressionId,
    pub source_range: SourceRange,
    pub family: SequenceLiteralFamily,
    pub result_type: TypeId,
    pub element_type: TypeId,
    pub extent: Option<TypeId>,
    pub construction: SequenceConstructionPlan,
    /// Intrinsic positional execution lanes whose cardinality is proved.
    pub exact_expansions: AnalysisMap<SourceRange, SequenceExpansionResolution>,
}

/// Proof-preserving positional spread for dimensioned sequence construction.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum SequenceExpansionResolution {
    /// Closed product positional lane; labeled entries are excluded.
    TuplePositionals { extent: TypeId },
    /// Native immutable Vector storage, independent from virtual iteration.
    VectorStorage { extent: TypeId },
}

pub type SequenceLiteralResolutionIndex = AnalysisMap<ExpressionId, SequenceLiteralResolution>;

#[derive(Clone, Debug)]
pub struct CallableAnalysis {
    pub callable: CallableId,
    pub body_range: SourceRange,
    /// Explicit source namespace when a callable body is checked against
    /// declarations owned outside its callable shard (for example, the
    /// synthetic module-body `<main>` analysis).
    pub body_source_owner: Option<SourceOwner>,
    pub expressions: ExpressionAnalysisIndex,
    pub bindings: BindingAnalysisIndex,
    /// Explicit storage authority for every binding state retained by this
    /// callable. `BindingId` remains body-local even when its origin is a
    /// module-owned global symbol.
    pub binding_origins: CallableBindingOriginIndex,
    /// Canonical evidence-backed iteration facts consumed by core lowering.
    pub for_iteration_lanes: Arc<ForIterationLaneResolutionIndex>,
    /// Canonical evidence-backed positional expansion facts consumed by core lowering.
    pub positional_expansions: Arc<PositionalExpansionIterationResolutionIndex>,
    /// Known invalid positional expansion facts consumed by core lowering.
    pub known_invalid_positional_expansions: Arc<KnownInvalidPositionalExpansionIndex>,
    /// Checked sequence literal resolution facts consumed by core lowering.
    pub sequence_literals: Arc<SequenceLiteralResolutionIndex>,
    pub associated_resolutions: Arc<crate::checker::associated::AssociatedResolutionIndex>,
    pub callable_reference_resolutions: Arc<crate::checker::associated::CallableReferenceResolutionIndex>,
    pub family_applications: Arc<crate::checker::associated::FamilyApplicationResolutionIndex>,
    pub match_resolutions: Arc<crate::types::proof::FrozenMatchResolutionIndex>,
    pub pattern_site_resolutions: Arc<crate::types::proof::FrozenPatternSiteResolutionIndex>,
    /// Immutable definitions for every rigid in retained match/pattern proofs.
    pub local_proof_graph: Arc<crate::types::proof::CallableProofGraph>,
    pub flow_graph: Arc<crate::checker::flow::graph::FlowGraph>,
    pub entry_flow: FlowStateSummary,
    pub exits: BodyExitFacts,
    pub return_validation: crate::signature::ReturnContractValidation,
    pub diagnostics: Arc<[SemanticDiagnostic]>,
    /// Structured analyzer incidents kept separate from source diagnostics.
    pub internal_incidents: Arc<[InternalSemanticIncident]>,
    pub explanations: Arc<crate::explain::ExplanationArena>,
    pub return_explanation: Option<ExplanationId>,
    pub dependencies: Arc<[CallableId]>,
    pub semantic_dependencies: Arc<[SemanticDependency]>,
    /// Canonical direct-parent initializer selected by the explicit chain call
    /// at the recorded source site, when this constructor has a valid prologue.
    pub constructor_super_initializer: Option<ConstructorSuperCallResolution>,
    /// Semantic result fingerprint used to decide whether downstream queries
    /// must propagate a body refresh. Despite its legacy field name, this is
    /// not a hash of dependency-edge metadata.
    pub dependency_fingerprint: crate::db::ProductFingerprint,
    pub status: CallableAnalysisStatus,
    /// Joined use-phases for each imported source binding observed in this callable body.
    pub observed_import_phases: Arc<
        std::collections::BTreeMap<
            phalcom_modules::identity::SourceImportBindingId,
            phalcom_modules::graph::DependencyPhase,
        >,
    >,
}

/// A frozen callable table's rows are grouped by module and ordered by
/// callable identity within each module. The module index stores charged row
/// ranges, so module iteration never scans unrelated callables or allocates a
/// shadow lookup table.
#[derive(Clone, Debug)]
struct FrozenCallableAnalysisTable {
    rows: Arc<Vec<(CallableId, Arc<CallableAnalysis>)>>,
    module_ranges: AnalysisMap<crate::identity::ModuleId, Range<usize>>,
}

#[derive(Clone, Debug)]
struct SourceCallableAnalysisTable {
    by_id: HashMap<CallableId, Arc<CallableAnalysis>>,
    by_module: HashMap<crate::identity::ModuleId, BTreeSet<CallableId>>,
}

#[derive(Clone, Debug)]
enum CallableAnalysisTableStorage {
    Source(SourceCallableAnalysisTable),
    Frozen(FrozenCallableAnalysisTable),
}

/// Failure while creating the immutable imported callable-table backing.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CallableAnalysisTableBuildError {
    /// Two rows use the same stable callable identity.
    DuplicateCallable,
    /// A charged buffer or shared owner exceeded the load allocation budget.
    Allocation(AllocationError),
    /// Construction of the charged module-range index failed.
    ModuleIndex(crate::checker::analysis_map::AnalysisMapBuildError),
}

/// Error returned when a source mutation is attempted on an imported table.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FrozenCallableAnalysisTableMutation;

/// Module-indexed table of callable analyses. Source checking uses mutable
/// hash/tree indexes; artifact import uses charged immutable sorted rows.
#[derive(Clone, Debug)]
pub struct CallableAnalysisTable {
    storage: CallableAnalysisTableStorage,
}

impl Default for CallableAnalysisTable {
    fn default() -> Self {
        Self::new()
    }
}

impl CallableAnalysisTable {
    /// Creates an empty source-side callable table.
    pub fn new() -> Self {
        Self {
            storage: CallableAnalysisTableStorage::Source(SourceCallableAnalysisTable {
                by_id: HashMap::new(),
                by_module: HashMap::new(),
            }),
        }
    }

    /// Takes ownership of rows whose vector storage was already charged to
    /// `allocations`, then builds the immutable module ranges against that
    /// same transaction ledger.
    pub fn from_budgeted_entries(
        mut rows: Vec<(CallableId, Arc<CallableAnalysis>)>,
        allocations: &mut AllocationBudget,
    ) -> Result<Self, CallableAnalysisTableBuildError> {
        rows.sort_unstable_by(|left, right| left.0.owner.module().cmp(right.0.owner.module()).then_with(|| left.0.cmp(&right.0)));
        if rows.windows(2).any(|pair| pair[0].0 == pair[1].0) {
            return Err(CallableAnalysisTableBuildError::DuplicateCallable);
        }

        let module_count = rows.windows(2).filter(|pair| pair[0].0.owner.module() != pair[1].0.owner.module()).count() + usize::from(!rows.is_empty());
        let mut module_ranges = allocations.vector(module_count).map_err(CallableAnalysisTableBuildError::Allocation)?;
        let mut start = 0;
        while start < rows.len() {
            let module = rows[start]
                .0
                .owner
                .module()
                .clone_with_allocation_budget(allocations)
                .map_err(CallableAnalysisTableBuildError::Allocation)?;
            let mut end = start + 1;
            while end < rows.len() && rows[end].0.owner.module() == &module {
                end += 1;
            }
            module_ranges.push((module, start..end));
            start = end;
        }
        let module_ranges = AnalysisMap::from_budgeted_entries(module_ranges, allocations).map_err(CallableAnalysisTableBuildError::ModuleIndex)?;
        let rows = allocations.arc(rows).map_err(CallableAnalysisTableBuildError::Allocation)?;

        Ok(Self {
            storage: CallableAnalysisTableStorage::Frozen(FrozenCallableAnalysisTable { rows, module_ranges }),
        })
    }

    /// Inserts or replaces a source-side callable analysis.
    pub fn insert_source(
        &mut self,
        callable: CallableId,
        analysis: Arc<CallableAnalysis>,
    ) -> Result<Option<Arc<CallableAnalysis>>, FrozenCallableAnalysisTableMutation> {
        let CallableAnalysisTableStorage::Source(source) = &mut self.storage else {
            return Err(FrozenCallableAnalysisTableMutation);
        };
        source.by_module.entry(callable.owner.module().clone()).or_default().insert(callable.clone());
        Ok(source.by_id.insert(callable, analysis))
    }

    /// Removes one source-side callable analysis.
    pub fn remove_source(&mut self, callable: &CallableId) -> Result<Option<Arc<CallableAnalysis>>, FrozenCallableAnalysisTableMutation> {
        let CallableAnalysisTableStorage::Source(source) = &mut self.storage else {
            return Err(FrozenCallableAnalysisTableMutation);
        };
        if let Some(ids) = source.by_module.get_mut(callable.owner.module()) {
            ids.remove(callable);
            if ids.is_empty() {
                source.by_module.remove(callable.owner.module());
            }
        }
        Ok(source.by_id.remove(callable))
    }

    /// Removes source-side entries that do not satisfy `predicate`.
    pub fn retain_source(
        &mut self,
        mut predicate: impl FnMut(&CallableId, &mut Arc<CallableAnalysis>) -> bool,
    ) -> Result<(), FrozenCallableAnalysisTableMutation> {
        let CallableAnalysisTableStorage::Source(source) = &mut self.storage else {
            return Err(FrozenCallableAnalysisTableMutation);
        };
        let by_module = &mut source.by_module;
        source.by_id.retain(|callable, analysis| {
            let keep = predicate(callable, analysis);
            if !keep {
                if let Some(ids) = by_module.get_mut(callable.owner.module()) {
                    ids.remove(callable);
                }
            }
            keep
        });
        source.by_module.retain(|_, ids| !ids.is_empty());
        Ok(())
    }

    /// Looks up a callable analysis by stable identity.
    pub fn get(&self, callable: &CallableId) -> Option<&Arc<CallableAnalysis>> {
        match &self.storage {
            CallableAnalysisTableStorage::Source(source) => source.by_id.get(callable),
            CallableAnalysisTableStorage::Frozen(frozen) => {
                let range = frozen.module_ranges.get(callable.owner.module())?;
                let rows = &frozen.rows[range.clone()];
                rows.binary_search_by(|(candidate, _)| candidate.cmp(callable)).ok().map(|index| &rows[index].1)
            }
        }
    }

    /// Returns whether the table contains `callable`.
    pub fn contains_key(&self, callable: &CallableId) -> bool {
        self.get(callable).is_some()
    }

    /// Returns the number of callable analyses.
    pub fn len(&self) -> usize {
        match &self.storage {
            CallableAnalysisTableStorage::Source(source) => source.by_id.len(),
            CallableAnalysisTableStorage::Frozen(frozen) => frozen.rows.len(),
        }
    }

    /// Returns whether the table has no callable analyses.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Iterates stable callable identities.
    pub fn keys(&self) -> impl ExactSizeIterator<Item = &CallableId> {
        self.iter().map(|(callable, _)| callable)
    }

    /// Iterates callable analysis owners.
    pub fn values(&self) -> impl ExactSizeIterator<Item = &Arc<CallableAnalysis>> {
        self.iter().map(|(_, analysis)| analysis)
    }

    /// Iterates all callable owners in the backing representation's order.
    pub fn iter(&self) -> impl ExactSizeIterator<Item = (&CallableId, &Arc<CallableAnalysis>)> {
        CallableAnalysisIter::new(&self.storage)
    }

    /// Iterates only this module's rows. Imported iteration is a direct slice
    /// traversal over its charged module range.
    pub fn iter_module<'a>(&'a self, module: &'a crate::identity::ModuleId) -> impl Iterator<Item = (&'a CallableId, &'a Arc<CallableAnalysis>)> {
        CallableAnalysisModuleIter::new(&self.storage, module)
    }
}

enum CallableAnalysisIter<'a> {
    Source(std::collections::hash_map::Iter<'a, CallableId, Arc<CallableAnalysis>>),
    Frozen(std::slice::Iter<'a, (CallableId, Arc<CallableAnalysis>)>),
}

impl<'a> CallableAnalysisIter<'a> {
    fn new(storage: &'a CallableAnalysisTableStorage) -> Self {
        match storage {
            CallableAnalysisTableStorage::Source(source) => Self::Source(source.by_id.iter()),
            CallableAnalysisTableStorage::Frozen(frozen) => Self::Frozen(frozen.rows.iter()),
        }
    }
}

impl<'a> Iterator for CallableAnalysisIter<'a> {
    type Item = (&'a CallableId, &'a Arc<CallableAnalysis>);

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Source(iter) => iter.next(),
            Self::Frozen(iter) => iter.next().map(|(callable, analysis)| (callable, analysis)),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len();
        (len, Some(len))
    }
}

impl ExactSizeIterator for CallableAnalysisIter<'_> {
    fn len(&self) -> usize {
        match self {
            Self::Source(iter) => iter.len(),
            Self::Frozen(iter) => iter.len(),
        }
    }
}

enum CallableAnalysisModuleIter<'a> {
    Source {
        ids: std::collections::btree_set::Iter<'a, CallableId>,
        by_id: &'a HashMap<CallableId, Arc<CallableAnalysis>>,
    },
    Frozen(std::slice::Iter<'a, (CallableId, Arc<CallableAnalysis>)>),
    Empty,
}

impl<'a> CallableAnalysisModuleIter<'a> {
    fn new(storage: &'a CallableAnalysisTableStorage, module: &crate::identity::ModuleId) -> Self {
        match storage {
            CallableAnalysisTableStorage::Source(source) => source
                .by_module
                .get(module)
                .map(|ids| Self::Source {
                    ids: ids.iter(),
                    by_id: &source.by_id,
                })
                .unwrap_or(Self::Empty),
            CallableAnalysisTableStorage::Frozen(frozen) => frozen
                .module_ranges
                .get(module)
                .map(|range| Self::Frozen(frozen.rows[range.clone()].iter()))
                .unwrap_or(Self::Empty),
        }
    }
}

impl<'a> Iterator for CallableAnalysisModuleIter<'a> {
    type Item = (&'a CallableId, &'a Arc<CallableAnalysis>);

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Source { ids, by_id } => ids.next().and_then(|callable| by_id.get(callable).map(|analysis| (callable, analysis))),
            Self::Frozen(iter) => iter.next().map(|(callable, analysis)| (callable, analysis)),
            Self::Empty => None,
        }
    }
}

impl std::ops::Index<&CallableId> for CallableAnalysisTable {
    type Output = Arc<CallableAnalysis>;

    fn index(&self, key: &CallableId) -> &Self::Output {
        self.get(key).expect("callable analysis is present in table")
    }
}

impl From<HashMap<CallableId, Arc<CallableAnalysis>>> for CallableAnalysisTable {
    fn from(map: std::collections::HashMap<CallableId, Arc<CallableAnalysis>>) -> Self {
        let mut by_module = HashMap::<crate::identity::ModuleId, BTreeSet<CallableId>>::new();
        for id in map.keys() {
            by_module.entry(id.owner.module().clone()).or_default().insert(id.clone());
        }
        Self {
            storage: CallableAnalysisTableStorage::Source(SourceCallableAnalysisTable { by_id: map, by_module }),
        }
    }
}

#[cfg(test)]
mod callable_analysis_table_tests {
    use super::{
        BodyExitFacts, CallableAnalysis, CallableAnalysisStatus, CallableAnalysisTable, CallableBindingOriginIndex, ExpressionAnalysisIndex, FlowStateSummary,
        SequenceLiteralResolutionIndex,
    };
    use crate::identity::{CallableId, CallableOwnerId, DeclarationId, DispatchSide, ModuleId, ModulePath, ResolvedProjectId};
    use crate::signature::ReturnContractValidation;
    use phalcom_common::range::SourceRange;
    use phalcom_common::selector::Selector;
    use phalcom_type_meta::allocation::{AllocationBudget, AllocationError};
    use std::collections::HashMap;
    use std::sync::Arc;

    fn module(project: u32) -> ModuleId {
        ModuleId::resolved(ResolvedProjectId::from_raw(project), ModulePath::root())
    }

    fn callable(module: ModuleId, name: &str) -> CallableId {
        CallableId::new(
            CallableOwnerId::Declaration(DeclarationId::new(module, "CallableTableTest".into())),
            Selector::getter(name).expect("test selector is valid"),
            DispatchSide::Instance,
        )
    }

    fn analysis(callable: CallableId) -> Arc<CallableAnalysis> {
        Arc::new(CallableAnalysis {
            callable,
            body_range: SourceRange { start: 0, end: 0 },
            body_source_owner: None,
            expressions: ExpressionAnalysisIndex::new(),
            bindings: super::BindingAnalysisIndex::new(),
            binding_origins: CallableBindingOriginIndex::new(),
            for_iteration_lanes: Arc::new(super::ForIterationLaneResolutionIndex::new()),
            positional_expansions: Arc::new(super::PositionalExpansionIterationResolutionIndex::new()),
            known_invalid_positional_expansions: Arc::new(super::KnownInvalidPositionalExpansionIndex::new()),
            sequence_literals: Arc::new(SequenceLiteralResolutionIndex::new()),
            associated_resolutions: Arc::new(crate::checker::associated::AssociatedResolutionIndex::new()),
            callable_reference_resolutions: Arc::new(crate::checker::associated::CallableReferenceResolutionIndex::new()),
            family_applications: Arc::new(crate::checker::associated::FamilyApplicationResolutionIndex::new()),
            match_resolutions: Arc::new(Default::default()),
            pattern_site_resolutions: Arc::new(Default::default()),
            local_proof_graph: Arc::new(Default::default()),
            flow_graph: Arc::new(crate::checker::flow::graph::FlowGraph::default()),
            entry_flow: FlowStateSummary::default(),
            exits: BodyExitFacts::default(),
            return_validation: ReturnContractValidation::NotApplicable,
            diagnostics: Arc::from([]),
            internal_incidents: Arc::from([]),
            explanations: Arc::new(crate::explain::ExplanationArena::new()),
            return_explanation: None,
            dependencies: Arc::from([]),
            semantic_dependencies: Arc::from([]),
            constructor_super_initializer: None,
            dependency_fingerprint: crate::db::ProductFingerprint(0),
            status: CallableAnalysisStatus::Complete,
        })
    }

    fn table_rows() -> Vec<(CallableId, Arc<CallableAnalysis>)> {
        [callable(module(2), "z"), callable(module(1), "b"), callable(module(1), "a")]
            .into_iter()
            .map(|id| (id.clone(), analysis(id)))
            .collect()
    }

    #[test]
    fn callable_proof_fingerprint_preserves_local_branch_authority() {
        use crate::identity::{BodyId, ExpressionId, LocalExpressionId};
        use crate::match_semantics::{
            BranchProofEnvironment, ExhaustivenessResult, MatchArmResolution, MatchResolution, PatternResolution, PatternSpaceSummary, PatternUsefulness,
        };
        use crate::types::{LocalType, TypeId, TypeKnowledge, TypeParameterId, UnknownReason};

        let mut original = (*analysis(callable(module(1), "proof"))).clone();
        let expression = ExpressionId::new(BodyId(0), LocalExpressionId(0));
        let knowledge = TypeKnowledge::Unknown(UnknownReason::NoTypeEvidence);
        let resolution = MatchResolution {
            expression,
            scrutinee: knowledge.clone(),
            initial_space: PatternSpaceSummary::Empty,
            arms: vec![MatchArmResolution {
                arm_index: 0,
                pattern: PatternResolution::Wildcard,
                reachable_space: PatternSpaceSummary::Empty,
                residual_after: PatternSpaceSummary::Empty,
                bindings: Box::new([]),
                proof: BranchProofEnvironment {
                    local_bindings: [(TypeParameterId(0), LocalType::Canonical(TypeId(1)))].into(),
                    ..Default::default()
                },
                usefulness: PatternUsefulness::Useful,
                branch_result: knowledge.clone(),
            }]
            .into_boxed_slice(),
            result: knowledge,
            exhaustiveness: ExhaustivenessResult::Proven,
        };
        original.match_resolutions = Arc::new([(expression, resolution.clone())].into());
        let mut changed = original.clone();
        let mut changed_resolution = resolution;
        changed_resolution.arms[0].proof.local_bindings = [(TypeParameterId(0), LocalType::Canonical(TypeId(2)))].into();
        changed.match_resolutions = Arc::new([(expression, changed_resolution)].into());
        assert_ne!(
            crate::db::fingerprint::callable_body_product_fingerprint(&original),
            crate::db::fingerprint::callable_body_product_fingerprint(&changed),
            "retained local proof authority must participate in the callable fingerprint"
        );
    }

    #[test]
    fn source_and_frozen_callable_tables_have_lookup_and_module_iteration_parity() {
        let rows = table_rows();
        let mut source_map = HashMap::new();
        for (callable, analysis) in rows.iter().cloned() {
            source_map.insert(callable, analysis);
        }
        let source = CallableAnalysisTable::from(source_map);
        let mut allocations = AllocationBudget::new(4096);
        let mut frozen_rows = allocations.vector(rows.len()).unwrap();
        frozen_rows.extend(rows);
        let frozen = CallableAnalysisTable::from_budgeted_entries(frozen_rows, &mut allocations).unwrap();

        assert_eq!(source.len(), 3);
        assert_eq!(frozen.len(), 3);
        for callable in source.keys() {
            assert!(frozen.contains_key(callable));
            assert_eq!(source.get(callable).unwrap().callable, frozen.get(callable).unwrap().callable);
        }
        for module in [module(1), module(2), module(3)] {
            let source_ids = source.iter_module(&module).map(|(id, _)| id.clone()).collect::<Vec<_>>();
            let frozen_ids = frozen.iter_module(&module).map(|(id, _)| id.clone()).collect::<Vec<_>>();
            assert_eq!(source_ids, frozen_ids);
            assert_eq!(frozen_ids.iter().filter(|id| id.owner.module() == &module).count(), frozen_ids.len());
        }
    }

    #[test]
    fn frozen_callable_table_rejects_source_mutation_and_source_clone_isolated() {
        let rows = table_rows();
        let mut allocations = AllocationBudget::new(4096);
        let mut frozen_rows = allocations.vector(rows.len()).unwrap();
        frozen_rows.extend(rows.clone());
        let mut frozen = CallableAnalysisTable::from_budgeted_entries(frozen_rows, &mut allocations).unwrap();
        let candidate = rows[0].0.clone();
        assert!(matches!(
            frozen.insert_source(candidate.clone(), analysis(candidate.clone())),
            Err(super::FrozenCallableAnalysisTableMutation)
        ));
        assert!(matches!(frozen.remove_source(&candidate), Err(super::FrozenCallableAnalysisTableMutation)));
        assert_eq!(frozen.retain_source(|_, _| true), Err(super::FrozenCallableAnalysisTableMutation));

        let mut source = CallableAnalysisTable::from(rows.into_iter().collect::<HashMap<_, _>>());
        let mut clone = source.clone();
        assert!(source.remove_source(&candidate).unwrap().is_some());
        assert!(!source.contains_key(&candidate));
        assert!(clone.contains_key(&candidate));
        assert!(clone.remove_source(&candidate).unwrap().is_some());
    }

    #[test]
    fn frozen_callable_table_charges_rows_and_module_index_at_exact_fit() {
        fn build(budget: &mut AllocationBudget) -> Result<CallableAnalysisTable, super::CallableAnalysisTableBuildError> {
            let mut rows = budget.vector(3).map_err(super::CallableAnalysisTableBuildError::Allocation)?;
            rows.extend(table_rows());
            CallableAnalysisTable::from_budgeted_entries(rows, budget)
        }

        let mut probe = AllocationBudget::new(16 * 1024);
        let _table = build(&mut probe).unwrap();
        let exact = probe.requested();
        assert!(exact > 0);

        let mut fits = AllocationBudget::new(exact);
        assert_eq!(build(&mut fits).unwrap().len(), 3);
        assert_eq!(fits.requested(), exact);

        let mut short = AllocationBudget::new(exact - 1);
        assert!(matches!(
            build(&mut short),
            Err(super::CallableAnalysisTableBuildError::Allocation(AllocationError::Exhausted))
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::checker::causal::CausalInvalidity;
    use crate::identity::DiagnosticCauseId;
    use crate::types::evidence::{EvidenceOrigin, TypeKnowledge, UnknownReason};
    use crate::types::id::TypeId;

    #[test]
    fn invalid_normal_return_fact_suppresses_recovery_knowledge_for_publication() {
        let fact = NormalReturnFact {
            knowledge: TypeKnowledge::established(TypeId(1), EvidenceOrigin::Flow),
            flow: FlowStateSummary::default(),
            status: AnalysisStatus::Invalid(DiagnosticCauseId(1)),
            causal_invalidity: CausalInvalidity::One(DiagnosticCauseId(1)),
            direct_result_dependency: None,
        };
        assert_eq!(fact.publication_knowledge(), TypeKnowledge::Unknown(UnknownReason::SuppressedByInvalidCause));
    }

    #[test]
    fn clean_normal_return_fact_preserves_knowledge_for_publication() {
        let fact = NormalReturnFact {
            knowledge: TypeKnowledge::established(TypeId(1), EvidenceOrigin::Flow),
            flow: FlowStateSummary::default(),
            status: AnalysisStatus::Ready,
            causal_invalidity: CausalInvalidity::Clean,
            direct_result_dependency: None,
        };
        assert_eq!(fact.publication_knowledge(), TypeKnowledge::established(TypeId(1), EvidenceOrigin::Flow));
    }
}
