//! Canonical global type-term graph.

use crate::fingerprint::Fingerprint128;
use crate::generic::{StableAssociatedTypeRequirementRef, StableTraitApplicationRef, StableTypeParameterRef};
use crate::identity::{StableDeclarationRef, StableDispatchSide};
use crate::kind::{KindNodeId, StaticDomainRef};
use crate::scalar::StaticInt;
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

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
pub enum StructuralMemberPresenceRef {
    #[default]
    Required,
    Optional,
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
pub enum AnonymousTupleRemainderRef {
    #[default]
    None,
    Unconstrained,
    Typed(TypeNodeId),
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
pub enum AnonymousRecordRemainderRef {
    Unconstrained,
    Typed(TypeNodeId),
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Deserialize, Serialize)]
pub struct TupleElementRef {
    pub label: Option<Box<str>>,
    pub ty: TypeNodeId,
    #[serde(default)]
    pub presence: StructuralMemberPresenceRef,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Deserialize, Serialize)]
pub struct RecordFieldRef {
    pub name: Box<str>,
    pub ty: TypeNodeId,
    #[serde(default)]
    pub presence: StructuralMemberPresenceRef,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Deserialize, Serialize)]
pub struct OpenTupleTypeRef {
    pub elements: Box<[TupleElementRef]>,
    #[serde(default)]
    pub positional_remainder: AnonymousTupleRemainderRef,
    #[serde(default)]
    pub labeled_remainder: AnonymousTupleRemainderRef,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Deserialize, Serialize)]
pub struct AnonymousRecordTypeRef {
    pub fields: Box<[RecordFieldRef]>,
    pub remainder: AnonymousRecordRemainderRef,
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
pub struct LabeledTypePackElementRef {
    pub label: Box<str>,
    pub element: TypeNodeId,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Deserialize, Serialize)]
pub enum TypeMemberSelectorRef {
    Named(Box<str>),
    Subscript,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Deserialize, Serialize)]
pub enum TypeMemberArgumentRef {
    Type(TypeNodeId),
    ConstInt(StaticInt),
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Deserialize, Serialize)]
pub struct TypeMemberProjectionRef {
    pub subject: TypeNodeId,
    pub selector: TypeMemberSelectorRef,
    pub arguments: Box<[TypeMemberArgumentRef]>,
    /// Stable source member identity for source-selected projections. New
    /// public source-authored intrinsic projections retain this field; `None`
    /// remains accepted for older metadata and internal synthetic projections.
    #[serde(default)]
    pub source_declaration: Option<crate::identity::StableTypeMemberRef>,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Deserialize, Serialize)]
pub struct StaticConstructorRef {
    pub declaration: StableDeclarationRef,
    pub case_name: Box<str>,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Deserialize, Serialize)]
pub enum StaticArgumentRef {
    Term(TypeNodeId),
    RecordRow(RecordRowRef),
    DynamicType,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Deserialize, Serialize)]
pub struct RecordRowRef {
    pub fields: Box<[RecordFieldRef]>,
    #[serde(default)]
    pub tail: Option<crate::generic::StableTypeParameterRef>,
    #[serde(default)]
    pub remainder: Option<AnonymousRecordRemainderRef>,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Deserialize, Serialize)]
pub struct StaticDataRef {
    pub domain: StaticDomainRef,
    pub constructor: StaticConstructorRef,
    pub arguments: Box<[StaticArgumentRef]>,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Deserialize, Serialize)]
pub struct PackTypeRef {
    pub elements: Box<[TupleElementRef]>,
    #[serde(default)]
    pub positional_remainder: AnonymousTupleRemainderRef,
    #[serde(default)]
    pub labeled_remainder: AnonymousTupleRemainderRef,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Deserialize, Serialize)]
pub enum TypeNode {
    Never,
    Unit,
    ConstInt(StaticInt),
    StaticData(StaticDataRef),
    Nominal {
        declaration: StableDeclarationRef,
    },
    Applied {
        origin: TypeNodeId,
        arguments: Box<[StaticArgumentRef]>,
    },
    Union(Box<[TypeNodeId]>),
    Tuple(Box<[TupleElementRef]>),
    Record(Box<[RecordFieldRef]>),
    Callable(CallableTypeRef),
    Parameter(StableTypeParameterRef),
    SelfType(SelfTypeRef),
    TypeLambda(TypeLambdaRef),
    OpenRecord(OpenRecordTypeRef),
    AssociatedProjection {
        subject: TypeNodeId,
        trait_application: StableTraitApplicationRef,
        requirement: StableAssociatedTypeRequirementRef,
    },
    TypePack(Box<[TypeNodeId]>),
    LabeledTypePack(Box<[LabeledTypePackElementRef]>),
    CompleteTypePack {
        positionals: TypeNodeId,
        labeled: TypeNodeId,
    },
    TypePackConcat(Box<[TypeNodeId]>),
    LabeledTypePackConcat(Box<[TypeNodeId]>),
    TupleFromPacks {
        positionals: TypeNodeId,
        labeled: TypeNodeId,
    },
    CallableFromPacks {
        positionals: TypeNodeId,
        labeled: TypeNodeId,
        return_type: TypeNodeId,
    },
    TypeMemberProjection(TypeMemberProjectionRef),
    OpenTuple(OpenTupleTypeRef),
    AnonymousRecord(AnonymousRecordTypeRef),
    Pack(PackTypeRef),
    PackFromPacks {
        positionals: TypeNodeId,
        labeled: TypeNodeId,
    },
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct TypeNodeEntry {
    pub kind: KindNodeId,
    pub form: TypeNode,
    pub structural_fingerprint: Fingerprint128,
}
