//! Semantic denotation facts and value semantic facts.

use super::evidence::TypeKnowledge;
use super::family::FamilyOperationShape;
use super::id::{KindId, TypeId};
use crate::associated::AssociatedMemberId;
use crate::identity::{AssociatedFamilyId, DeclarationId, InvocationTargetId};
use std::sync::Arc;

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CapturedAssociatedMember {
    pub operation: FamilyOperationShape,
    pub member: AssociatedMemberId,
    pub target: Option<InvocationTargetId>,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum AssociatedValueDenotation {
    Exact {
        owner_form: TypeId,
        lookup_owner: DeclarationId,
        member: AssociatedMemberId,
        target: Box<Option<InvocationTargetId>>,
    },
    Family {
        owner_form: TypeId,
        lookup_owner: DeclarationId,
        family: AssociatedFamilyId,
        members: Arc<[CapturedAssociatedMember]>,
    },
}

impl AssociatedValueDenotation {
    pub fn exact(owner_form: TypeId, lookup_owner: DeclarationId, member: AssociatedMemberId, target: Option<InvocationTargetId>) -> Self {
        Self::Exact {
            owner_form,
            lookup_owner,
            member,
            target: Box::new(target),
        }
    }

    pub fn family(owner_form: TypeId, lookup_owner: DeclarationId, family: AssociatedFamilyId, mut members: Vec<CapturedAssociatedMember>) -> Self {
        members.sort();
        Self::Family {
            owner_form,
            lookup_owner,
            family,
            members: members.into(),
        }
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum SemanticDenotation {
    TypeForm(TypeId),
    Kind(KindId),
    AssociatedValue(Box<AssociatedValueDenotation>),
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
        let denotation = match (&left.denotation, &right.denotation) {
            (Some(d1), Some(d2)) if d1 == d2 => Some(d1.clone()),
            _ => None,
        };
        Self {
            knowledge: merged_knowledge,
            denotation,
        }
    }
}
