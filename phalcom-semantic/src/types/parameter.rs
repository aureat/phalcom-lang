//! Type parameter identities and generic signatures.

use super::id::{InferVarId, KindId, TypeId, TypeParameterId};
use super::variance::Variance;
use crate::diagnostic::SemanticSourceSpan;
use crate::identity::{CallableId, DeclarationId, DispatchSide};

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum TypeParameterOwner {
    Declaration(DeclarationId),
    Callable(CallableId),
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct TypeParameterData {
    pub owner: TypeParameterOwner,
    pub index: u32,
    pub name: Box<str>,
    pub kind: KindId,
    pub variance: Variance,
    pub source: Option<SemanticSourceSpan>,
}

impl TypeParameterData {
    pub fn new(owner: TypeParameterOwner, index: u32, name: impl Into<Box<str>>, kind: KindId) -> Self {
        Self {
            owner,
            index,
            name: name.into(),
            kind,
            variance: Variance::Invariant,
            source: None,
        }
    }

    pub fn with_variance(mut self, variance: Variance) -> Self {
        self.variance = variance;
        self
    }

    pub fn with_source(mut self, source: SemanticSourceSpan) -> Self {
        self.source = Some(source);
        self
    }
}

/// Role of a `Self` type term.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum SelfRole {
    /// Instance type denoted by the current declaration, including specialization.
    InstanceType,
    /// Type of the lexical/dynamic receiver value itself.
    ReceiverValue,
}

/// Owner-relative `Self` type term.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct SelfTypeTerm {
    pub owner: DeclarationId,
    pub side: DispatchSide,
    pub role: SelfRole,
}

/// Term used in generic declaration signatures and constraints.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum TypeTerm {
    Canonical(TypeId),
    SelfType(SelfTypeTerm),
    Infer(InferVarId),
}

impl From<TypeId> for TypeTerm {
    fn from(ty: TypeId) -> Self {
        Self::Canonical(ty)
    }
}

/// Canonical generic constraint owned by a generic signature scope.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum GenericConstraint {
    Subtype { lower: TypeTerm, upper: TypeTerm },
    Equivalent { left: TypeTerm, right: TypeTerm },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GenericSignature {
    pub owner: TypeParameterOwner,
    pub parameters: Box<[TypeParameterId]>,
    pub constraints: Box<[GenericConstraint]>,
}

impl GenericSignature {
    pub fn new(owner: TypeParameterOwner, parameters: Box<[TypeParameterId]>) -> Self {
        Self {
            owner,
            parameters,
            constraints: Box::new([]),
        }
    }

    pub fn with_constraints(owner: TypeParameterOwner, parameters: Box<[TypeParameterId]>, constraints: Box<[GenericConstraint]>) -> Self {
        Self {
            owner,
            parameters,
            constraints,
        }
    }

    pub fn parameter_count(&self) -> usize {
        self.parameters.len()
    }

    pub fn parameter_at(&self, index: usize) -> Option<TypeParameterId> {
        self.parameters.get(index).copied()
    }

    pub fn constraint_count(&self) -> usize {
        self.constraints.len()
    }

    pub fn constraint_at(&self, index: usize) -> Option<&GenericConstraint> {
        self.constraints.get(index)
    }
}
