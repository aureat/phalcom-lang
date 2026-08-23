//! Bidirectional checking type expectations (Spec 04.5).

use crate::checker::inference::InferenceTerm;
use crate::types::id::TypeId;

/// Contextual type expectation propagated downward into an expression during bidirectional checking.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub enum ExpectedType {
    /// No contextual expectation (synthesis mode).
    #[default]
    None,
    /// A known canonical proper type expectation (checking mode).
    Proper(TypeId),
    /// A solver-local inference term expectation.
    Inference(InferenceTerm),
}

impl ExpectedType {
    pub fn none() -> Self {
        Self::None
    }

    pub fn proper(ty: TypeId) -> Self {
        Self::Proper(ty)
    }

    pub fn inference(term: InferenceTerm) -> Self {
        Self::Inference(term)
    }

    pub fn is_none(&self) -> bool {
        matches!(self, Self::None)
    }

    pub fn ty(&self) -> Option<TypeId> {
        match self {
            Self::Proper(ty) => Some(*ty),
            Self::Inference(InferenceTerm::Canonical(ty)) => Some(*ty),
            _ => None,
        }
    }

    pub fn from_knowledge(k: &crate::types::evidence::TypeKnowledge) -> Self {
        if let Some(ty) = k.ty() { Self::Proper(ty) } else { Self::None }
    }

    pub fn collection_element_type(&self, store: &crate::types::store::TypeStore) -> ExpectedType {
        if let Some(ty) = self.ty() {
            if let crate::types::store::TypeData::Applied { arguments, .. } = store.get(ty) {
                if let Some(&first) = arguments.first() {
                    return ExpectedType::Proper(first);
                }
            }
        } else if let Self::Inference(InferenceTerm::Applied { arguments, .. }) = self {
            if let Some(first) = arguments.first() {
                return ExpectedType::Inference(first.clone());
            }
        }
        ExpectedType::None
    }

    pub fn map_key_val_types(&self, store: &crate::types::store::TypeStore) -> (ExpectedType, ExpectedType) {
        if let Some(ty) = self.ty() {
            if let crate::types::store::TypeData::Applied { arguments, .. } = store.get(ty) {
                if arguments.len() >= 2 {
                    return (ExpectedType::Proper(arguments[0]), ExpectedType::Proper(arguments[1]));
                }
            }
        } else if let Self::Inference(InferenceTerm::Applied { arguments, .. }) = self {
            if arguments.len() >= 2 {
                return (ExpectedType::Inference(arguments[0].clone()), ExpectedType::Inference(arguments[1].clone()));
            }
        }
        (ExpectedType::None, ExpectedType::None)
    }

    pub fn callable_signature(&self, store: &crate::types::store::TypeStore) -> Option<(Vec<ExpectedType>, ExpectedType)> {
        if let Some(ty) = self.ty() {
            if let crate::types::store::TypeData::Callable(c) = store.get(ty) {
                let params = c.parameters.iter().map(|p| ExpectedType::Proper(p.ty)).collect();
                let ret = ExpectedType::Proper(c.return_type);
                return Some((params, ret));
            }
        } else if let Self::Inference(InferenceTerm::Callable(c)) = self {
            let params = c.parameters.iter().map(|p| ExpectedType::Inference(p.term.clone())).collect();
            let ret = ExpectedType::Inference((*c.return_type).clone());
            return Some((params, ret));
        }
        None
    }
}
