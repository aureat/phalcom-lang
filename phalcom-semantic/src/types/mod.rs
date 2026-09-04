//! Canonical static type system layer for Phalcom.

pub mod annotation;
pub mod application;
pub mod case_environment;
pub mod case_instantiation;
pub mod constraint;
pub mod denotation;
pub mod environment;
pub mod evidence;
pub mod family;
pub mod id;
pub mod instantiation;
pub mod kind;
pub mod native;
pub mod outcome;
pub mod parameter;
pub mod relation;
pub mod rigid;
pub mod row;
pub mod row_solver;
pub mod specialization;
pub mod store;
pub mod substitution;
pub mod type_lambda;
pub mod variance;

pub use annotation::{
    KindResolution, SimpleTypeResolver, TypeFormResolution, TypeFormationInvalid, TypeFormationMissing, TypeFormationOutcome, TypeFormationUnresolved,
    TypeLevelBinding, TypeResolver, resolve_type_annotation, resolve_type_form, type_level_binding_for_parameter,
};
pub use application::TypeApplicationError;
pub use case_environment::{CaseEnvironmentError, CaseTypeEnvironment, derive_case_environment};
pub use case_instantiation::CaseInstantiation;
pub use constraint::{ConstraintSet, TypeConstraint};
pub use denotation::{AssociatedValueDenotation, CapturedAssociatedMember, SemanticDenotation, ValueSemanticFact};
pub use environment::{SpecializedCallableView, SpecializedMemberView, TypeEnvironment, TypeView};
pub use evidence::{
    ContractAssumptionEligibility, DynamicReason, EvidenceOrigin, EvidenceSet, EvidenceStatus, TypeEvidence, TypeKnowledge, UnknownReason, join_type_knowledge,
};
pub use family::{FamilyMemberType, FamilyMemberTypeKind, FamilyOperationShape, FamilyType, FamilyTypeError, FamilyTypeId};
pub use id::{
    InferVarId, KindId, ProperTypeId, RecordRowId, RigidScopeId, RigidTypeVariableId, ScopedTypeId, TypeId, TypeLambdaId, TypeParameterId, TypeStoreId,
    VariantTypeId,
};
pub use instantiation::{GenericInstantiation, RowMaterializationMode, TypeMaterializationError, materialize_type};
pub use kind::{KindApplicationError, KindData};
pub use native::{
    NativeSurfaceImportError, NativeSurfaceImportReport, NativeTypeResolutionError, normalize_native_type, register_native_surfaces,
    register_native_surfaces_from_records, resolve_native_type_form,
};
pub use outcome::{
    BlockReason, BudgetKind, BudgetReport, CancellationToken, DynamicBoundaryObligation, QueryBudget, RelationEvidence, RelationFailure, RelationOutcome,
};
pub use parameter::{GenericConstraint, GenericSignature, SelfRole, SelfTypeTerm, TypeParameterData, TypeParameterOwner, TypeTerm};
pub use relation::{
    Assignability, MapTypeHierarchy, RefutationReason, TypeHierarchy, check_assignability, check_assignability_bounded, check_knowledge_against_type,
    check_knowledge_against_type_bounded, check_subtype_bounded, is_subtype,
};
pub use rigid::{LocalConstraint, LocalType, RigidArena, RigidMaterializationError, RigidOrigin, RigidSubstitution, RigidTypeVariable};
pub use row::{DuplicateFieldError, RecordRowData, RecordRowField, RecordRowFormationError, RecordRowTail};
pub use row_solver::{
    IncidentId, RecordRowBlockedReason, RecordRowFailure, RecordRowLacks, RecordRowSolution, RecordRowSolveResult, RecordRowSolver, RecordRowTerm,
    RecordRowTermTail, RecordRowUnderconstrained, RecordRowVarId, RecordRowZonkError,
};
pub use specialization::{
    ReceiverSpecialization, ReceiverSpecializationFailure, ReceiverSpecializationStep, SpecializationControl, specialize_receiver_to_owner,
};
pub use store::{CallableParameterType, CallableType, RecordTypeField, TupleTypeElement, TypeData, TypeStore};
pub use substitution::{TypeSubstitution, substitution_for_applied};
pub use type_lambda::{
    BetaReductionError, BetaResult, ScopedCallableParameter, ScopedCallableType, ScopedOpenRecord, ScopedRecordField, ScopedRecordTail, ScopedTupleElement,
    ScopedTypeData, TypeLambdaArena, TypeLambdaData, TypeLambdaProvenance,
};
pub use variance::{Variance, VarianceDiagnostic, VarianceStep, compute_variance_occurrence};
