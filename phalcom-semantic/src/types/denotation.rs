//! Semantic denotation facts and value semantic facts.

use super::evidence::TypeKnowledge;
use super::id::{KindId, TypeId};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SemanticDenotation {
    TypeForm(TypeId),
    Kind(KindId),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValueSemanticFact {
    pub knowledge: TypeKnowledge,
    pub denotation: Option<SemanticDenotation>,
}

impl ValueSemanticFact {
    pub fn new(knowledge: TypeKnowledge) -> Self {
        Self { knowledge, denotation: None }
    }

    pub fn with_denotation(mut self, denotation: SemanticDenotation) -> Self {
        self.denotation = Some(denotation);
        self
    }

    pub fn merge(left: &Self, right: &Self, merged_knowledge: TypeKnowledge) -> Self {
        let denotation = match (left.denotation, right.denotation) {
            (Some(d1), Some(d2)) if d1 == d2 => Some(d1),
            _ => None,
        };
        Self {
            knowledge: merged_knowledge,
            denotation,
        }
    }
}
