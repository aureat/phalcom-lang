//! Formal expression and callable analysis product models (Spec 04.5).

use crate::checker::causal::CausalInvalidity;
use crate::checker::incident::InternalSemanticIncident;
use crate::diagnostic::SemanticDiagnostic;
use crate::identity::{
    AnalysisIncidentId, BindingId, CallResolutionId, CallableId, CallableParameterId, DiagnosticCauseId, ExplanationId, ExpressionId, FieldId, ImplId,
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
}

/// Binding for one callee generic conformance obligation discharged during callable application.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GenericCallConformanceBinding {
    pub callee_conformance: crate::generic_evidence::GenericConformanceKey,
    pub source: GenericCallConformanceSource,
}

/// Solved generic arguments and discharged conformance bindings for one canonical callable application.
///
/// Bindings contain canonical `TypeId`s only. Inference-local variables never
/// cross this publication boundary.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CallSpecialization {
    pub callable: CallableId,
    pub bindings: Box<[(TypeParameterId, TypeId)]>,
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
    pub expressions: ExpressionAnalysisIndex,
    pub bindings: BindingAnalysisIndex,
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
