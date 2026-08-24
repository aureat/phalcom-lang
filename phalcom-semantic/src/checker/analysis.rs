//! Formal expression and callable analysis product models (Spec 04.5).

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
    pub denotation: Option<SemanticDenotation>,
    pub status: AnalysisStatus,
    pub explanation: Option<ExplanationId>,
    pub call: Option<CallResolutionId>,
}

impl ExpressionAnalysis {
    pub fn ready(id: ExpressionId, range: SourceRange, knowledge: TypeKnowledge) -> Self {
        Self {
            id,
            range,
            knowledge,
            denotation: None,
            status: AnalysisStatus::Ready,
            explanation: None,
            call: None,
        }
    }

    pub fn invalid(id: ExpressionId, range: SourceRange, cause: DiagnosticCauseId) -> Self {
        Self {
            id,
            range,
            knowledge: TypeKnowledge::Unknown(crate::types::evidence::UnknownReason::SyntaxError),
            denotation: None,
            status: AnalysisStatus::Invalid(cause),
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
    pub current: TypeKnowledge,
    pub mutable: bool,
    pub version: u32,
    pub explanation: Option<ExplanationId>,
}

impl BindingState {
    pub fn new(
        binding: BindingId,
        name: impl Into<String>,
        range: SourceRange,
        declared: Option<crate::types::id::TypeId>,
        current: TypeKnowledge,
        mutable: bool,
    ) -> Self {
        Self {
            binding,
            name: name.into(),
            range,
            declared,
            current,
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
    pub throws: Vec<FlowStateSummary>,
    pub unreachable: bool,
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
    pub dependency_fingerprint: crate::db::ProductFingerprint,
    pub status: CallableAnalysisStatus,
}
