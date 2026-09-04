//! Semantic typing result representation for expressions.

use crate::checker::analysis::AnalysisStatus;
use crate::checker::causal::CausalInvalidity;
use crate::checker::inference::{InferenceContextId, InferenceTerm};
use crate::dispatch::DispatchLookup;
use crate::identity::{CallableId, DiagnosticCauseId, ExplanationId, ExpressionId};
use crate::types::constraint::TypeConstraint;
use crate::types::denotation::{SemanticDenotation, ValueSemanticFact};
use crate::types::evidence::{EvidenceOrigin, EvidenceSet, TypeKnowledge, UnknownReason};
use crate::types::id::TypeId;
use crate::types::rigid::LocalType;
use phalcom_common::range::SourceRange;

/// Solver-local result retained while one expression is checked inside a live
/// generic application. It never enters durable expression analysis products.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct SymbolicInferenceResult {
    pub context: InferenceContextId,
    pub term: InferenceTerm,
}

/// Semantic typing result for an expression with type knowledge, constraints, and provenance.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TypedExpression {
    pub expression_id: Option<ExpressionId>,
    pub callable: Option<CallableId>,
    pub explanation_parents: Vec<ExplanationId>,
    pub knowledge: TypeKnowledge,
    pub status: AnalysisStatus,
    pub denotation: Option<SemanticDenotation>,
    pub dispatch_lookup: DispatchLookup,
    pub constraints: Vec<TypeConstraint>,
    pub provenance: EvidenceSet,
    pub causal_invalidity: CausalInvalidity,
    /// Query-local type view. Never copied into durable expression metadata.
    pub(crate) local_type: Option<LocalType>,
    pub(crate) symbolic_result: Option<SymbolicInferenceResult>,
}

impl TypedExpression {
    /// Marks this expression as owning `cause` while keeping independently
    /// available type knowledge intact.
    pub(crate) fn invalidate(&mut self, cause: DiagnosticCauseId) {
        self.status = AnalysisStatus::Invalid(cause);
        self.causal_invalidity = self.causal_invalidity.join(CausalInvalidity::One(cause));
    }

    pub(crate) fn debug_assert_coherent(&self) {
        if let AnalysisStatus::Invalid(cause) = self.status {
            debug_assert!(
                self.causal_invalidity.contains(cause),
                "Invalid expression status must include its owning diagnostic cause"
            );
        }

        if matches!(self.status, AnalysisStatus::Suppressed(_)) {
            debug_assert!(
                !matches!(self.causal_invalidity, CausalInvalidity::Clean),
                "Suppressed expression must have non-clean causal invalidity"
            );
        }
    }

    pub fn new(knowledge: TypeKnowledge) -> Self {
        let provenance = match &knowledge {
            TypeKnowledge::Known(ev) => ev.provenance().clone(),
            _ => EvidenceSet::default(),
        };
        Self {
            expression_id: None,
            callable: None,
            explanation_parents: Vec::new(),
            knowledge: knowledge.clone(),
            status: match &knowledge {
                TypeKnowledge::Dynamic(reason) => AnalysisStatus::DynamicBoundary(reason.clone()),
                _ => AnalysisStatus::Ready,
            },
            denotation: None,
            dispatch_lookup: DispatchLookup::Normal,
            constraints: Vec::new(),
            provenance,
            causal_invalidity: CausalInvalidity::Clean,
            local_type: None,
            symbolic_result: None,
        }
    }

    pub fn established(ty: TypeId, origin: EvidenceOrigin, range: SourceRange) -> Self {
        let knowledge = TypeKnowledge::established(ty, origin).with_range(range);
        let mut provenance = EvidenceSet::default();
        provenance.ranges.push(range);
        Self {
            expression_id: None,
            callable: None,
            explanation_parents: Vec::new(),
            knowledge,
            status: AnalysisStatus::Ready,
            denotation: None,
            dispatch_lookup: DispatchLookup::Normal,
            constraints: Vec::new(),
            provenance,
            causal_invalidity: CausalInvalidity::Clean,
            local_type: None,
            symbolic_result: None,
        }
    }

