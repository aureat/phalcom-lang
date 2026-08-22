//! Scoped node graph for alpha-normalized type lambda bodies.

use crate::fingerprint::Fingerprint128;
use crate::kind::KindNodeId;
use crate::type_node::TypeNodeId;
use serde::{Deserialize, Serialize};

use crate::generic::StableTypeParameterRef;

#[repr(transparent)]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
pub struct ScopedTypeNodeId(pub u32);

#[derive(Clone, Debug, Eq, Hash, PartialEq, Deserialize, Serialize)]
pub struct ScopedTupleElementRef {
    pub label: Option<Box<str>>,
    pub ty: ScopedTypeNodeId,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Deserialize, Serialize)]
pub struct ScopedRecordFieldRef {
    pub name: Box<str>,
    pub ty: ScopedTypeNodeId,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Deserialize, Serialize)]
pub enum ScopedRecordTailRef {
    Bound { depth: u32, index: u32 },
    FreeParameter(StableTypeParameterRef),
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Deserialize, Serialize)]
pub struct ScopedOpenRecordTypeRef {
    pub fields: Box<[ScopedRecordFieldRef]>,
    pub tail: ScopedRecordTailRef,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Deserialize, Serialize)]
pub struct ScopedCallableParamRef {
    pub label: Option<Box<str>>,
    pub ty: ScopedTypeNodeId,
    pub rest: bool,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Deserialize, Serialize)]
pub struct ScopedCallableTypeRef {
    pub parameters: Box<[ScopedCallableParamRef]>,
    pub return_type: ScopedTypeNodeId,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Deserialize, Serialize)]
pub enum ScopedTypeNode {
    /// Lambda-bound variable; depth 0 is the innermost binder.
    Bound {
        depth: u32,
        index: u32,
    },
    /// Canonical free term from the enclosing global type graph.
    Free(TypeNodeId),
    Applied {
        origin: ScopedTypeNodeId,
        arguments: Box<[ScopedTypeNodeId]>,
    },
    Union(Box<[ScopedTypeNodeId]>),
    Tuple(Box<[ScopedTupleElementRef]>),
    Record(Box<[ScopedRecordFieldRef]>),
    Callable(ScopedCallableTypeRef),
    /// Nested lambda.
    Lambda {
        parameter_kinds: Box<[KindNodeId]>,
        body: ScopedTypeNodeId,
    },
    OpenRecord(ScopedOpenRecordTypeRef),
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct ScopedTypeNodeEntry {
    pub kind: KindNodeId,
    pub form: ScopedTypeNode,
    pub structural_fingerprint: Fingerprint128,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Deserialize, Serialize)]
pub struct TypeLambdaRef {
    pub parameter_kinds: Box<[KindNodeId]>,
    pub body: ScopedTypeNodeId,
}
