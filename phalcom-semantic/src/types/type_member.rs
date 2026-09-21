//! Canonical type-member signatures, selectors, and compiler-intrinsic operations.

use crate::types::id::{KindId, TypeId};
use phalcom_ast::ast::TypeMemberSelectorSyntax;

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
    ConstInt,
    Type(TypeId),
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
    TupleAt,
    TupleMap,
    RecordMap,
    TypePackMap,
    LabeledTypePackMap,
    RecordRowMap,
}

impl IntrinsicTypeOperation {
    pub fn name(&self) -> &'static str {
        match self {
            Self::TuplePositionals => "Tuple::Positionals",
            Self::TupleLabeled => "Tuple::Labeled",
            Self::TupleAt => "Tuple::At",
            Self::TupleMap => "Tuple::Map",
            Self::RecordMap => "Record::Map",
            Self::TypePackMap => "TypePack::Map",
            Self::LabeledTypePackMap => "LabeledTypePack::Map",
            Self::RecordRowMap => "RecordRow::Map",
        }
    }
}

/// Canonical signature for a domain-native or compiler-intrinsic type member.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct TypeMemberSignature {
    pub selector: TypeMemberSelector,
    pub parameters: Box<[TypeMemberParameter]>,
    pub result_domain: TypeMemberDomain,
    pub intrinsic: Option<IntrinsicTypeOperation>,
}

/// Unique identifier for an interned type member.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct TypeMemberId(pub u32);

/// Lookup an intrinsic type operation for a target domain and selector.
pub fn lookup_intrinsic_type_operation(target_domain: &str, selector: &TypeMemberSelector) -> Option<IntrinsicTypeOperation> {
    match target_domain {
        "Tuple" | "tuple" => match selector {
            TypeMemberSelector::Named(name) if name.as_ref() == "Positionals" => Some(IntrinsicTypeOperation::TuplePositionals),
            TypeMemberSelector::Named(name) if name.as_ref() == "Labeled" => Some(IntrinsicTypeOperation::TupleLabeled),
            TypeMemberSelector::Named(name) if name.as_ref() == "Map" => Some(IntrinsicTypeOperation::TupleMap),
            TypeMemberSelector::Named(name) if name.as_ref() == "At" => Some(IntrinsicTypeOperation::TupleAt),
            TypeMemberSelector::Subscript => Some(IntrinsicTypeOperation::TupleAt),
            _ => None,
        },
        "Record" | "record" => match selector {
            TypeMemberSelector::Named(name) if name.as_ref() == "Map" => Some(IntrinsicTypeOperation::RecordMap),
            _ => None,
        },
        "TypePack" => match selector {
            TypeMemberSelector::Named(name) if name.as_ref() == "Map" => Some(IntrinsicTypeOperation::TypePackMap),
            _ => None,
        },
        "LabeledTypePack" => match selector {
            TypeMemberSelector::Named(name) if name.as_ref() == "Map" => Some(IntrinsicTypeOperation::LabeledTypePackMap),
            _ => None,
        },
        "RecordRow" => match selector {
            TypeMemberSelector::Named(name) if name.as_ref() == "Map" => Some(IntrinsicTypeOperation::RecordRowMap),
            _ => None,
        },
        _ => None,
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
}
