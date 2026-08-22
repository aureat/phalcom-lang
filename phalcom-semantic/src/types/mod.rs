//! Canonical static type system layer for Phalcom.

pub mod annotation;
pub mod evidence;
pub mod id;
pub mod kind;
pub mod native;
pub mod relation;
pub mod store;

pub use annotation::{resolve_type_annotation, SimpleTypeResolver, TypeResolver};
pub use evidence::{
    DynamicReason, EvidenceAuthority, EvidenceSet, TypeEvidence, TypeKnowledge, UnknownReason,
};
pub use id::{InferVarId, KindId, TypeId, TypeParameterId};
pub use kind::KindData;
pub use native::normalize_native_type;
pub use relation::{
    check_assignability, is_subtype, Assignability, MapTypeHierarchy, RefutationReason,
    TypeHierarchy,
};
pub use store::{
    CallableParameterType, CallableType, RecordTypeField, TupleTypeElement, TypeData, TypeStore,
};
