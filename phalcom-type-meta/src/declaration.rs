//! Declaration records, superclasses, aliases, callables, and fields.

use crate::generic::GenericSignatureRecordId;
use crate::identity::{SourceSpanRef, StableCallableRef, StableDeclarationRef, StableFieldRef};
use crate::kind::KindNodeId;
use crate::type_node::TypeNodeId;
use serde::{Deserialize, Serialize};

#[repr(transparent)]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
pub struct DeclarationRecordId(pub u32);

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq, Deserialize, Serialize)]
pub struct DeclarationTypeFlags {
    pub is_abstract: bool,
    pub is_final: bool,
    pub is_trait: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct DeclarationTypeRecord {
    pub declaration: StableDeclarationRef,
    pub form: TypeNodeId,
    pub kind: KindNodeId,
    pub generic_signature: Option<GenericSignatureRecordId>,
    pub superclass_template: Option<TypeNodeId>,
    pub instance_callables: Box<[StableCallableRef]>,
    pub class_callables: Box<[StableCallableRef]>,
    pub instance_fields: Box<[StableFieldRef]>,
    pub class_fields: Box<[StableFieldRef]>,
    pub flags: DeclarationTypeFlags,
    pub source: Option<SourceSpanRef>,
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
pub struct AliasRecordId(pub u32);

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct TypeAliasRecord {
    pub declaration: StableDeclarationRef,
    pub generic_signature: Option<GenericSignatureRecordId>,
    pub target: TypeNodeId,
    pub source: Option<SourceSpanRef>,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
pub enum PublishedTypeAuthority {
    DeclaredAnnotation,
    TrustedNative,
    GeneratedDeclaration,
    CompilerInferred,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
pub enum DynamicReasonRef {
    ExplicitEscape,
    UncheckedBoundary,
    UnsupportedNative,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
pub enum UnknownReasonRef {
    UnannotatedDeclaration,
    InferenceFailed,
    OpaqueNative,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
pub enum MetadataUnavailableReason {
    StrippedByProfile,
    UnloadedModule,
    IncompatibleModel,
    DynamicReplacement,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub enum PublishedTypeSlot {
    Known { form: TypeNodeId, authority: PublishedTypeAuthority },
    Dynamic { reason: DynamicReasonRef },
    Unknown { reason: UnknownReasonRef },
    Unavailable { reason: MetadataUnavailableReason },
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
pub struct CallableRecordId(pub u32);

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
pub enum RestModeRef {
    None,
    Anonymous,
    Named,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct CallableParameterRecord {
    pub index: u32,
    pub local_name: Box<str>,
    pub external_label: Option<Box<str>>,
    pub rest: RestModeRef,
    pub ty: PublishedTypeSlot,
    pub source: Option<SourceSpanRef>,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct CallableSemanticRecord {
    pub callable: StableCallableRef,
    pub generic_signature: Option<GenericSignatureRecordId>,
    pub parameters: Box<[CallableParameterRecord]>,
    pub return_type: PublishedTypeSlot,
    pub source: Option<SourceSpanRef>,
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
pub struct FieldRecordId(pub u32);

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
pub enum FieldMutabilityRef {
    Immutable,
    Mutable,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct FieldSemanticRecord {
    pub field: StableFieldRef,
    pub mutability: FieldMutabilityRef,
    pub ty: PublishedTypeSlot,
    pub source: Option<SourceSpanRef>,
}
