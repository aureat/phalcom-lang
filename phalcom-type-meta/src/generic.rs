//! Canonical generic parameters, variance, and signature-owned constraints.

use crate::identity::{SourceSpanRef, StableCallableRef, StableDeclarationRef};
use crate::kind::KindNodeId;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
pub enum StableTypeParameterOwnerRef {
    Declaration(StableDeclarationRef),
    Callable(StableCallableRef),
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
pub struct StableTypeParameterRef {
    pub owner: StableTypeParameterOwnerRef,
    pub index: u32,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
pub enum VarianceRef {
    Covariant,
    Contravariant,
    Invariant,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct TypeParameterRecord {
    pub id: StableTypeParameterRef,
    pub name: Box<str>,
    pub kind: KindNodeId,
    pub variance: VarianceRef,
    pub source: Option<SourceSpanRef>,
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
pub struct GenericSignatureRecordId(pub u32);

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct GenericSignatureRecord {
    pub owner: StableTypeParameterOwnerRef,
    pub parameters: Box<[StableTypeParameterRef]>,
    pub constraints: Box<[GenericConstraintRef]>,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Deserialize, Serialize)]
pub enum GenericConstraintRef {
    Subtype {
        lower: crate::type_node::TypeNodeId,
        upper: crate::type_node::TypeNodeId,
    },
    Equivalent {
        left: crate::type_node::TypeNodeId,
        right: crate::type_node::TypeNodeId,
    },
}
