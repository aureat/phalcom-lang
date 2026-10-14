//! Domain-neutral member projection representation and evaluation outcomes.
//!
//! A projection is domain-neutral: it applies a type member or trait-associated
//! member to canonical arguments under optional conformance evidence. Projections
//! whose result domain is kinded may lower into `TypeStore`'s `TypeLevelProjection`,
//! while projections whose result domain is a value type (`ValueType`) reduce to
//! `StaticValueRef` or remain a symbolic `MemberProjection` when stuck.

use crate::generic_evidence::GenericConformanceKey;
use crate::static_domain::StaticValueDomain;
use crate::static_value::{StaticValueRef, StaticValueStore, StaticValueStoreError};
use crate::traits::{AssociatedTypeRequirementId, TraitRef};
use crate::types::evidence::UnknownReason;
use crate::types::id::{RecordRowId, TypeId};
use crate::types::outcome::{BlockReason, BudgetReport, DynamicBoundaryObligation};
use crate::types::store::{TypeData, TypeStore};
use crate::types::type_member::{ConstTerm, IntrinsicTypeOperation, TypeMemberArgument, TypeMemberDeclarationId, TypeMemberSignature};

/// A domain-neutral argument passed to a member application/projection.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum MemberProjectionArgument {
    Kinded(TypeMemberArgument),
    Value(StaticValueRef),
}

impl From<TypeMemberArgument> for MemberProjectionArgument {
    fn from(arg: TypeMemberArgument) -> Self {
        Self::Kinded(arg)
    }
}

impl From<StaticValueRef> for MemberProjectionArgument {
    fn from(val: StaticValueRef) -> Self {
        Self::Value(val)
    }
}

impl MemberProjectionArgument {
    pub fn as_kinded(&self) -> Option<&TypeMemberArgument> {
        match self {
            Self::Kinded(arg) => Some(arg),
            Self::Value(_) => None,
        }
    }

    pub fn as_value(&self) -> Option<StaticValueRef> {
        match self {
            Self::Value(val) => Some(*val),
            Self::Kinded(_) => None,
        }
    }
}

/// The subject of a domain-neutral projection.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum MemberProjectionSubject {
    Type(TypeId),
    RecordRow(RecordRowId),
    Value(StaticValueRef),
}

impl From<TypeId> for MemberProjectionSubject {
    fn from(ty: TypeId) -> Self {
        Self::Type(ty)
    }
}

impl From<RecordRowId> for MemberProjectionSubject {
    fn from(row: RecordRowId) -> Self {
        Self::RecordRow(row)
    }
}

impl From<StaticValueRef> for MemberProjectionSubject {
    fn from(val: StaticValueRef) -> Self {
        Self::Value(val)
    }
}

/// Canonical semantic identity of the member being projected.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum MemberDeclarationIdentity {
    TypeMember(TypeMemberDeclarationId),
    TraitAssociated {
        trait_ref: TraitRef,
        requirement: AssociatedTypeRequirementId,
    },
    Intrinsic(IntrinsicTypeOperation),
}

/// A domain-neutral symbolic member projection.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct MemberProjection {
    pub subject: MemberProjectionSubject,
    pub member: MemberDeclarationIdentity,
    pub arguments: Box<[MemberProjectionArgument]>,
    pub evidence: Option<GenericConformanceKey>,
    pub result_domain: StaticValueDomain,
}

/// The typed outcome of projecting or reducing a member.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum StaticMemberProjectionOutcome {
    ReducedValue(StaticValueRef),
    ReducedKinded(TypeId),
    Stuck(MemberProjection),
    Invalid(Box<str>),
    Unknown(UnknownReason),
    Blocked(BlockReason),
    Dynamic(DynamicBoundaryObligation),
    Cancelled,
    BudgetExceeded(BudgetReport),
    InternalFailure(Box<str>),
}

impl StaticMemberProjectionOutcome {
    pub fn as_reduced_value(&self) -> Option<StaticValueRef> {
        match self {
            Self::ReducedValue(val) => Some(*val),
            _ => None,
        }
    }

    pub fn as_reduced_kinded(&self) -> Option<TypeId> {
        match self {
            Self::ReducedKinded(ty) => Some(*ty),
            _ => None,
        }
    }

    pub fn is_stuck(&self) -> bool {
        matches!(self, Self::Stuck(_))
    }
}

/// Errors occurring during member projection argument validation.
#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
pub enum MemberProjectionApplicationError {
    #[error("arity mismatch: expected {expected}, got {actual}")]
    ArityMismatch { expected: usize, actual: usize },
    #[error("domain mismatch at parameter {index}: expected {expected:?}")]
    DomainMismatch { index: usize, expected: StaticValueDomain },
    #[error("static value store error: {0}")]
    ValueStore(StaticValueStoreError),
}

