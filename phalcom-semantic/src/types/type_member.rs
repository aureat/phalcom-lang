//! Canonical type-member signatures, selectors, and compiler-intrinsic operations.

use crate::identity::DeclarationId;
use crate::types::id::{KindId, TypeId};
use crate::types::store::TypeStore;
use phalcom_ast::ast::TypeMemberSelectorSyntax;
use phalcom_common::modifier::ImplementationKind;

/// Canonical selector for a type-level member or projection.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum TypeMemberSelector {
    Named(Box<str>),
    Subscript,
}

impl TypeMemberSelector {
    pub fn named(name: impl Into<Box<str>>) -> Self {
        Self::Named(name.into())
    }

    pub fn subscript() -> Self {
        Self::Subscript
    }

    pub fn from_syntax(syntax: &TypeMemberSelectorSyntax) -> Self {
        match syntax {
            TypeMemberSelectorSyntax::Named { name, .. } => {
                if name == "At" {
                    Self::Subscript
                } else {
                    Self::Named(name.clone().into_boxed_str())
                }
            }
            TypeMemberSelectorSyntax::Subscript { .. } => Self::Subscript,
        }
    }

    pub fn name(&self) -> &str {
        match self {
            Self::Named(name) => name,
            Self::Subscript => "[]",
        }
    }
}

/// The domain requirement for a type-member parameter or return result.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum TypeMemberDomain {
    Kind(KindId),
}

/// Stable declaration identity for a source type member.
///
/// Unlike `TypeMemberId`, this identity is independent of interner allocation order and
/// remains stable across semantic snapshots. `At` is canonicalized to `Subscript` in the
/// selector before constructing this identity.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct TypeMemberDeclarationId {
    pub owner: DeclarationId,
    pub selector: TypeMemberSelector,
}

impl TypeMemberDeclarationId {
    pub fn new(owner: DeclarationId, selector: TypeMemberSelector) -> Self {
        Self { owner, selector }
    }
}

/// A formal parameter of a type member.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct TypeMemberParameter {
    pub name: Box<str>,
    pub domain: TypeMemberDomain,
}

/// Known compiler-intrinsic type operations.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum IntrinsicTypeOperation {
    TuplePositionals,
    TupleLabeled,
    TupleEntries,
    TupleElement,
    TupleAt,
    TupleMap,
    RecordFields,
    RecordValue,
    RecordMap,
    TypePackMap,
    LabeledTypePackMap,
    RecordRowMap,
    CompleteTypePackPositionals,
    CompleteTypePackLabeled,
    CompleteTypePackMap,
}

impl IntrinsicTypeOperation {
    pub const ALL: [Self; 15] = [
        Self::TuplePositionals,
        Self::TupleLabeled,
        Self::TupleEntries,
        Self::TupleElement,
        Self::TupleAt,
        Self::TupleMap,
        Self::RecordFields,
        Self::RecordValue,
        Self::RecordMap,
        Self::TypePackMap,
        Self::LabeledTypePackMap,
        Self::RecordRowMap,
        Self::CompleteTypePackPositionals,
        Self::CompleteTypePackLabeled,
        Self::CompleteTypePackMap,
    ];

