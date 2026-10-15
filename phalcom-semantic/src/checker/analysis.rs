//! Formal expression and callable analysis product models (Spec 04.5).

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
use std::collections::BTreeMap;
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
pub type ForIterationLaneResolutionIndex = BTreeMap<ExpressionId, ForIterationLaneResolution>;

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
pub type PositionalExpansionIterationResolutionIndex = BTreeMap<ExpressionId, PositionalExpansionIterationResolution>;

/// A statically known positional expansion source for which canonical
/// Iterable/Iterator evidence was not available. Lowering consumes this
/// product to fail closed without re-solving the semantic question.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KnownInvalidPositionalExpansion {
    pub source_expression: ExpressionId,
    pub source_range: SourceRange,
    pub source_type: TypeId,
}

pub type KnownInvalidPositionalExpansionIndex = BTreeMap<ExpressionId, KnownInvalidPositionalExpansion>;

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
    pub bindings: BTreeMap<BindingId, FlowBindingSummary>,
    pub fields: BTreeMap<FieldId, FlowFieldSummary>,
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
pub type ExpressionAnalysisIndex = BTreeMap<ExpressionId, ExpressionAnalysis>;

/// Index of binding states within a body.
pub type BindingAnalysisIndex = BTreeMap<BindingId, BindingState>;

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
pub type CallableBindingOriginIndex = BTreeMap<BindingId, CallableBindingOrigin>;

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
    pub exact_expansions: BTreeMap<SourceRange, SequenceExpansionResolution>,
}

/// Proof-preserving positional spread for dimensioned sequence construction.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum SequenceExpansionResolution {
    /// Closed product positional lane; labeled entries are excluded.
    TuplePositionals { extent: TypeId },
    /// Native immutable Vector storage, independent from virtual iteration.
    VectorStorage { extent: TypeId },
}

pub type SequenceLiteralResolutionIndex = BTreeMap<ExpressionId, SequenceLiteralResolution>;

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
    pub match_resolutions: Arc<crate::match_semantics::MatchResolutionIndex>,
    pub pattern_site_resolutions: Arc<crate::match_semantics::PatternSiteResolutionIndex>,
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
}

/// Secondary module-indexed table of callable analyses.
/// Provides O(1) module-bounded iteration for compiler lowering while
/// preserving fast ID lookups and familiar map-like query methods.
#[derive(Clone, Debug, Default)]
pub struct CallableAnalysisTable {
    by_id: std::collections::HashMap<CallableId, Arc<CallableAnalysis>>,
    by_module: std::collections::HashMap<crate::identity::ModuleId, std::collections::BTreeSet<CallableId>>,
}

impl CallableAnalysisTable {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, callable: CallableId, analysis: Arc<CallableAnalysis>) {
        self.by_module.entry(callable.owner.module().clone()).or_default().insert(callable.clone());
        self.by_id.insert(callable, analysis);
    }

    pub fn remove(&mut self, callable: &CallableId) -> Option<Arc<CallableAnalysis>> {
        if let Some(set) = self.by_module.get_mut(callable.owner.module()) {
            set.remove(callable);
        }
        self.by_id.remove(callable)
    }

    pub fn retain(&mut self, mut predicate: impl FnMut(&CallableId, &mut Arc<CallableAnalysis>) -> bool) {
        let by_module = &mut self.by_module;
        self.by_id.retain(|callable, analysis| {
            let keep = predicate(callable, analysis);
            if !keep {
                if let Some(set) = by_module.get_mut(callable.owner.module()) {
                    set.remove(callable);
                }
            }
            keep
        });
    }

    pub fn get(&self, callable: &CallableId) -> Option<&Arc<CallableAnalysis>> {
        self.by_id.get(callable)
    }

    pub fn contains_key(&self, callable: &CallableId) -> bool {
        self.by_id.contains_key(callable)
    }

    pub fn len(&self) -> usize {
        self.by_id.len()
    }

    pub fn is_empty(&self) -> bool {
        self.by_id.is_empty()
    }

    pub fn keys(&self) -> std::collections::hash_map::Keys<'_, CallableId, Arc<CallableAnalysis>> {
        self.by_id.keys()
    }

    pub fn values(&self) -> std::collections::hash_map::Values<'_, CallableId, Arc<CallableAnalysis>> {
        self.by_id.values()
    }

    pub fn iter(&self) -> std::collections::hash_map::Iter<'_, CallableId, Arc<CallableAnalysis>> {
        self.by_id.iter()
    }

    /// Iterates only the callable analyses belonging to the specified module.
    /// This is an O(callables in module) operation instead of O(all callables).
    pub fn iter_module<'a>(&'a self, module: &'a crate::identity::ModuleId) -> impl Iterator<Item = (&'a CallableId, &'a Arc<CallableAnalysis>)> {
        self.by_module
            .get(module)
            .into_iter()
            .flat_map(move |ids| ids.iter().filter_map(move |id| self.by_id.get(id).map(|analysis| (id, analysis))))
    }
}

impl std::ops::Index<&CallableId> for CallableAnalysisTable {
    type Output = Arc<CallableAnalysis>;

    fn index(&self, key: &CallableId) -> &Self::Output {
        self.get(key).expect("callable analysis is present in table")
    }
}

impl std::ops::Deref for CallableAnalysisTable {
    type Target = std::collections::HashMap<CallableId, Arc<CallableAnalysis>>;

    fn deref(&self) -> &Self::Target {
        &self.by_id
    }
}

impl From<std::collections::HashMap<CallableId, Arc<CallableAnalysis>>> for CallableAnalysisTable {
    fn from(map: std::collections::HashMap<CallableId, Arc<CallableAnalysis>>) -> Self {
        let mut by_module = std::collections::HashMap::<crate::identity::ModuleId, std::collections::BTreeSet<CallableId>>::new();
        for id in map.keys() {
            by_module.entry(id.owner.module().clone()).or_default().insert(id.clone());
        }
        Self { by_id: map, by_module }
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
