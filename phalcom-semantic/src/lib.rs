//! `phalcom-semantic` — Canonical static semantic analysis, type store, subtyping relations,
//! and verification engine for Phalcom.

#![allow(clippy::large_enum_variant)]
#![allow(clippy::result_large_err)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::type_complexity)]

pub mod access;
pub mod advisory;
pub mod algebraic_semantics;
pub mod associated;
pub mod canonical_baseline;
pub mod checker;
pub mod contracts;
pub mod control_summary;
pub mod core_surface;
pub mod data_semantics;
pub mod db;
pub mod declaration_type;
pub mod declarations;
pub mod derived_conformance;
pub mod diagnostic;
pub mod diagnostic_presentation;
pub mod dispatch;
pub mod editor;
pub mod effects;
pub mod enum_requirements;
pub mod enum_semantics;
pub mod explain;
pub mod field_key;
pub mod export;
pub mod generic_evidence;
pub mod hierarchy_product;
pub mod identity;
pub mod impls;
#[cfg(any(test, feature = "test-timings"))]
pub mod instrumentation;
pub mod invalidation;
pub mod match_semantics;
pub mod materialization;
pub mod metadata;
pub mod module_bindings;
pub mod module_product;
pub mod prelude;
pub mod presentation;
pub mod prover;
pub mod reflection;
pub mod resolver;
pub mod scope;
pub mod semantic_shard;
pub mod scalar;
pub mod session;
pub mod signature;
pub mod snapshot;
pub mod source;
pub mod source_index;
pub mod stable_identity;
pub mod static_case;
pub mod static_constant;
pub mod static_definition;
pub mod static_domain;
pub mod static_eval;
pub mod static_intrinsic;
pub mod static_value;
pub mod structural;
pub mod surface;
pub mod termination;
pub mod tooling;
pub mod trait_dispatch;
pub mod traits;
pub mod type_alias;
pub mod types;
pub mod workspace;
pub mod workspace_inputs;

