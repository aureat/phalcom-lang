//! Canonical generic parameters, variance, and signature-owned constraints.

use crate::identity::{SourceSpanRef, StableCallableRef, StableDeclarationRef};
use crate::kind::KindNodeId;
use crate::type_node::TypeNodeId;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
#[derive(phalcom_type_meta_derive::BudgetDecode)]
pub enum StableTypeParameterOwnerRef {
    Declaration(StableDeclarationRef),
    Callable(StableCallableRef),
    Impl(crate::identity::StableImplRef),
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
#[derive(phalcom_type_meta_derive::BudgetDecode)]
pub struct StableTypeParameterRef {
    pub owner: StableTypeParameterOwnerRef,
    pub index: u32,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
#[derive(phalcom_type_meta_derive::BudgetDecode)]
pub enum VarianceRef {
    Covariant,
    Contravariant,
    Invariant,
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
#[derive(phalcom_type_meta_derive::BudgetDecode)]
pub enum GenericParameterModeRef {
    #[default]
    Single,
    PositionalPack,
    LabeledPack,
    CompletePack,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[derive(phalcom_type_meta_derive::BudgetDecode)]
pub struct TypeParameterRecord {
    pub id: StableTypeParameterRef,
    pub name: Box<str>,
    pub kind: KindNodeId,
    pub variance: VarianceRef,
    #[serde(default)]
    pub mode: GenericParameterModeRef,
    pub source: Option<SourceSpanRef>,
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
#[derive(phalcom_type_meta_derive::BudgetDecode)]
pub struct GenericSignatureRecordId(pub u32);

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[derive(phalcom_type_meta_derive::BudgetDecode)]
pub struct GenericSignatureRecord {
    pub owner: StableTypeParameterOwnerRef,
    pub parameters: Box<[StableTypeParameterRef]>,
    pub constraints: Box<[GenericConstraintRef]>,
}

/// Stable identity for an applied trait contract in a generic constraint.
///
/// Traits are behavioral declarations rather than nominal type nodes, so the
/// declaration and its canonical type arguments are carried explicitly.
/// Stable identity for an associated type requirement within a trait declaration.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
#[derive(phalcom_type_meta_derive::BudgetDecode)]
pub struct StableAssociatedTypeRequirementRef {
    pub owner: StableDeclarationRef,
    pub index: u32,
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
#[derive(phalcom_type_meta_derive::BudgetDecode)]
pub struct StableTraitApplicationRef {
    pub declaration: StableDeclarationRef,
    pub arguments: Box<[TypeNodeId]>,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Deserialize, Serialize)]
#[derive(phalcom_type_meta_derive::BudgetDecode)]
pub enum GenericConstraintRef {
    Subtype {
        lower: crate::type_node::TypeNodeId,
        upper: crate::type_node::TypeNodeId,
    },
    Equivalent {
        left: crate::type_node::TypeNodeId,
        right: crate::type_node::TypeNodeId,
    },
    Conforms {
        subject: TypeNodeId,
        trait_application: StableTraitApplicationRef,
    },
}
