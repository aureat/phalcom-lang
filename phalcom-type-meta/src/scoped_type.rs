//! Scoped node graph for alpha-normalized type lambda bodies.

use crate::field_key::FieldKeyRef;
use crate::fingerprint::Fingerprint128;
use crate::kind::KindNodeId;
use crate::type_node::TypeNodeId;
use serde::{Deserialize, Serialize};

use crate::generic::{GenericParameterModeRef, StableTypeParameterRef};
use crate::type_node::StructuralMemberPresenceRef;

#[repr(transparent)]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize, phalcom_type_meta_derive::BudgetDecode)]
pub struct ScopedTypeNodeId(pub u32);

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize, phalcom_type_meta_derive::BudgetDecode)]
pub enum ScopedAnonymousTupleRemainderRef {
    #[default]
    None,
    Unconstrained,
    Typed(ScopedTypeNodeId),
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize, phalcom_type_meta_derive::BudgetDecode)]
pub enum ScopedAnonymousRecordRemainderRef {
    Unconstrained,
    Typed(ScopedTypeNodeId),
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Deserialize, Serialize, phalcom_type_meta_derive::BudgetDecode)]
pub struct ScopedTupleElementRef {
    pub label: Option<Box<str>>,
    pub ty: ScopedTypeNodeId,
    #[serde(default)]
    pub presence: StructuralMemberPresenceRef,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Deserialize, Serialize, phalcom_type_meta_derive::BudgetDecode)]
pub struct ScopedOpenTupleTypeRef {
    pub elements: Box<[ScopedTupleElementRef]>,
    #[serde(default)]
    pub positional_remainder: ScopedAnonymousTupleRemainderRef,
    #[serde(default)]
    pub labeled_remainder: ScopedAnonymousTupleRemainderRef,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Deserialize, Serialize, phalcom_type_meta_derive::BudgetDecode)]
pub struct ScopedPackTypeRef {
    pub elements: Box<[ScopedTupleElementRef]>,
    #[serde(default)]
    pub positional_remainder: ScopedAnonymousTupleRemainderRef,
    #[serde(default)]
    pub labeled_remainder: ScopedAnonymousTupleRemainderRef,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Deserialize, Serialize, phalcom_type_meta_derive::BudgetDecode)]
pub struct ScopedAnonymousRecordTypeRef {
    pub fields: Box<[ScopedRecordFieldRef]>,
    pub remainder: ScopedAnonymousRecordRemainderRef,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Deserialize, Serialize, phalcom_type_meta_derive::BudgetDecode)]
pub struct ScopedLabeledTypePackElementRef {
    pub label: Box<str>,
    pub element: ScopedTypeNodeId,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Deserialize, Serialize, phalcom_type_meta_derive::BudgetDecode)]
pub enum ScopedPackProjectionLaneRef {
    Positionals,
    Labeled,
    Complete,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize)]
pub struct ScopedRecordFieldRef {
    pub key: FieldKeyRef,
    pub ty: ScopedTypeNodeId,
    #[serde(default)]
    pub presence: StructuralMemberPresenceRef,
}

impl crate::budgeted::BudgetDecode for ScopedRecordFieldRef {
    fn decode(json: &mut crate::budgeted::Decoder<'_, '_>) -> Result<Self, crate::budgeted::DecodeError> {
        json.nested(|json| {
            let mut key: Option<FieldKeyRef> = None;
            let mut name: Option<Box<str>> = None;
            let mut ty: Option<ScopedTypeNodeId> = None;
            let mut presence: Option<StructuralMemberPresenceRef> = None;
            json.cursor.expect(b'{')?;
            let mut first = true;
            while let Some(k) = json.cursor.object_key(&mut first)? {
                if k.equals("key") {
                    key = Some(<FieldKeyRef as crate::budgeted::BudgetDecode>::decode(json)?);
                    continue;
                }
                if k.equals("name") {
                    name = Some(<Box<str> as crate::budgeted::BudgetDecode>::decode(json)?);
                    continue;
                }
                if k.equals("ty") {
                    ty = Some(<ScopedTypeNodeId as crate::budgeted::BudgetDecode>::decode(json)?);
                    continue;
                }
                if k.equals("presence") {
                    presence = Some(<StructuralMemberPresenceRef as crate::budgeted::BudgetDecode>::decode(json)?);
                    continue;
                }
                json.cursor.skip_value()?;
            }
            let key = match (key, name) {
                (Some(k), _) => k,
                (None, Some(name)) => FieldKeyRef::Symbol(name),
                (None, None) => return Err(crate::budgeted::DecodeError::Syntax("missing metadata field key")),
            };
            let ty = ty.ok_or(crate::budgeted::DecodeError::Syntax("missing metadata field ty"))?;
            let presence = presence.unwrap_or_default();
            Ok(ScopedRecordFieldRef { key, ty, presence })
        })
    }
}

