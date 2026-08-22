//! Compact runtime semantic handles.

use phalcom_type_meta::kind::KindNodeId;
use phalcom_type_meta::type_node::TypeNodeId;

#[repr(transparent)]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct MetadataPoolId(pub u32);

#[repr(transparent)]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct RuntimeOverlayTypeId(pub u32);

#[repr(transparent)]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct RuntimeOverlayKindId(pub u32);

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum RuntimeTypeRef {
    Base { pool: MetadataPoolId, node: TypeNodeId },
    Overlay(RuntimeOverlayTypeId),
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum RuntimeKindRef {
    Base { pool: MetadataPoolId, node: KindNodeId },
    Overlay(RuntimeOverlayKindId),
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct RuntimeTypeParameterRef {
    pub pool: MetadataPoolId,
    pub index: u32,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct RuntimeGenericSignatureRef {
    pub pool: MetadataPoolId,
    pub id: phalcom_type_meta::generic::GenericSignatureRecordId,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct RuntimeGenericConstraintRef {
    pub pool: MetadataPoolId,
    pub signature: phalcom_type_meta::generic::GenericSignatureRecordId,
    pub index: u32,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct RuntimeCallableSignatureRef {
    pub pool: MetadataPoolId,
    pub record: phalcom_type_meta::declaration::CallableRecordId,
    pub specialization_receiver: Option<RuntimeTypeRef>,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct RuntimeCallableParameterRef {
    pub callable: RuntimeCallableSignatureRef,
    pub index: u32,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct RuntimeFieldSignatureRef {
    pub pool: MetadataPoolId,
    pub record: phalcom_type_meta::declaration::FieldRecordId,
    pub specialization_receiver: Option<RuntimeTypeRef>,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct RuntimeTypeUseRef {
    pub pool: MetadataPoolId,
    pub index: u32,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum RuntimeSemanticHandle {
    Type(RuntimeTypeRef),
    Kind(RuntimeKindRef),
    TypeParameter(RuntimeTypeParameterRef),
    GenericSignature(RuntimeGenericSignatureRef),
    GenericConstraint(RuntimeGenericConstraintRef),
    CallableSignature(RuntimeCallableSignatureRef),
    CallableParameter(RuntimeCallableParameterRef),
    FieldSignature(RuntimeFieldSignatureRef),
    TypeUse(RuntimeTypeUseRef),
}
