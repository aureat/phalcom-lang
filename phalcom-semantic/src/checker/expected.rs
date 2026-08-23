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
}