pub use access::{AccessContext, DeclarationAccessOrigin, is_member_accessible};
pub use advisory::{
    AdvisoryBuiltins, AdvisoryCallableSummary, AdvisoryConfidence, AdvisoryContributionSource, AdvisoryExpressionContext, AdvisoryFact, AdvisoryFlowContext,
    AdvisoryFlowProduct, AdvisoryLiteral, AdvisoryModuleProduct, AdvisoryOrigin, AdvisoryParameterContributions, AdvisoryParameterFactDelta,
    AdvisoryParameterSlot, AdvisoryProductStatus, AdvisorySummaryEffects, AdvisoryTargetResolution, AdvisoryWorkspace, CapturedMethodFamilyShape,
    MAX_SHAPE_UNION, ValueShape, analyze_expr, analyze_statements,
};
pub use associated::{AssociatedFamilyInfo, AssociatedFamilyKind, AssociatedFamilyTable, AssociatedMemberId, AssociatedSurface};
pub use canonical_baseline::*;
pub use checker::{
    AssociatedResolution, AssociatedResolutionIndex, AssociatedResolutionKind, CallableReferenceResolution, CallableReferenceResolutionIndex,
    CallableReferenceResolutionKind, CheckingContext, FamilyApplicationCandidate, FamilyApplicationResolution, FamilyApplicationResolutionIndex,
    FamilyApplicationSelection, SpecializedAssociatedMember, StatementControl, TypeCheckReport, TypedExpression, check_class, check_class_bodies,
    check_statement, register_class_surface, synthesize_expr, synthesize_typed_expr,
};
pub use contracts::{ConditionKind, ContractCondition, ContractSpec};
pub use control_summary::{ControlFacts, DivergenceKnowledge, DivergenceOpaqueReason, ExitSummary, RaiseKnowledge, RaiseOpaqueReason};
pub use core_surface::*;
pub use data_semantics::{DataComponentSemantic, DataConstructorParameter, DataConstructorSignature, DataInfo, DataSemanticTable, DataShape};
pub use declaration_type::{DeclaredTypeBasis, DeclaredTypeFact, DeclaredTypeState};
pub use declarations::{
    DeclarationTypeInfo, DeclarationTypeTable, GenericSupertypeTemplate, TypeDeclarationShell, bootstrap_universe_declarations, lower_kind_spec,
};
pub use diagnostic::{
    DiagnosticCode, DiagnosticFix, DiagnosticGuidance, DiagnosticLabel, DiagnosticSeverity, ExplanationRef, SemanticDiagnostic, SemanticSourceSpan,
};
pub use diagnostic_presentation::{
    DiagnosticDetail, DiagnosticPresenter, PresentedDiagnostic, PresentedLabel, PresentedLabelRole, PresentedLine, PresentedTraceNode,
};
pub use dispatch::{
    CallableParameter, CallableSignature, DispatchResolver, DispatchResult, DispatchSide, DispatchSignatureSpecialization, DispatchTarget,
    SurfaceDispatchResolver,
};
pub use editor::{
    EditorMember, EditorMemberTarget, EditorSemanticQuery, EditorTypeHint, EditorTypeHintKind, NativeCallablePresentation, PartialCallPattern,
    ReceiverAlternative, ReceiverMode, ReferenceDomain, RenameTargetView, ResolvedReceiver, SemanticDefinitionLocation, VisibleSymbol,
};
pub use effects::{
    EffectAtom, EffectKnowledge, EffectOpaqueReason, EffectSet, adapt_effect_atom, adapt_effect_spec, infer_interprocedural_effects_scc,
    infer_intraprocedural_effects,
};
pub use enum_requirements::{CaseRequirementResult, CaseRequirementStatus, EnumRequirement, EnumRequirementId, EnumRequirementTable};
pub use enum_semantics::{
    EnumInfo, EnumSemanticTable, VariantConstructorParameter, VariantConstructorSignature, VariantFieldSemantic, VariantInfo, VariantShape, VariantVisibility,
};
pub use explain::{DerivationRule, ExplanationArena, ExplanationNode, ExplanationStep, causal_slice, causal_trace};
pub use export::{
    CompiledCallableParam, CompiledCallableType, CompiledKindRef, CompiledRecordField, CompiledTupleElement, CompiledTypeParameterOwner, CompiledTypeRef,
    SemanticExportError, export_kind, export_type_form,
};
pub use generic_evidence::*;
pub use identity::{
    AssociatedFamilyId, BindingId, CallableId, CallableOwnerId, CallableParameterId, DataComponentId, DataConstructorId, DeclarationId, FieldId,
    InvocationTargetId, ModuleId, ProperTypeId, SemanticRevision, SemanticTargetId, SnapshotId, SnapshotTypeRef, SourceOwner, SourceSiteId, SourceSiteLocalId,
    SourceSiteRef, TypeStoreId, VariantConstructorId, VariantFamilyId, VariantFieldId, VariantId, WorkspaceId,
};
#[cfg(any(test, feature = "test-timings"))]
pub use instrumentation::*;
pub use invalidation::{DeclarationFingerprint, InvalidationIndex};
pub use match_semantics::*;
pub use module_bindings::{ModuleBindingFact, ModuleBindingPhase, ModuleStaticBindingIndex};
pub use phalcom_modules::WorkspaceSourceBatchMutation;
pub use prelude::PreludeTypeMap;
pub use presentation::{
    AdvisoryPresenter, CallablePresentation, FieldPresentation, FormalContractRelation, FormalFactRef, FormalFactSite, FormalFactStatus, FormalPresentation,
    FormalSemanticProjection, FormalSiteId, FormalTypeSite, ParameterPresentation, SemanticPresentationIndex, SemanticSiteView, TypePresenter,
};
pub use prover::{
    Counterexample, ProofBinaryOp, ProofEvidence, ProofObligationKind, ProofOpaqueReason, ProofTerm, ProofUnaryOp, VcStatus, VcUnknownReason,
    VerificationCondition, simplify_proof_term, solve_vc_deterministic,
};
pub use resolver::LinkedTypeResolver;
pub use scope::ScopeTable;
pub use semantic_shard::ModuleSemanticStructureShard;
pub use session::{SemanticPublicationEffects, SemanticUpdateStats, SemanticWorkspacePublication, SemanticWorkspaceSession, SemanticWorkspaceUpdate};
pub use signature::{
    CallableParameterSemantic, CallableSemanticSignature, CallableSignatureTable, ConstructorInitializationProduct, ConstructorInitializationTable,
    ConstructorOrigin, ConstructorParameterField, FieldSemanticSignature, FieldSignatureTable, ReturnContractValidation,
};
pub use snapshot::{ModuleQueryProducts, SemanticSnapshot};
pub use source_index::{
    EditorSymbolKind, ModuleReferenceContribution, ReferenceIndex, SourceIndexUpdateStats, TargetReferenceSet, WorkspaceSymbolEntry, WorkspaceSymbolId,
    WorkspaceSymbolIndex,
};
pub use static_constant::{CheckedStaticConstantInitializer, StaticConstantId, StaticConstantProduct, StaticConstantTable};
pub use static_definition::{
    CheckedStaticBody, StaticBodyId, StaticDefinition, StaticDefinitionBodyRepr, StaticDefinitionProduct, StaticDefinitionSignature, StaticDefinitionTable,
    StaticParameter, resolve_static_definition,
};
pub use static_domain::{StaticDomainResolution, StaticValueDomain, is_proper_type, resolve_static_domain, validate_proper_type};
pub use structural::{
    OrderedProductEntry, OrderedProductForm, OrderedProductOrigin, OrderedProductStructure, SpecializedVariantStructure, StructuralClassification,
    StructuralClassifier, SumStructure, SumStructureError,
};
pub use static_intrinsic::{
    StaticIntrinsicOperation, StaticIntrinsicRegistry, StaticIntrinsicRegistryError, StaticOperationResolution, StaticOperationResolutionIndex,
};
pub use termination::{
    RankingMeasure, TerminationBlockedReason, TerminationCounterevidence, TerminationEvidence, TerminationKnowledge, TerminationRequirement,
    analyze_callable_termination, check_cfg_acyclicity,
};
pub use trait_dispatch::{
    AssumedTraitDispatchSelection, ConformanceSemanticView, TraitDispatchBucketKey, TraitDispatchContribution, TraitDispatchConvergenceKey, TraitDispatchIndex,
    TraitDispatchResolution, TraitDispatchSelection, TraitDispatchSite, TraitDispatchTargetFamily, TraitDispatchTerminal, TraitEvidencedMemberCandidate,
    selected_callable,
};
pub use traits::{
    AssociatedTypeRequirementId, TraitAssociatedTypeRequirement, TraitHeader, TraitHeaderTable, TraitRef, TraitRefFormationError, TraitRequirementId,
    TraitSurface, TraitSurfaceMember, TraitSurfaceTable,
};
pub use type_alias::{TypeAliasInfo, TypeAliasTable};
pub use workspace_inputs::*;

