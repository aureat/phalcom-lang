//! Canonical static type system layer for Phalcom.

pub mod annotation;
pub mod application;
pub mod constraint;
pub mod evidence;
pub mod id;
pub mod kind;
pub mod native;
pub mod parameter;
pub mod relation;
pub mod store;

pub use annotation::{SimpleTypeResolver, TypeResolver, resolve_type_annotation};
pub use application::TypeApplicationError;
pub use constraint::{ConstraintSet, LocalConstraintSolver, TypeConstraint};
pub use evidence::{
    DynamicReason, EvidenceAuthority, EvidenceSet, TypeEvidence, TypeKnowledge, UnknownReason,
};
pub use id::{InferVarId, KindId, TypeId, TypeParameterId};
pub use kind::{KindApplicationError, KindData};
pub use native::{
    NativeTypeResolutionError, normalize_native_type, register_standard_surfaces,
    resolve_native_type_form,
};
pub use parameter::{GenericSignature, TypeParameterData, TypeParameterOwner};
pub use relation::{
    Assignability, MapTypeHierarchy, RefutationReason, TypeHierarchy, check_assignability,
    is_subtype,
};
pub use store::{
    CallableParameterType, CallableType, RecordTypeField, TupleTypeElement, TypeData, TypeStore,
};
