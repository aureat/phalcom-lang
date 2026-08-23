//! `phalcom-semantic` — Canonical static semantic analysis, type store, subtyping relations,
//! and verification engine for Phalcom.

pub mod checker;
pub mod contracts;
pub mod control_summary;
pub mod core_surface;
pub mod db;
pub mod declarations;
pub mod diagnostic;
pub mod dispatch;
pub mod effects;
pub mod explain;
pub mod export;
pub mod identity;
pub mod invalidation;
pub mod metadata;
pub mod presentation;
pub mod prover;
pub mod resolver;
pub mod scope;
pub mod session;
pub mod signature;
pub mod snapshot;
pub mod source;
pub mod surface;
pub mod termination;
pub mod types;
pub mod workspace;

pub use checker::{
    CheckingContext, TypeCheckReport, TypedExpression, check_arguments, check_class, check_class_bodies, check_statement, match_callable_arguments,
    register_class_surface, synthesize_expr, synthesize_typed_expr,
};
pub use contracts::{ConditionKind, ContractCondition, ContractSpec};
pub use control_summary::{ControlFacts, DivergenceKnowledge, DivergenceOpaqueReason, ExitSummary, RaiseKnowledge, RaiseOpaqueReason};
pub use core_surface::*;
pub use declarations::{DeclarationTypeInfo, DeclarationTypeTable, GenericSupertypeTemplate, bootstrap_universe_declarations, lower_kind_spec};
pub use diagnostic::{DiagnosticCode, DiagnosticLabel, DiagnosticSeverity, SemanticDiagnostic, SemanticSourceSpan};
pub use dispatch::{CallableParameter, CallableSignature, DispatchResolver, DispatchResult, DispatchSide, DispatchTarget, SurfaceDispatchResolver};
pub use effects::{
    EffectAtom, EffectKnowledge, EffectOpaqueReason, EffectSet, adapt_effect_atom, adapt_effect_spec, infer_interprocedural_effects_scc,
    infer_intraprocedural_effects,
};
pub use explain::{ExplanationArena, ExplanationNode, ExplanationStep, causal_slice};
pub use export::{
    CompiledCallableParam, CompiledCallableType, CompiledKindRef, CompiledRecordField, CompiledTupleElement, CompiledTypeParameterOwner, CompiledTypeRef,
    SemanticExportError, export_kind, export_type_form,
};
pub use identity::{
    BindingId, CallableId, DeclarationId, FieldId, ModuleId, ProperTypeId, SemanticRevision, SnapshotId, SnapshotTypeRef, TypeStoreId, WorkspaceId,
};
pub use invalidation::{DeclarationFingerprint, InvalidationIndex};
pub use presentation::{FormalPresentation, FormalSiteId, FormalTypeSite, SemanticPresentationIndex, TypePresenter};
pub use prover::{
    Counterexample, ProofBinaryOp, ProofEvidence, ProofObligationKind, ProofOpaqueReason, ProofTerm, ProofUnaryOp, VcStatus, VcUnknownReason,
    VerificationCondition, simplify_proof_term, solve_vc_deterministic,
};
pub use resolver::LinkedTypeResolver;
pub use scope::ScopeTable;
pub use session::{SemanticUpdateStats, SemanticWorkspaceSession, SemanticWorkspaceUpdate};
pub use signature::{CallableParameterSemantic, CallableSemanticSignature, CallableSignatureTable, FieldSemanticSignature, FieldSignatureTable};
pub use snapshot::SemanticSnapshot;
pub use termination::{
    RankingMeasure, TerminationBlockedReason, TerminationCounterevidence, TerminationEvidence, TerminationKnowledge, TerminationRequirement,
    analyze_callable_termination, check_cfg_acyclicity,
};

pub use source::ParsedSourceUnit;
pub use surface::DeclarationSurface;
pub use types::{
    Assignability, BlockReason, BudgetKind, BudgetReport, CallableParameterType, CallableType, CancellationToken, ConstraintSet, DynamicBoundaryObligation,
    DynamicReason, EvidenceAuthority, EvidenceSet, GenericSignature, InferVarId, KindData, KindId, MapTypeHierarchy, NativeSurfaceImportError,
    NativeSurfaceImportReport, NativeTypeResolutionError, QueryBudget, RecordTypeField, RefutationReason, RelationEvidence, RelationFailure, RelationOutcome,
    SemanticDenotation, SimpleTypeResolver, TupleTypeElement, TypeApplicationError, TypeConstraint, TypeData, TypeEvidence, TypeHierarchy, TypeId,
    TypeKnowledge, TypeParameterData, TypeParameterId, TypeParameterOwner, TypeResolver, TypeStore, TypeSubstitution, UnknownReason, ValueSemanticFact,
    check_assignability, check_assignability_bounded, check_subtype_bounded, is_subtype, normalize_native_type, register_native_surfaces,
    resolve_native_type_form, resolve_type_annotation, resolve_type_form, substitution_for_applied,
};
pub use workspace::{SemanticAnalysis, SemanticWorkspaceInput, analyze_single_module, analyze_workspace};
