//! Formal expression and callable analysis product models (Spec 04.5).

use crate::checker::causal::CausalInvalidity;
use crate::checker::incident::InternalSemanticIncident;
use crate::diagnostic::SemanticDiagnostic;
use crate::identity::{
    AnalysisIncidentId, BindingId, CallResolutionId, CallableId, CallableParameterId, DiagnosticCauseId, ExplanationId, ExpressionId, FieldId,
};
use crate::types::denotation::SemanticDenotation;
use crate::types::evidence::{DynamicReason, TypeKnowledge};
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
    pub denotation: Option<SemanticDenotation>,
    pub status: AnalysisStatus,
    pub causal_invalidity: CausalInvalidity,
    pub explanation: Option<ExplanationId>,
    pub call: Option<CallResolutionId>,
}

impl ExpressionAnalysis {
    pub fn ready(id: ExpressionId, range: SourceRange, knowledge: TypeKnowledge) -> Self {
        Self {
            id,
            range,
            knowledge,
            callable: None,
            denotation: None,
            status: AnalysisStatus::Ready,
            causal_invalidity: CausalInvalidity::Clean,
            explanation: None,
            call: None,
        }
    }

    pub fn invalid(id: ExpressionId, range: SourceRange, cause: DiagnosticCauseId) -> Self {
        Self {
            id,
            range,
            knowledge: TypeKnowledge::Unknown(crate::types::evidence::UnknownReason::SyntaxError),
            callable: None,
            denotation: None,
            status: AnalysisStatus::Invalid(cause),
            causal_invalidity: CausalInvalidity::One(cause),
            explanation: None,
            call: None,
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
        Self::new_with_contract(binding, name, range, contract, current, None, mutable)
    }

    pub fn new_with_contract(
        binding: BindingId,
        name: impl Into<String>,
        range: SourceRange,
        contract: Option<super::binding::BindingContract>,
        current: TypeKnowledge,
        denotation: Option<SemanticDenotation>,
        mutable: bool,
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
    CallableSignature(CallableId),
    FieldSignature(FieldId),
    DeclarationSurface(DeclarationId),
    HierarchyEdge(DeclarationId),
    LinkedInterface(ModuleId),
    EnumDeclaration(DeclarationId),
    AssociatedSurface(DeclarationId),
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

/// Published semantic analysis product for a single callable body.
#[derive(Clone, Debug)]
pub struct CallableAnalysis {
    pub callable: CallableId,
    pub body_range: SourceRange,
    pub expressions: ExpressionAnalysisIndex,
    pub bindings: BindingAnalysisIndex,
    pub associated_resolutions: Arc<crate::checker::associated::AssociatedResolutionIndex>,
    pub family_applications: Arc<crate::checker::associated::FamilyApplicationResolutionIndex>,
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
        };
        assert_eq!(fact.publication_knowledge(), TypeKnowledge::established(TypeId(1), EvidenceOrigin::Flow));
    }
}
