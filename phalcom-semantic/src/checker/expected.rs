//! Bidirectional checking type expectations (Spec 04.5).

use crate::checker::inference::InferenceTerm;
use crate::types::id::TypeId;

/// Why a checker expectation is being propagated downward. This is
/// contextual control information, never value evidence.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ExpectationOrigin {
    DeclarationContract,
    ReturnContract,
    CallableSignature,
    AssignmentContract,
    ContextualBlockParameter,
    GenericArgument,
    GenericResult,
    CollectionElement,
    ProductComponent,
    ExplicitCheck,
}

/// Contextual type expectation propagated downward into an expression during bidirectional checking.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub enum ExpectedType {
    /// No contextual expectation (synthesis mode).
    #[default]
    None,
    /// A known canonical proper type expectation (checking mode).
    Proper { ty: TypeId, origin: ExpectationOrigin },
    /// A solver-local inference term expectation.
    Inference { term: InferenceTerm, origin: ExpectationOrigin },
}

impl ExpectedType {
    pub fn none() -> Self {
        Self::None
    }

    pub fn proper(ty: TypeId) -> Self {
        Self::Proper {
            ty,
            origin: ExpectationOrigin::ExplicitCheck,
        }
    }

    pub fn proper_from(ty: TypeId, origin: ExpectationOrigin) -> Self {
        Self::Proper { ty, origin }
    }

    pub fn inference(term: InferenceTerm) -> Self {
        Self::Inference {
            term,
            origin: ExpectationOrigin::GenericArgument,
        }
    }

    pub fn inference_from(term: InferenceTerm, origin: ExpectationOrigin) -> Self {
        Self::Inference { term, origin }
    }

    pub fn is_none(&self) -> bool {
        matches!(self, Self::None)
    }

    pub fn ty(&self) -> Option<TypeId> {
        match self {
            Self::Proper { ty, .. } => Some(*ty),
            Self::Inference {
                term: InferenceTerm::Canonical(ty),
                ..
            } => Some(*ty),
            _ => None,
        }
    }

    pub fn origin(&self) -> Option<ExpectationOrigin> {
        match self {
            Self::None => None,
            Self::Proper { origin, .. } | Self::Inference { origin, .. } => Some(*origin),
        }
    }

    pub fn collection_element_type(&self, store: &crate::types::store::TypeStore) -> ExpectedType {
        if let Some(ty) = self.ty() {
            if let crate::types::store::TypeData::Applied { arguments, .. } = store.get(ty) {
                if let Some(&first) = arguments.first() {
                    return ExpectedType::proper_from(first, ExpectationOrigin::CollectionElement);
                }
            }
        } else if let Self::Inference {
            term: InferenceTerm::Applied { arguments, .. },
            ..
        } = self
        {
            if let Some(first) = arguments.first() {
                return ExpectedType::inference_from(first.clone(), ExpectationOrigin::CollectionElement);
            }
        }
        ExpectedType::None
    }

    pub fn map_key_val_types(&self, store: &crate::types::store::TypeStore) -> (ExpectedType, ExpectedType) {
        if let Some(ty) = self.ty() {
            if let crate::types::store::TypeData::Applied { arguments, .. } = store.get(ty) {
                if arguments.len() >= 2 {
                    return (
                        ExpectedType::proper_from(arguments[0], ExpectationOrigin::ProductComponent),
                        ExpectedType::proper_from(arguments[1], ExpectationOrigin::ProductComponent),
                    );
                }
            }
        } else if let Self::Inference {
            term: InferenceTerm::Applied { arguments, .. },
            ..
        } = self
        {
            if arguments.len() >= 2 {
                return (
                    ExpectedType::inference_from(arguments[0].clone(), ExpectationOrigin::ProductComponent),
                    ExpectedType::inference_from(arguments[1].clone(), ExpectationOrigin::ProductComponent),
                );
            }
        }
        (ExpectedType::None, ExpectedType::None)
    }

    pub fn callable_signature(&self, store: &crate::types::store::TypeStore) -> Option<(Vec<ExpectedType>, ExpectedType)> {
        if let Some(ty) = self.ty() {
            if let crate::types::store::TypeData::Callable(c) = store.get(ty) {
                let params = c
                    .parameters
                    .iter()
                    .map(|p| ExpectedType::proper_from(p.ty, ExpectationOrigin::CallableSignature))
                    .collect();
                let ret = ExpectedType::proper_from(c.return_type, ExpectationOrigin::CallableSignature);
                return Some((params, ret));
            }
        } else if let Self::Inference {
            term: InferenceTerm::Callable(c),
            ..
        } = self
        {
            let params = c
                .parameters
                .iter()
                .map(|p| ExpectedType::inference_from(p.term.clone(), ExpectationOrigin::CallableSignature))
                .collect();
            let ret = ExpectedType::inference_from((*c.return_type).clone(), ExpectationOrigin::CallableSignature);
            return Some((params, ret));
        }
        None
    }

    pub fn contextual_knowledge(&self, ty: TypeId) -> Option<crate::types::evidence::TypeKnowledge> {
        match self {
            Self::Proper { ty: expected_ty, origin } => {
                if *expected_ty == ty {
                    let status = match origin {
                        ExpectationOrigin::DeclarationContract
                        | ExpectationOrigin::ReturnContract
                        | ExpectationOrigin::AssignmentContract
                        | ExpectationOrigin::ContextualBlockParameter => crate::types::evidence::EvidenceStatus::Assumed,
                        ExpectationOrigin::ExplicitCheck => crate::types::evidence::EvidenceStatus::Established,
                        _ => crate::types::evidence::EvidenceStatus::Assumed,
                    };
                    Some(match status {
                        crate::types::evidence::EvidenceStatus::Established => {
                            crate::types::evidence::TypeKnowledge::established(ty, crate::types::evidence::EvidenceOrigin::ContextualDerivation)
                        }
                        crate::types::evidence::EvidenceStatus::Assumed => {
                            crate::types::evidence::TypeKnowledge::assumed(ty, crate::types::evidence::EvidenceOrigin::ContextualDerivation)
                        }
                    })
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}
