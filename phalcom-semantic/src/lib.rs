//! `phalcom-semantic` — Canonical static semantic analysis, type store, subtyping relations,
//! and verification engine for Phalcom.

pub mod checker;
pub mod diagnostic;
pub mod dispatch;
pub mod identity;
pub mod invalidation;
pub mod scope;
pub mod snapshot;
pub mod source;
pub mod surface;
pub mod types;

pub use checker::{check_program, CheckingContext, TypeCheckReport};
pub use diagnostic::{DiagnosticCode, DiagnosticLabel, DiagnosticSeverity, SemanticDiagnostic};
pub use dispatch::{DispatchSide, DispatchTarget};
pub use identity::{BindingId, CallableId, DeclarationId, FieldId, ModuleId};
pub use invalidation::{DeclarationFingerprint, InvalidationIndex};
pub use scope::ScopeTable;
pub use snapshot::SemanticSnapshot;
pub use source::ParsedSourceUnit;
pub use surface::DeclarationSurface;
pub use types::{
    check_assignability, is_subtype, normalize_native_type, resolve_type_annotation, Assignability,
    CallableParameterType, CallableType, DynamicReason, EvidenceAuthority, EvidenceSet, InferVarId,
    KindData, KindId, MapTypeHierarchy, RecordTypeField, RefutationReason, SimpleTypeResolver,
    TupleTypeElement, TypeData, TypeEvidence, TypeHierarchy, TypeId, TypeKnowledge, TypeParameterId,
    TypeResolver, TypeStore, UnknownReason,
};
