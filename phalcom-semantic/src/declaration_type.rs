//! Range-free declaration type facts.
//!
//! These values describe declaration-owned static type state. They deliberately
//! exclude source ranges so semantic products can survive presentation-only
//! movement; current source provenance is joined through canonical identity in
//! `source_index`.

use crate::types::evidence::{DynamicReason, EvidenceOrigin, TypeKnowledge, UnknownReason};
use crate::types::parameter::TypeTerm;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum DeclaredTypeState {
    Known(TypeTerm),
    Dynamic(DynamicReason),
    Unknown(UnknownReason),
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum DeclaredTypeBasis {
    Unspecified,
    SourceAnnotation,
    NativeSignature,
    DeclarationSemantics,
    ConstructorSemantics,
    InitializerInference,
    BodyInference,
    ContextualTyping,
    PatternDecomposition,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct DeclaredTypeFact {
    pub state: DeclaredTypeState,
    pub basis: DeclaredTypeBasis,
}

impl DeclaredTypeFact {
    pub fn unknown(reason: UnknownReason) -> Self {
        Self {
            state: DeclaredTypeState::Unknown(reason),
            basis: DeclaredTypeBasis::Unspecified,
        }
    }

    pub fn from_knowledge(knowledge: &TypeKnowledge) -> Self {
        let basis = match knowledge.origin() {
            Some(EvidenceOrigin::DeveloperAnnotation) => DeclaredTypeBasis::SourceAnnotation,
            Some(EvidenceOrigin::NativeSignature) => DeclaredTypeBasis::NativeSignature,
            Some(EvidenceOrigin::DeclarationSemantics) => DeclaredTypeBasis::DeclarationSemantics,
            Some(EvidenceOrigin::ConstructorSemantics) => DeclaredTypeBasis::ConstructorSemantics,
            _ => DeclaredTypeBasis::Unspecified,
        };
        Self::from_knowledge_with_basis(knowledge, basis)
    }

    pub fn from_knowledge_with_basis(knowledge: &TypeKnowledge, basis: DeclaredTypeBasis) -> Self {
        let state = match knowledge {
            TypeKnowledge::Known(evidence) => DeclaredTypeState::Known(TypeTerm::Canonical(evidence.ty())),
            TypeKnowledge::Dynamic(reason) => DeclaredTypeState::Dynamic(reason.clone()),
            TypeKnowledge::Unknown(reason) => DeclaredTypeState::Unknown(reason.clone()),
        };
        Self { state, basis }
    }

    pub fn known(term: TypeTerm, basis: DeclaredTypeBasis) -> Self {
        Self {
            state: DeclaredTypeState::Known(term),
            basis,
        }
    }

    pub fn known_term(&self) -> Option<&TypeTerm> {
        match &self.state {
            DeclaredTypeState::Known(term) => Some(term),
            DeclaredTypeState::Dynamic(_) | DeclaredTypeState::Unknown(_) => None,
        }
    }

    pub fn canonical_type(&self) -> Option<crate::types::id::TypeId> {
        match &self.state {
            DeclaredTypeState::Known(TypeTerm::Canonical(ty)) => Some(*ty),
            _ => None,
        }
    }

    pub fn is_known(&self) -> bool {
        matches!(self.state, DeclaredTypeState::Known(_))
    }

    pub fn is_unknown(&self) -> bool {
        matches!(self.state, DeclaredTypeState::Unknown(_))
    }

    pub fn is_dynamic(&self) -> bool {
        matches!(self.state, DeclaredTypeState::Dynamic(_))
    }

    /// Projects a declaration fact into the declaration-level formal knowledge
    /// used by dispatch/checker consumers. This is a one-way projection.
    pub fn to_knowledge(&self) -> TypeKnowledge {
        match &self.state {
            DeclaredTypeState::Known(TypeTerm::Canonical(ty)) => match self.basis {
                DeclaredTypeBasis::SourceAnnotation => TypeKnowledge::assumed(*ty, EvidenceOrigin::DeveloperAnnotation),
                DeclaredTypeBasis::NativeSignature => TypeKnowledge::established(*ty, EvidenceOrigin::NativeSignature),
                DeclaredTypeBasis::DeclarationSemantics => TypeKnowledge::established(*ty, EvidenceOrigin::DeclarationSemantics),
                DeclaredTypeBasis::ConstructorSemantics => TypeKnowledge::established(*ty, EvidenceOrigin::ConstructorSemantics),
                DeclaredTypeBasis::BodyInference | DeclaredTypeBasis::InitializerInference => TypeKnowledge::established(*ty, EvidenceOrigin::Flow),
                DeclaredTypeBasis::ContextualTyping => TypeKnowledge::assumed(*ty, EvidenceOrigin::ContextualDerivation),
                DeclaredTypeBasis::PatternDecomposition => TypeKnowledge::established(*ty, EvidenceOrigin::PatternDecomposition),
                DeclaredTypeBasis::Unspecified => TypeKnowledge::assumed(*ty, EvidenceOrigin::CallableSignature),
            },
            DeclaredTypeState::Known(TypeTerm::SelfType(_)) | DeclaredTypeState::Known(TypeTerm::Infer(_)) => {
                TypeKnowledge::Unknown(UnknownReason::UnderconstrainedTypeVariable)
            }
            DeclaredTypeState::Dynamic(reason) => TypeKnowledge::Dynamic(reason.clone()),
            DeclaredTypeState::Unknown(reason) => TypeKnowledge::Unknown(reason.clone()),
        }
    }
}
