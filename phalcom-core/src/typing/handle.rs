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
pub enum RuntimeSemanticHandle {
    Type(RuntimeTypeRef),
    Kind(RuntimeKindRef),
}
