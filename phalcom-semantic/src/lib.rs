//! `phalcom-semantic` — Canonical static semantic analysis, type store, subtyping relations,
//! and verification engine for Phalcom.

pub mod checker;
pub mod declarations;
pub mod diagnostic;
pub mod dispatch;
pub mod identity;
pub mod invalidation;
pub mod resolver;
pub mod scope;
pub mod snapshot;
pub mod source;
pub mod surface;
pub mod types;
pub mod workspace;

pub use checker::{
    CheckingContext, TypeCheckReport, TypedExpression, check_arguments, check_class,
    check_class_bodies, check_statement, match_callable_arguments, register_class_surface,
    synthesize_expr, synthesize_typed_expr,
};
pub use declarations::{
    DeclarationTypeInfo, DeclarationTypeTable, bootstrap_universe_declarations, lower_kind_spec,
};
pub use diagnostic::{DiagnosticCode, DiagnosticLabel, DiagnosticSeverity, SemanticDiagnostic};
pub use dispatch::{
    CallableParameter, CallableSignature, DispatchResolver, DispatchResult, DispatchSide,
    DispatchTarget, SurfaceDispatchResolver,
};
pub use identity::{BindingId, CallableId, DeclarationId, FieldId, ModuleId};
pub use invalidation::{DeclarationFingerprint, InvalidationIndex};
pub use resolver::LinkedTypeResolver;
pub use scope::ScopeTable;
pub use snapshot::SemanticSnapshot;
pub use source::ParsedSourceUnit;
pub use surface::DeclarationSurface;
pub use types::{
    Assignability, CallableParameterType, CallableType, ConstraintSet, DynamicReason,
    EvidenceAuthority, EvidenceSet, GenericSignature, InferVarId, KindData, KindId,
    LocalConstraintSolver, MapTypeHierarchy, NativeTypeResolutionError, RecordTypeField,
    RefutationReason, SemanticDenotation, SimpleTypeResolver, TupleTypeElement,
    TypeApplicationError, TypeConstraint, TypeData, TypeEvidence, TypeHierarchy, TypeId,
    TypeKnowledge, TypeParameterData, TypeParameterId, TypeParameterOwner, TypeResolver,
    TypeStore, UnknownReason, ValueSemanticFact, check_assignability, is_subtype,
    normalize_native_type, register_standard_surfaces, resolve_native_type_form,
    resolve_type_annotation, resolve_type_form, substitution_for_applied, TypeSubstitution,
};
pub use workspace::{
    SemanticAnalysis, SemanticWorkspaceInput, analyze_single_module, analyze_workspace,
};
