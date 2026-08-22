//! Epistemic Type Knowledge, Evidence Authority, and Provenance.

use super::id::TypeId;
use phalcom_common::range::SourceRange;

/// Epistemic degree of knowledge established for a value or expression.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TypeKnowledge {
    /// Semantic engine has established a concrete type with supporting evidence.
    Known(TypeEvidence),
    /// Semantic engine cannot establish a type with current evidence.
    Unknown(UnknownReason),
    /// Deliberate static dynamic escape (e.g. `Dynamic` annotation or runtime rest dispatch).
    Dynamic(DynamicReason),
}

impl TypeKnowledge {
    #[inline]
    pub fn is_known(&self) -> bool {
        matches!(self, Self::Known(_))
    }

    #[inline]
    pub fn is_unknown(&self) -> bool {
        matches!(self, Self::Unknown(_))
    }

    #[inline]
    pub fn is_dynamic(&self) -> bool {
        matches!(self, Self::Dynamic(_))
    }

    #[inline]
    pub fn ty(&self) -> Option<TypeId> {
        match self {
            Self::Known(e) => Some(e.ty),
            _ => None,
        }
    }

    pub fn known(ty: impl Into<TypeId>, authority: EvidenceAuthority) -> Self {
        Self::Known(TypeEvidence {
            ty: ty.into(),
            authority,
            provenance: EvidenceSet::default(),
        })
    }

    pub fn known_proper(ty: super::id::ProperTypeId, authority: EvidenceAuthority) -> Self {
        Self::known(ty.raw(), authority)
    }

    pub fn proper_ty(&self, store: &super::store::TypeStore) -> Option<super::id::ProperTypeId> {
        self.ty().and_then(|id| store.proper_type(id).ok())
    }

    pub fn with_range(mut self, range: SourceRange) -> Self {
        if let Self::Known(ref mut ev) = self {
            ev.provenance.ranges.push(range);
        }
        self
    }
}

/// A proven or declared type accompanied by epistemic authority and provenance.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TypeEvidence {
    pub ty: TypeId,
    pub authority: EvidenceAuthority,
    pub provenance: EvidenceSet,
}

/// The epistemic authority under which a type claim is made.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum EvidenceAuthority {
    /// Developer-authored explicit type declaration.
    Declared,
    /// Proven soundly by the type checker or static solver.
    Proven,
    /// Exact syntax literal or constructor expression fact.
    ExactSyntax,
    /// Normalized signature from trusted native universe metadata.
    TrustedNative,
    /// Advisory shape inference (LSP level, not hard compile contradiction authority alone).
    Advisory,
}

impl EvidenceAuthority {
    /// Whether this authority is sound for rejecting code at compile time.
    pub fn is_sound_for_rejection(self) -> bool {
        matches!(self, Self::Declared | Self::Proven | Self::ExactSyntax | Self::TrustedNative)
    }
}

/// Provenance facts explaining where type evidence was synthesized or constrained.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct EvidenceSet {
    pub ranges: Vec<SourceRange>,
    pub descriptions: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum UnknownReason {
    UnannotatedDeclaration,
    UnresolvedName(Box<str>),
    DynamicMessageSend,
    OpaqueNative,
    RecursiveFixpoint,
    UncheckedExpression,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DynamicReason {
    ExplicitEscape,
    DynamicRestPack,
    RuntimeReflection,
}