pub use source::ParsedSourceUnit;
pub use source_index::{CallableSourceAttachment, ModuleSourceIndex, SourceAttachmentError, SourceIndexFingerprints, SourceSemanticIndex};
pub use source_index::{CallableSourceInfo, DeclarationSourceInfo, FieldSourceInfo, SourceCallableKind, SourceReceiverKind};
pub use source_index::{
    ImportBindingOrigin, SourceBindingInfo, SourceBindingKind, SourceIndexContext, SourceNameResolution, SourceScope, SourceScopeId, SourceScopeIndex,
    build_source_scope_index,
};
pub use source_index::{OccurrenceHint, OccurrenceIndex, OccurrenceKind, OccurrenceRole, OccurrenceView, SemanticOccurrence};
pub use source_index::{SourceSite, SourceSiteKind};
pub use stable_identity::*;
pub use field_key::{FieldKey, FieldKeyKind};
pub use scalar::StaticInt;
pub use surface::{DeclarationSurface, MemberVisibility};
pub use types::{
    Assignability, BlockReason, BudgetKind, BudgetReport, CallableParameterType, CallableType, CancellationToken, ConstraintSet, ContractAssumptionEligibility,
    DynamicBoundaryObligation, DynamicReason, EvidenceOrigin, EvidenceSet, EvidenceStatus, GenericSignature, InferVarId, KindData, KindId, MapTypeHierarchy,
    NativeSurfaceImportError, NativeSurfaceImportReport, NativeTypeResolutionError, QueryBudget, RecordTypeField, RefutationReason, RelationEvidence,
    RelationFailure, RelationOutcome, SemanticDenotation, SimpleTypeResolver, TupleTypeElement, TypeApplicationError, TypeConstraint, TypeData, TypeEvidence,
    TypeFormResolution, TypeFormationInvalid, TypeFormationMissing, TypeFormationOutcome, TypeFormationUnresolved, TypeHierarchy, TypeId, TypeKnowledge,
    TypeLevelBinding, TypeParameterData, TypeParameterId, TypeParameterOwner, TypeResolver, TypeStore, TypeSubstitution, UnknownReason, ValueSemanticFact,
    VariantTypeId, check_assignability, check_assignability_bounded, check_knowledge_against_type, check_knowledge_against_type_bounded, check_subtype_bounded,
    is_subtype, normalize_native_type, register_native_surfaces, register_native_surfaces_from_records, resolve_native_type_form, resolve_type_annotation,
    resolve_type_form, substitution_for_applied,
};
pub use workspace::{SemanticAnalysis, SemanticWorkspaceInput, analyze_single_module, analyze_workspace};
