//! Formal expression and callable analysis product models (Spec 04.5).

use crate::checker::causal::CausalInvalidity;
use crate::diagnostic::SemanticDiagnostic;
use crate::identity::{AnalysisIncidentId, BindingId, CallResolutionId, CallableId, DiagnosticCauseId, ExplanationId, ExpressionId};
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
    pub range: SourceRange,
    pub declared: Option<crate::types::id::TypeId>,
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
    pub fn from_seed(binding: BindingId, seed: super::binding::BindingSeed, current: TypeKnowledge, consistency: super::binding::BindingConsistency) -> Self {
        let declared = seed.contract.as_ref().map(|contract| contract.ty);
        Self {
            binding,
            name: seed.name,
            range: seed.range,
            declared,
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
        Self::new_with_contract(binding, name, range, declared, contract, current, None, mutable)
    }

    pub fn new_with_contract(
        binding: BindingId,
        name: impl Into<String>,
        range: SourceRange,
        declared: Option<crate::types::id::TypeId>,
        contract: Option<super::binding::BindingContract>,
        current: TypeKnowledge,
        denotation: Option<SemanticDenotation>,
        mutable: bool,
    ) -> Self {
        Self {
            binding,
            name: name.into(),
            range,
            declared,
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
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct FlowStateSummary {
    pub known_bindings: BTreeMap<BindingId, crate::types::id::TypeId>,
    pub fact_count: usize,
}

/// Recorded exit facts across returns, throws, and unreachable points.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct BodyExitFacts {
    pub returns: Vec<FlowStateSummary>,
    /// Type knowledge produced by normal callable exits. An empty vector
    /// means no normal return path exists (`Never`); abrupt exits are not
    /// values and therefore do not participate in this collection.
    pub normal_return_values: Vec<TypeKnowledge>,
    pub throws: Vec<FlowStateSummary>,
    pub unreachable: bool,
}

/// Joins values from all normal callable exits into one published return
/// knowledge fact. Abrupt-only bodies produce `Never`; incomplete knowledge
/// keeps the summary incomplete instead of inventing a return type.
pub fn normal_return_summary(store: &mut crate::types::store::TypeStore, values: &[TypeKnowledge]) -> TypeKnowledge {
    use crate::types::evidence::{EvidenceOrigin, TypeKnowledge, UnknownReason, join_type_knowledge};

    if values.is_empty() {
        return TypeKnowledge::established(store.never(), EvidenceOrigin::Flow);
    }

    let joined = join_type_knowledge(store, values.iter().cloned());
    if joined.ty().is_none() {
        TypeKnowledge::Unknown(UnknownReason::UncheckedExpression)
    } else {
        joined
    }
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
    DeclarationSurface(DeclarationId),
    HierarchyEdge(DeclarationId),
    LinkedInterface(ModuleId),
}

/// Status of callable-body analysis.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum CallableAnalysisStatus {
    Complete,
    Partial,
    Blocked,
    Cancelled,
    BudgetExceeded,
}

/// Published semantic analysis product for a single callable body.
#[derive(Clone, Debug)]
pub struct CallableAnalysis {
    pub callable: CallableId,
    pub body_range: SourceRange,
    pub expressions: ExpressionAnalysisIndex,
    pub bindings: BindingAnalysisIndex,
    pub flow_graph: Arc<crate::checker::flow::graph::FlowGraph>,
    pub entry_flow: FlowStateSummary,
    pub exits: BodyExitFacts,
    pub diagnostics: Arc<[SemanticDiagnostic]>,
    pub explanations: Arc<crate::explain::ExplanationArena>,
    pub dependencies: Arc<[CallableId]>,
    pub semantic_dependencies: Arc<[SemanticDependency]>,
    /// Semantic result fingerprint used to decide whether downstream queries
    /// must propagate a body refresh. Despite its legacy field name, this is
    /// not a hash of dependency-edge metadata.
    pub dependency_fingerprint: crate::db::ProductFingerprint,
    pub status: CallableAnalysisStatus,
}