    pub fn name(&self) -> &'static str {
        match self {
            Self::TuplePositionals => "Tuple::Positionals",
            Self::TupleLabeled => "Tuple::Labeled",
            Self::TupleEntries => "Tuple::Entries",
            Self::TupleElement => "Tuple::Element",
            Self::TupleAt => "Tuple::At",
            Self::TupleMap => "Tuple::Map",
            Self::RecordFields => "Record::Fields",
            Self::RecordValue => "Record::Value",
            Self::RecordMap => "Record::Map",
            Self::TypePackMap => "TypePack::Map",
            Self::LabeledTypePackMap => "LabeledTypePack::Map",
            Self::RecordRowMap => "RecordFields::Map",
            Self::CompleteTypePackPositionals => "CompleteTypePack::Positionals",
            Self::CompleteTypePackLabeled => "CompleteTypePack::Labeled",
            Self::CompleteTypePackMap => "CompleteTypePack::Map",
        }
    }

    /// Returns the declared parameter and result kinds implemented by this evaluator.
    pub fn signature_kinds(self, store: &mut TypeStore) -> (Box<[KindId]>, KindId) {
        let type_to_type = store.arrow_kind(vec![KindId::TYPE].into_boxed_slice(), KindId::TYPE);
        match self {
            Self::TuplePositionals => (Box::new([]), KindId::TYPE_PACK),
            Self::TupleLabeled => (Box::new([]), KindId::LABELED_TYPE_PACK),
            Self::TupleEntries => (Box::new([]), KindId::COMPLETE_TYPE_PACK),
            Self::TupleElement => (Box::new([]), KindId::TYPE),
            Self::TupleAt => {
                let const_int = store.static_domain_kind(store.const_int_domain());
                (Box::new([const_int]), KindId::TYPE)
            }
            Self::TupleMap => (Box::new([type_to_type]), KindId::TYPE),
            Self::RecordFields => (Box::new([]), KindId::RECORD_ROW),
            Self::RecordValue => (Box::new([]), KindId::TYPE),
            Self::RecordMap => (Box::new([type_to_type]), KindId::TYPE),
            Self::TypePackMap => (Box::new([type_to_type]), KindId::TYPE_PACK),
            Self::LabeledTypePackMap => (Box::new([type_to_type]), KindId::LABELED_TYPE_PACK),
            Self::RecordRowMap => (Box::new([type_to_type]), KindId::RECORD_ROW),
            Self::CompleteTypePackPositionals => (Box::new([]), KindId::TYPE_PACK),
            Self::CompleteTypePackLabeled => (Box::new([]), KindId::LABELED_TYPE_PACK),
            Self::CompleteTypePackMap => (Box::new([type_to_type]), KindId::COMPLETE_TYPE_PACK),
        }
    }
}

/// Canonical compiler authority that owns a structural type-member family.
/// Source declaration names are deliberately absent from dispatch decisions.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum IntrinsicTypeOwner {
    Tuple,
    Record,
    TypePack,
    LabeledTypePack,
    CompleteTypePack,
    RecordRow,
}

/// Canonical signature for a domain-native or compiler-intrinsic type member.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct TypeMemberSignature {
    pub selector: TypeMemberSelector,
    pub parameters: Box<[TypeMemberParameter]>,
    pub result_domain: TypeMemberDomain,
    pub intrinsic: Option<IntrinsicTypeOperation>,
    pub implementation: ImplementationKind,
}

/// A checked member application failed its declared arity or argument-domain contract.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TypeMemberApplicationError {
    ArityMismatch { expected: usize, actual: usize },
    ArgumentDomainMismatch { index: usize, expected: KindId, actual: KindId },
}

/// Validates arguments against an already-selected declaration signature.
///
/// This helper performs no name lookup. Callers must first select a member through its
/// owning semantic surface, then retrieve this signature by declaration identity.
pub fn validate_type_member_arguments(
    store: &mut TypeStore,
    signature: &TypeMemberSignature,
    arguments: &[TypeMemberArgument],
) -> Result<(), TypeMemberApplicationError> {
    if arguments.len() != signature.parameters.len() {
        return Err(TypeMemberApplicationError::ArityMismatch {
            expected: signature.parameters.len(),
            actual: arguments.len(),
        });
    }
    for (index, (parameter, argument)) in signature.parameters.iter().zip(arguments).enumerate() {
        let TypeMemberDomain::Kind(expected) = &parameter.domain;
        let actual = match argument {
            TypeMemberArgument::Type(ty) => store.kind_of(*ty),
            TypeMemberArgument::Const(ConstTerm::Int(value)) => {
                let integer = store.const_int(*value);
                store.kind_of(integer)
            }
            TypeMemberArgument::RecordRow(_) => KindId::RECORD_ROW,
        };
        if actual != *expected {
            return Err(TypeMemberApplicationError::ArgumentDomainMismatch {
                index,
                expected: *expected,
                actual,
            });
        }
    }
    Ok(())
}

/// Unique identifier for an interned type member.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct TypeMemberId(pub u32);

