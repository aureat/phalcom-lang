//! Canonical static type system layer for Phalcom.

pub mod annotation;
pub mod application;
pub mod constraint;
pub mod denotation;
pub mod environment;
pub mod evidence;
pub mod id;
pub mod kind;
pub mod native;
pub mod outcome;
pub mod parameter;
pub mod relation;
pub mod row;
pub mod row_solver;
pub mod store;
pub mod substitution;
pub mod type_lambda;
pub mod variance;

pub use annotation::{SimpleTypeResolver, TypeFormResolution, TypeResolver, resolve_type_annotation, resolve_type_form};
pub use application::TypeApplicationError;
pub use constraint::{ConstraintSet, TypeConstraint};
pub use denotation::{SemanticDenotation, ValueSemanticFact};
pub use environment::{SpecializedCallableView, SpecializedMemberView, TypeEnvironment, TypeView};
pub use evidence::{
    ContractAssumptionEligibility, DynamicReason, EvidenceOrigin, EvidenceSet, EvidenceStatus, TypeEvidence, TypeKnowledge, UnknownReason, join_type_knowledge,
};
pub use id::{InferVarId, KindId, ProperTypeId, RecordRowId, ScopedTypeId, TypeId, TypeLambdaId, TypeParameterId, TypeStoreId};
pub use kind::{KindApplicationError, KindData};
pub use native::{
    NativeSurfaceImportError, NativeSurfaceImportReport, NativeTypeResolutionError, normalize_native_type, register_native_surfaces, resolve_native_type_form,
};
pub use outcome::{
    BlockReason, BudgetKind, BudgetReport, CancellationToken, DynamicBoundaryObligation, QueryBudget, RelationEvidence, RelationFailure, RelationOutcome,
};
pub use parameter::{GenericConstraint, GenericSignature, SelfRole, SelfTypeTerm, TypeParameterData, TypeParameterOwner, TypeTerm};
pub use relation::{
    Assignability, MapTypeHierarchy, RefutationReason, TypeHierarchy, check_assignability, check_assignability_bounded, check_knowledge_against_type,
    check_knowledge_against_type_bounded, check_subtype_bounded, is_subtype,
};
pub use row::{DuplicateFieldError, RecordAccess, RecordRowData, RecordRowField, RecordRowTail};
pub use row_solver::{
    IncidentId, RecordRowBlockedReason, RecordRowFailure, RecordRowLacks, RecordRowSolution, RecordRowSolveResult, RecordRowSolver, RecordRowTerm,
    RecordRowVarId, RowBudgetReport,
};
pub use store::{CallableParameterType, CallableType, RecordTypeField, TupleTypeElement, TypeData, TypeStore};
pub use substitution::{TypeSubstitution, substitution_for_applied};
pub use type_lambda::{
    BetaReductionError, BetaResult, ScopedCallableParameter, ScopedCallableType, ScopedRecordField, ScopedTupleElement, ScopedTypeData, TypeLambdaArena,
    TypeLambdaData, TypeLambdaProvenance,
};
pub use variance::{Variance, VarianceDiagnostic, VarianceStep, compute_variance_occurrence};
