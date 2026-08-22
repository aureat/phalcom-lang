//! Semantic typing result representation for expressions.

use crate::types::constraint::TypeConstraint;
use crate::types::denotation::{SemanticDenotation, ValueSemanticFact};
use crate::types::evidence::{EvidenceAuthority, EvidenceSet, TypeKnowledge, UnknownReason};
use crate::types::id::TypeId;
use phalcom_common::range::SourceRange;

/// Semantic typing result for an expression with type knowledge, constraints, and provenance.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TypedExpression {
    pub knowledge: TypeKnowledge,
    pub denotation: Option<SemanticDenotation>,
    pub constraints: Vec<TypeConstraint>,
    pub provenance: EvidenceSet,
}

impl TypedExpression {
    pub fn new(knowledge: TypeKnowledge) -> Self {
        let provenance = match &knowledge {
            TypeKnowledge::Known(ev) => ev.provenance.clone(),
            _ => EvidenceSet::default(),
        };
        Self {
            knowledge,
            denotation: None,
            constraints: Vec::new(),
            provenance,
        }
    }

    pub fn known(ty: TypeId, authority: EvidenceAuthority, range: SourceRange) -> Self {
        let knowledge = TypeKnowledge::known(ty, authority).with_range(range);
        let mut provenance = EvidenceSet::default();
        provenance.ranges.push(range);
        Self {
            knowledge,
            denotation: None,
            constraints: Vec::new(),
            provenance,
        }
    }

    pub fn unknown(reason: UnknownReason) -> Self {
        Self {
            knowledge: TypeKnowledge::Unknown(reason),
            denotation: None,
            constraints: Vec::new(),
            provenance: EvidenceSet::default(),
        }
    }

    pub fn dynamic(reason: crate::types::evidence::DynamicReason) -> Self {
        Self {
            knowledge: TypeKnowledge::Dynamic(reason),
            denotation: None,
            constraints: Vec::new(),
            provenance: EvidenceSet::default(),
        }
    }

    pub fn with_denotation(mut self, denotation: SemanticDenotation) -> Self {
        self.denotation = Some(denotation);
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

    pub fn with_constraints(
        mut self,
        constraints: impl IntoIterator<Item = TypeConstraint>,
    ) -> Self {
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