/// Validates member projection arguments against a type member signature.
pub fn validate_member_projection_arguments(
    types: &mut TypeStore,
    values: &StaticValueStore,
    signature: &TypeMemberSignature,
    arguments: &[MemberProjectionArgument],
    field_key_type: Option<TypeId>,
) -> Result<(), MemberProjectionApplicationError> {
    if arguments.len() != signature.parameters.len() {
        return Err(MemberProjectionApplicationError::ArityMismatch {
            expected: signature.parameters.len(),
            actual: arguments.len(),
        });
    }

    for (index, (param, arg)) in signature.parameters.iter().zip(arguments.iter()).enumerate() {
        let expected_domain = param.domain.to_static_value_domain();
        match arg {
            MemberProjectionArgument::Kinded(kinded) => match (param.domain, kinded) {
                (crate::types::type_member::TypeMemberDomain::Kind(expected_kind), TypeMemberArgument::Type(ty)) => {
                    if types.kind_of(*ty) != expected_kind {
                        return Err(MemberProjectionApplicationError::DomainMismatch {
                            index,
                            expected: expected_domain,
                        });
                    }
                }
                (crate::types::type_member::TypeMemberDomain::Kind(expected_kind), TypeMemberArgument::Const(ConstTerm::Int(_))) => {
                    if expected_kind != types.static_domain_kind(types.const_int_domain()) {
                        return Err(MemberProjectionApplicationError::DomainMismatch {
                            index,
                            expected: expected_domain,
                        });
                    }
                }
                (crate::types::type_member::TypeMemberDomain::Kind(expected_kind), TypeMemberArgument::RecordRow(_)) => {
                    if expected_kind != crate::types::id::KindId::RECORD_ROW {
                        return Err(MemberProjectionApplicationError::DomainMismatch {
                            index,
                            expected: expected_domain,
                        });
                    }
                }
                (crate::types::type_member::TypeMemberDomain::ValueType(expected_ty), TypeMemberArgument::Type(ty)) => {
                    if *ty != expected_ty {
                        if matches!(types.get(*ty), TypeData::ConstInt(_)) {
                            let int_decl = crate::core_surface::universe_declaration(phalcom_native_meta::UniverseKey::Int);
                            if !matches!(types.get(expected_ty), TypeData::Nominal { declaration } if declaration == &int_decl) {
                                return Err(MemberProjectionApplicationError::DomainMismatch {
                                    index,
                                    expected: expected_domain,
                                });
                            }
                        } else {
                            return Err(MemberProjectionApplicationError::DomainMismatch {
                                index,
                                expected: expected_domain,
                            });
                        }
                    }
                }
                (crate::types::type_member::TypeMemberDomain::ValueType(expected_ty), TypeMemberArgument::Const(ConstTerm::Int(_))) => {
                    let int_decl = crate::core_surface::universe_declaration(phalcom_native_meta::UniverseKey::Int);
                    if !matches!(types.get(expected_ty), TypeData::Nominal { declaration } if declaration == &int_decl) {
                        return Err(MemberProjectionApplicationError::DomainMismatch {
                            index,
                            expected: expected_domain,
                        });
                    }
                }
                _ => {
                    return Err(MemberProjectionApplicationError::DomainMismatch {
                        index,
                        expected: expected_domain,
                    });
                }
            },
            MemberProjectionArgument::Value(val_ref) => match param.domain {
                crate::types::type_member::TypeMemberDomain::ValueType(expected_ty) => {
                    if Some(expected_ty) == field_key_type {
                        if values.field_key_of(*val_ref).is_err() {
                            return Err(MemberProjectionApplicationError::DomainMismatch {
                                index,
                                expected: expected_domain,
                            });
                        }
                    } else {
                        let actual_domain = values.domain_of(*val_ref, types).map_err(MemberProjectionApplicationError::ValueStore)?;
                        if actual_domain != expected_domain {
                            return Err(MemberProjectionApplicationError::DomainMismatch {
                                index,
                                expected: expected_domain,
                            });
                        }
                    }
                }
                crate::types::type_member::TypeMemberDomain::Kind(expected_kind) => {
                    let actual_domain = values.domain_of(*val_ref, types).map_err(MemberProjectionApplicationError::ValueStore)?;
                    if actual_domain != StaticValueDomain::Kind(expected_kind) {
                        return Err(MemberProjectionApplicationError::DomainMismatch {
                            index,
                            expected: expected_domain,
                        });
                    }
                }
            },
        }
    }

    Ok(())
}
