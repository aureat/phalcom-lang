//! Indexed kind graph.

use crate::fingerprint::Fingerprint128;
use crate::identity::StableDeclarationRef;
use serde::{Deserialize, Serialize};

#[repr(transparent)]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize, phalcom_type_meta_derive::BudgetDecode)]
pub struct KindNodeId(pub u32);

#[derive(Clone, Debug, Eq, Hash, PartialEq, Deserialize, Serialize, phalcom_type_meta_derive::BudgetDecode)]
pub enum StaticDomainRef {
    ConstInt,
    Nominal {
        declaration: StableDeclarationRef,
    },
    Associated {
        declaration: crate::identity::StableAssociatedStaticDomainDeclarationRef,
        owner_environment: Box<[crate::type_node::StaticArgumentRef]>,
    },
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Deserialize, Serialize, phalcom_type_meta_derive::BudgetDecode)]
pub enum KindNode {
    Type,
    RecordRow,
    TypePack,
    LabeledTypePack,
    CompleteTypePack,
    StaticDomain(StaticDomainRef),
    Arrow { parameters: Box<[KindNodeId]>, result: KindNodeId },
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize, phalcom_type_meta_derive::BudgetDecode)]
pub struct KindNodeEntry {
    pub node: KindNode,
    pub structural_fingerprint: Fingerprint128,
}
