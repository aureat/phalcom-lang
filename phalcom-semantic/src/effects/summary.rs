//! Effect summary and opacity reasoning.

use super::atom::EffectSet;
use crate::diagnostic::SemanticDiagnostic;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EffectKnowledge {
    Known(EffectSet),
    Opaque(EffectOpaqueReason),
    Invalid(Box<[SemanticDiagnostic]>),
}

impl EffectKnowledge {
    pub const PURE: Self = Self::Known(EffectSet::EMPTY);

    pub fn is_known_pure(&self) -> bool {
        matches!(self, Self::Known(set) if set.is_empty())
    }

    pub fn join(&self, other: &Self) -> Self {
        match (self, other) {
            (Self::Invalid(d1), _) => Self::Invalid(d1.clone()),
            (_, Self::Invalid(d2)) => Self::Invalid(d2.clone()),
            (Self::Opaque(r1), _) => Self::Opaque(r1.clone()),
            (_, Self::Opaque(r2)) => Self::Opaque(r2.clone()),
            (Self::Known(s1), Self::Known(s2)) => Self::Known(s1.join(*s2)),
        }
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum EffectOpaqueReason {
    MissingNativeMetadata,
    DynamicDispatch,
    ReflectivePerform,
    DoesNotUnderstandBoundary,
    ForeignBoundary,
    UnknownDependency,
    UnsupportedConstruct,
}