/// Lookup an intrinsic type operation for a target domain and selector.
pub fn lookup_intrinsic_type_operation(owner: IntrinsicTypeOwner, selector: &TypeMemberSelector) -> Option<IntrinsicTypeOperation> {
    match owner {
        IntrinsicTypeOwner::Tuple => match selector {
            TypeMemberSelector::Named(name) if name.as_ref() == "Positionals" => Some(IntrinsicTypeOperation::TuplePositionals),
            TypeMemberSelector::Named(name) if name.as_ref() == "Labeled" => Some(IntrinsicTypeOperation::TupleLabeled),
            TypeMemberSelector::Named(name) if name.as_ref() == "Entries" => Some(IntrinsicTypeOperation::TupleEntries),
            TypeMemberSelector::Named(name) if name.as_ref() == "Element" => Some(IntrinsicTypeOperation::TupleElement),
            TypeMemberSelector::Named(name) if name.as_ref() == "Map" => Some(IntrinsicTypeOperation::TupleMap),
            TypeMemberSelector::Named(name) if name.as_ref() == "At" => Some(IntrinsicTypeOperation::TupleAt),
            TypeMemberSelector::Subscript => Some(IntrinsicTypeOperation::TupleAt),
            _ => None,
        },
        IntrinsicTypeOwner::Record => match selector {
            TypeMemberSelector::Named(name) if name.as_ref() == "Fields" => Some(IntrinsicTypeOperation::RecordFields),
            TypeMemberSelector::Named(name) if name.as_ref() == "Value" => Some(IntrinsicTypeOperation::RecordValue),
            TypeMemberSelector::Named(name) if name.as_ref() == "Map" => Some(IntrinsicTypeOperation::RecordMap),
            _ => None,
        },
        IntrinsicTypeOwner::TypePack => match selector {
            TypeMemberSelector::Named(name) if name.as_ref() == "Map" => Some(IntrinsicTypeOperation::TypePackMap),
            _ => None,
        },
        IntrinsicTypeOwner::LabeledTypePack => match selector {
            TypeMemberSelector::Named(name) if name.as_ref() == "Map" => Some(IntrinsicTypeOperation::LabeledTypePackMap),
            _ => None,
        },
        IntrinsicTypeOwner::RecordRow => match selector {
            TypeMemberSelector::Named(name) if name.as_ref() == "Map" => Some(IntrinsicTypeOperation::RecordRowMap),
            _ => None,
        },
        IntrinsicTypeOwner::CompleteTypePack => match selector {
            TypeMemberSelector::Named(name) if name.as_ref() == "Positionals" => Some(IntrinsicTypeOperation::CompleteTypePackPositionals),
            TypeMemberSelector::Named(name) if name.as_ref() == "Labeled" => Some(IntrinsicTypeOperation::CompleteTypePackLabeled),
            TypeMemberSelector::Named(name) if name.as_ref() == "Map" => Some(IntrinsicTypeOperation::CompleteTypePackMap),
            _ => None,
        },
    }
}

/// A constant value passed as a type-member argument.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ConstTerm {
    Int(i64),
}

/// The subject of a generalized type-level projection.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum ProjectionSubject {
    Type(TypeId),
    RecordRow(crate::types::id::RecordRowId),
}

/// An argument passed to a type member application/projection.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum TypeMemberArgument {
    Type(TypeId),
    Const(ConstTerm),
    RecordRow(crate::types::id::RecordRowId),
}

/// Canonical semantic type-level member projection.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct TypeLevelProjection {
    pub subject: ProjectionSubject,
    pub selector: TypeMemberSelector,
    pub arguments: Box<[TypeMemberArgument]>,
    /// Present when this projection selected a checked source declaration via
    /// the owner's AssociatedSurface, including a source declaration backed by
    /// a compiler intrinsic evaluator. Source-authored public intrinsic
    /// projections must retain this identity; `None` is reserved for internal
    /// synthetic semantic projections. Legacy wire omission is handled by the
    /// optional metadata field, not by importing source-less values here.
    pub source_declaration: Option<TypeMemberDeclarationId>,
}
