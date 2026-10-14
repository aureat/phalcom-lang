//! Scoped node graph for alpha-normalized type lambda bodies.

use crate::fingerprint::Fingerprint128;
use crate::kind::KindNodeId;
use crate::type_node::TypeNodeId;
use serde::{Deserialize, Serialize};

use crate::generic::{GenericParameterModeRef, StableTypeParameterRef};
use crate::type_node::{RecordKeyRef, StructuralMemberPresenceRef};

#[repr(transparent)]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
pub struct ScopedTypeNodeId(pub u32);

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
pub enum ScopedAnonymousTupleRemainderRef {
    #[default]
    None,
    Unconstrained,
    Typed(ScopedTypeNodeId),
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
pub enum ScopedAnonymousRecordRemainderRef {
    Unconstrained { key_type: ScopedTypeNodeId },
    Typed { key_type: ScopedTypeNodeId, value_type: ScopedTypeNodeId },
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Deserialize, Serialize)]
pub struct ScopedTupleElementRef {
    pub label: Option<Box<str>>,
    pub ty: ScopedTypeNodeId,
    #[serde(default)]
    pub presence: StructuralMemberPresenceRef,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Deserialize, Serialize)]
pub struct ScopedOpenTupleTypeRef {
    pub elements: Box<[ScopedTupleElementRef]>,
    #[serde(default)]
    pub positional_remainder: ScopedAnonymousTupleRemainderRef,
    #[serde(default)]
    pub labeled_remainder: ScopedAnonymousTupleRemainderRef,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Deserialize, Serialize)]
pub struct ScopedPackTypeRef {
    pub elements: Box<[ScopedTupleElementRef]>,
    #[serde(default)]
    pub positional_remainder: ScopedAnonymousTupleRemainderRef,
    #[serde(default)]
    pub labeled_remainder: ScopedAnonymousTupleRemainderRef,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Deserialize, Serialize)]
pub struct ScopedAnonymousRecordTypeRef {
    pub fields: Box<[ScopedRecordFieldRef]>,
    pub remainder: ScopedAnonymousRecordRemainderRef,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Deserialize, Serialize)]
pub struct ScopedLabeledTypePackElementRef {
    pub label: Box<str>,
    pub element: ScopedTypeNodeId,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Deserialize, Serialize)]
pub enum ScopedPackProjectionLaneRef {
    Positionals,
    Labeled,
    Complete,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Deserialize, Serialize)]
pub struct ScopedRecordFieldRef {
    pub key: RecordKeyRef,
    pub ty: ScopedTypeNodeId,
    #[serde(default)]
    pub presence: StructuralMemberPresenceRef,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Deserialize, Serialize)]
pub enum ScopedRecordTailRef {
    Bound { depth: u32, index: u32 },
    FreeParameter(StableTypeParameterRef),
    Anonymous(ScopedAnonymousRecordRemainderRef),
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Deserialize, Serialize)]
pub struct ScopedOpenRecordTypeRef {
    pub fields: Box<[ScopedRecordFieldRef]>,
    pub tail: ScopedRecordTailRef,
}

/// Tail identity for a row-valued argument inside a scoped type graph.
#[derive(Clone, Debug, Eq, Hash, PartialEq, Deserialize, Serialize)]
pub enum ScopedRecordRowTailRef {
    Closed,
    Bound { depth: u32, index: u32 },
    FreeParameter(StableTypeParameterRef),
    Projection(TypeNodeId),
    Anonymous(ScopedAnonymousRecordRemainderRef),
}

/// A heterogeneous `RecordFields` argument retained in a scoped application.
#[derive(Clone, Debug, Eq, Hash, PartialEq, Deserialize, Serialize)]
pub struct ScopedRecordRowTypeRef {
    pub fields: Box<[ScopedRecordFieldRef]>,
    pub tail: ScopedRecordRowTailRef,
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
        #[serde(default, skip_serializing_if = "Option::is_none")]
        parameter_modes: Option<Box<[GenericParameterModeRef]>>,
        body: ScopedTypeNodeId,
    },
    OpenRecord(ScopedOpenRecordTypeRef),
    /// Row-valued `RecordFields` argument. This is distinct from a proper
    /// structural `Record` or an open structural record type.
    RecordRow(ScopedRecordRowTypeRef),
    TypePack(Box<[ScopedTypeNodeId]>),
    LabeledTypePack(Box<[ScopedLabeledTypePackElementRef]>),
    CompleteTypePack {
        positionals: ScopedTypeNodeId,
        labeled: ScopedTypeNodeId,
    },
    TypePackConcat(Box<[ScopedTypeNodeId]>),
    LabeledTypePackConcat(Box<[ScopedTypeNodeId]>),
    TupleFromPacks {
        positionals: ScopedTypeNodeId,
        labeled: ScopedTypeNodeId,
    },
    CallableFromPacks {
        positionals: ScopedTypeNodeId,
        labeled: ScopedTypeNodeId,
        return_type: ScopedTypeNodeId,
    },
    PackProjection {
        subject: ScopedTypeNodeId,
        lane: ScopedPackProjectionLaneRef,
    },
    OpenTuple(ScopedOpenTupleTypeRef),
    AnonymousRecord(ScopedAnonymousRecordTypeRef),
    Pack(ScopedPackTypeRef),
    PackFromPacks {
        positionals: ScopedTypeNodeId,
        labeled: ScopedTypeNodeId,
    },
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parameter_modes: Option<Box<[GenericParameterModeRef]>>,
    pub body: ScopedTypeNodeId,
}
