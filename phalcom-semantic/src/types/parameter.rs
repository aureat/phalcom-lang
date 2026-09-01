//! Type parameter identities and generic signatures.

use super::id::{InferVarId, KindId, TypeId, TypeParameterId};
use super::store::TypeStore;
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
    pub parameter_kinds: Box<[KindId]>,
    pub parameter_kind_shapes: Box<[Box<str>]>,
    pub parameter_variances: Box<[Variance]>,
    pub constraint_shapes: Box<[Box<str>]>,
    pub constraints: Box<[GenericConstraint]>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GenericSignaturePublicationError {
    MissingParameter { index: u32, parameter: TypeParameterId },
    OwnerMismatch { parameter: TypeParameterId },
    NonContiguousParameter { parameter: TypeParameterId, expected: u32, actual: u32 },
    RecordRowParameterForm { parameter: TypeParameterId },
    InvalidCanonicalType { ty: TypeId },
    InferenceVariable { variable: InferVarId },
}

impl GenericSignature {
    pub fn new(owner: TypeParameterOwner, parameters: Box<[TypeParameterId]>) -> Self {
        Self {
            owner,
            parameters,
            parameter_kinds: Box::new([]),
            parameter_kind_shapes: Box::new([]),
            parameter_variances: Box::new([]),
            constraint_shapes: Box::new([]),
            constraints: Box::new([]),
        }
    }

    pub fn with_constraints(owner: TypeParameterOwner, parameters: Box<[TypeParameterId]>, constraints: Box<[GenericConstraint]>) -> Self {
        Self {
            owner,
            parameters,
            parameter_kinds: Box::new([]),
            parameter_kind_shapes: Box::new([]),
            parameter_variances: Box::new([]),
            constraint_shapes: Box::new([]),
            constraints,
        }
    }

    pub fn with_parameter_metadata(mut self, kinds: Box<[KindId]>, variances: Box<[Variance]>) -> Self {
        self.parameter_kinds = kinds;
        self.parameter_variances = variances;
        self
    }

    pub fn with_parameter_kind_shapes(mut self, shapes: Box<[Box<str>]>) -> Self {
        self.parameter_kind_shapes = shapes;
        self
    }

    pub fn with_constraint_shapes(mut self, shapes: Box<[Box<str>]>) -> Self {
        self.constraint_shapes = shapes;
        self
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

    pub fn validate_publishable(&self, store: &TypeStore) -> Result<(), GenericSignaturePublicationError> {
        for (index, &parameter) in self.parameters.iter().enumerate() {
            let expected = index as u32;
            let Some(found) = store.find_type_parameter_id(&self.owner, expected) else {
                return Err(GenericSignaturePublicationError::MissingParameter { index: expected, parameter });
            };
            if found != parameter {
                return Err(GenericSignaturePublicationError::OwnerMismatch { parameter });
            }
            let data = store.type_parameter(parameter);
            if data.owner != self.owner {
                return Err(GenericSignaturePublicationError::OwnerMismatch { parameter });
            }
            if data.index != expected {
                return Err(GenericSignaturePublicationError::NonContiguousParameter {
                    parameter,
                    expected,
                    actual: data.index,
                });
            }
            if data.kind == KindId::RECORD_ROW && store.contains_parameter_type(parameter) {
                return Err(GenericSignaturePublicationError::RecordRowParameterForm { parameter });
            }
        }

        let validate_term = |term: &TypeTerm| match term {
            TypeTerm::Canonical(ty) if ty.index() < store.type_count() => Ok(()),
            TypeTerm::Canonical(ty) => Err(GenericSignaturePublicationError::InvalidCanonicalType { ty: *ty }),
            TypeTerm::SelfType(_) => Ok(()),
            TypeTerm::Infer(variable) => Err(GenericSignaturePublicationError::InferenceVariable { variable: *variable }),
        };
        for constraint in self.constraints.iter() {
            match constraint {
                GenericConstraint::Subtype { lower, upper } => {
                    validate_term(lower)?;
                    validate_term(upper)?;
                }
                GenericConstraint::Equivalent { left, right } => {
                    validate_term(left)?;
                    validate_term(right)?;
                }
            }
        }
        Ok(())
    }
}
