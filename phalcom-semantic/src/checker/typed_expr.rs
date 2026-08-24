//! Semantic typing result representation for expressions.

use crate::checker::causal::CausalInvalidity;
use crate::dispatch::DispatchLookup;
use crate::identity::{CallableId, ExplanationId, ExpressionId};
use crate::types::constraint::TypeConstraint;
use crate::types::denotation::{SemanticDenotation, ValueSemanticFact};
use crate::types::evidence::{EvidenceOrigin, EvidenceSet, TypeKnowledge, UnknownReason};
use crate::types::id::TypeId;
use phalcom_common::range::SourceRange;

/// Semantic typing result for an expression with type knowledge, constraints, and provenance.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TypedExpression {
    pub expression_id: Option<ExpressionId>,
    pub callable: Option<CallableId>,
    pub explanation_parents: Vec<ExplanationId>,
    pub knowledge: TypeKnowledge,
    pub denotation: Option<SemanticDenotation>,
    pub dispatch_lookup: DispatchLookup,
    pub constraints: Vec<TypeConstraint>,
    pub provenance: EvidenceSet,
    pub causal_invalidity: CausalInvalidity,
}

impl TypedExpression {
    pub fn new(knowledge: TypeKnowledge) -> Self {
        let provenance = match &knowledge {
            TypeKnowledge::Known(ev) => ev.provenance.clone(),
            _ => EvidenceSet::default(),
        };
        Self {
            expression_id: None,
            callable: None,
            explanation_parents: Vec::new(),
            knowledge,
            denotation: None,
            dispatch_lookup: DispatchLookup::Normal,
            constraints: Vec::new(),
            provenance,
            causal_invalidity: CausalInvalidity::Clean,
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
            denotation: None,
            dispatch_lookup: DispatchLookup::Normal,
            constraints: Vec::new(),
            provenance,
            causal_invalidity: CausalInvalidity::Clean,
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
            denotation: None,
            dispatch_lookup: DispatchLookup::Normal,
            constraints: Vec::new(),
            provenance,
            causal_invalidity: CausalInvalidity::Clean,
        }
    }

    pub fn unknown(reason: UnknownReason) -> Self {
        Self {
            expression_id: None,
            callable: None,
            explanation_parents: Vec::new(),
            knowledge: TypeKnowledge::Unknown(reason),
            denotation: None,
            dispatch_lookup: DispatchLookup::Normal,
            constraints: Vec::new(),
            provenance: EvidenceSet::default(),
            causal_invalidity: CausalInvalidity::Clean,
        }
    }

    pub fn dynamic(reason: crate::types::evidence::DynamicReason) -> Self {
        Self {
            expression_id: None,
            callable: None,
            explanation_parents: Vec::new(),
            knowledge: TypeKnowledge::Dynamic(reason),
            denotation: None,
            dispatch_lookup: DispatchLookup::Normal,
            constraints: Vec::new(),
            provenance: EvidenceSet::default(),
            causal_invalidity: CausalInvalidity::Clean,
        }
    }

    pub fn with_denotation(mut self, denotation: SemanticDenotation) -> Self {
        self.denotation = Some(denotation);
        self
    }

    pub fn with_dispatch_lookup(mut self, lookup: DispatchLookup) -> Self {
        self.dispatch_lookup = lookup;
        self
    }

    pub fn fact(&self) -> ValueSemanticFact {
        ValueSemanticFact {
            knowledge: self.knowledge.clone(),
            denotation: self.denotation,
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

impl From<crate::checker::analysis::ExpressionAnalysis> for TypedExpression {
    fn from(analysis: crate::checker::analysis::ExpressionAnalysis) -> Self {
        let mut expr = Self::new(analysis.knowledge);
        expr.expression_id = Some(analysis.id);
        expr.callable = analysis.callable;
        expr.explanation_parents = analysis.explanation.into_iter().collect();
        expr.denotation = analysis.denotation;
        expr.causal_invalidity = analysis.causal_invalidity;
        expr
    }
}