impl<'de> Deserialize<'de> for ScopedRecordFieldRef {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        struct RawField {
            key: Option<FieldKeyRef>,
            name: Option<Box<str>>,
            ty: ScopedTypeNodeId,
            #[serde(default)]
            presence: StructuralMemberPresenceRef,
        }
        let raw = RawField::deserialize(deserializer)?;
        let key = match (raw.key, raw.name) {
            (Some(k), _) => k,
            (None, Some(name)) => FieldKeyRef::Symbol(name),
            (None, None) => return Err(serde::de::Error::missing_field("key")),
        };
        Ok(Self {
            key,
            ty: raw.ty,
            presence: raw.presence,
        })
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Deserialize, Serialize, phalcom_type_meta_derive::BudgetDecode)]
pub enum ScopedRecordTailRef {
    Bound { depth: u32, index: u32 },
    FreeParameter(StableTypeParameterRef),
    Anonymous(ScopedAnonymousRecordRemainderRef),
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Deserialize, Serialize, phalcom_type_meta_derive::BudgetDecode)]
pub struct ScopedOpenRecordTypeRef {
    pub fields: Box<[ScopedRecordFieldRef]>,
    pub tail: ScopedRecordTailRef,
}

/// Tail identity for a row-valued argument inside a scoped type graph.
#[derive(Clone, Debug, Eq, Hash, PartialEq, Deserialize, Serialize, phalcom_type_meta_derive::BudgetDecode)]
pub enum ScopedRecordRowTailRef {
    Closed,
    Bound { depth: u32, index: u32 },
    FreeParameter(StableTypeParameterRef),
    Projection(TypeNodeId),
    Anonymous(ScopedAnonymousRecordRemainderRef),
}

/// A heterogeneous `RecordFields` argument retained in a scoped application.
#[derive(Clone, Debug, Eq, Hash, PartialEq, Deserialize, Serialize, phalcom_type_meta_derive::BudgetDecode)]
pub struct ScopedRecordRowTypeRef {
    pub fields: Box<[ScopedRecordFieldRef]>,
    pub tail: ScopedRecordRowTailRef,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Deserialize, Serialize, phalcom_type_meta_derive::BudgetDecode)]
pub struct ScopedCallableParamRef {
    pub label: Option<Box<str>>,
    pub ty: ScopedTypeNodeId,
    pub rest: bool,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Deserialize, Serialize, phalcom_type_meta_derive::BudgetDecode)]
pub struct ScopedCallableTypeRef {
    pub parameters: Box<[ScopedCallableParamRef]>,
    pub return_type: ScopedTypeNodeId,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Deserialize, Serialize, phalcom_type_meta_derive::BudgetDecode)]
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

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize, phalcom_type_meta_derive::BudgetDecode)]
pub struct ScopedTypeNodeEntry {
    pub kind: KindNodeId,
    pub form: ScopedTypeNode,
    pub structural_fingerprint: Fingerprint128,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Deserialize, Serialize, phalcom_type_meta_derive::BudgetDecode)]
pub struct TypeLambdaRef {
    pub parameter_kinds: Box<[KindNodeId]>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parameter_modes: Option<Box<[GenericParameterModeRef]>>,
    pub body: ScopedTypeNodeId,
}