    pub fn assumed(ty: TypeId, origin: EvidenceOrigin, range: SourceRange) -> Self {
        let knowledge = TypeKnowledge::assumed(ty, origin).with_range(range);
        let mut provenance = EvidenceSet::default();
        provenance.ranges.push(range);
        Self {
            expression_id: None,
            callable: None,
            explanation_parents: Vec::new(),
            knowledge,
            status: AnalysisStatus::Ready,
            denotation: None,
            dispatch_lookup: DispatchLookup::Normal,
            constraints: Vec::new(),
            provenance,
            causal_invalidity: CausalInvalidity::Clean,
            local_type: None,
            symbolic_result: None,
        }
    }

    pub fn unknown(reason: UnknownReason) -> Self {
        Self {
            expression_id: None,
            callable: None,
            explanation_parents: Vec::new(),
            knowledge: TypeKnowledge::Unknown(reason),
            status: AnalysisStatus::Ready,
            denotation: None,
            dispatch_lookup: DispatchLookup::Normal,
            constraints: Vec::new(),
            provenance: EvidenceSet::default(),
            causal_invalidity: CausalInvalidity::Clean,
            local_type: None,
            symbolic_result: None,
        }
    }

    pub fn dynamic(reason: crate::types::evidence::DynamicReason) -> Self {
        Self {
            expression_id: None,
            callable: None,
            explanation_parents: Vec::new(),
            knowledge: TypeKnowledge::Dynamic(reason.clone()),
            status: AnalysisStatus::DynamicBoundary(reason),
            denotation: None,
            dispatch_lookup: DispatchLookup::Normal,
            constraints: Vec::new(),
            provenance: EvidenceSet::default(),
            causal_invalidity: CausalInvalidity::Clean,
            local_type: None,
            symbolic_result: None,
        }
    }

    pub fn with_denotation(mut self, denotation: SemanticDenotation) -> Self {
        self.denotation = Some(denotation);
        self
    }

    pub(crate) fn with_symbolic_result(mut self, result: SymbolicInferenceResult) -> Self {
        self.symbolic_result = Some(result);
        self
    }

    pub fn with_status(mut self, status: AnalysisStatus) -> Self {
        self.status = status;
        self
    }

    pub fn with_dispatch_lookup(mut self, lookup: DispatchLookup) -> Self {
        self.dispatch_lookup = lookup;
        self
    }

    pub fn fact(&self) -> ValueSemanticFact {
        ValueSemanticFact {
            knowledge: self.knowledge.clone(),
            denotation: self.denotation.clone(),
        }
    }

    pub fn with_constraint(mut self, constraint: TypeConstraint) -> Self {
        self.constraints.push(constraint);
        self
    }

    pub fn with_constraints(mut self, constraints: impl IntoIterator<Item = TypeConstraint>) -> Self {
        self.constraints.extend(constraints);
        self
    }

    pub fn ty(&self) -> Option<TypeId> {
        self.knowledge.ty()
    }
}

impl From<TypeKnowledge> for TypedExpression {
    fn from(knowledge: TypeKnowledge) -> Self {
        Self::new(knowledge)
    }
}

impl From<crate::checker::call::CallCheckResult> for TypedExpression {
    fn from(result: crate::checker::call::CallCheckResult) -> Self {
        let mut typed = Self::new(result.knowledge);
        typed.status = result.status;
        typed.causal_invalidity = result.causal_invalidity;
        typed.explanation_parents = result.explanation_parents;
        typed.callable = result.callable;
        typed.local_type = result.local_type;
        typed.symbolic_result = result.symbolic_result;
        typed.debug_assert_coherent();
        typed
    }
}

impl From<crate::checker::analysis::ExpressionAnalysis> for TypedExpression {
    fn from(analysis: crate::checker::analysis::ExpressionAnalysis) -> Self {
        let mut expr = Self::new(analysis.knowledge);
        expr.expression_id = Some(analysis.id);
        expr.callable = analysis.callable;
        expr.explanation_parents = analysis.explanation.into_iter().collect();
        expr.denotation = analysis.denotation;
        expr.status = analysis.status;
        expr.causal_invalidity = analysis.causal_invalidity;
        expr
    }
}
