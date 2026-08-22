//! Canonical global type-term graph.

use crate::fingerprint::Fingerprint128;
use crate::generic::StableTypeParameterRef;
use crate::identity::{StableDeclarationRef, StableDispatchSide};
use crate::kind::KindNodeId;
use crate::scoped_type::TypeLambdaRef;
use serde::{Deserialize, Serialize};

#[repr(transparent)]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
pub struct TypeNodeId(pub u32);

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
pub enum SelfRoleRef {
    InstanceType,
    ReceiverValue,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Deserialize, Serialize)]
pub struct SelfTypeRef {
    pub owner: StableDeclarationRef,
    pub side: StableDispatchSide,
    pub role: SelfRoleRef,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Deserialize, Serialize)]
pub struct TupleElementRef {
    pub label: Option<Box<str>>,
    pub ty: TypeNodeId,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Deserialize, Serialize)]
pub struct RecordFieldRef {
    pub name: Box<str>,
    pub ty: TypeNodeId,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Deserialize, Serialize)]
pub struct CallableParamRef {
    pub label: Option<Box<str>>,
    pub ty: TypeNodeId,
    pub rest: bool,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Deserialize, Serialize)]
pub struct CallableTypeRef {
    pub parameters: Box<[CallableParamRef]>,
    pub return_type: TypeNodeId,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Deserialize, Serialize)]
pub struct OpenRecordTypeRef {
    pub fields: Box<[RecordFieldRef]>,
    pub tail: StableTypeParameterRef,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Deserialize, Serialize)]
pub enum TypeNode {
    Never,
    Unit,
    Nominal { declaration: StableDeclarationRef },
    Applied { origin: TypeNodeId, arguments: Box<[TypeNodeId]> },
    Union(Box<[TypeNodeId]>),
    Tuple(Box<[TupleElementRef]>),
    Record(Box<[RecordFieldRef]>),
    Callable(CallableTypeRef),
    Parameter(StableTypeParameterRef),
    SelfType(SelfTypeRef),
    TypeLambda(TypeLambdaRef),
    OpenRecord(OpenRecordTypeRef),
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct TypeNodeEntry {
    pub kind: KindNodeId,
    pub form: TypeNode,
    pub structural_fingerprint: Fingerprint128,
}
