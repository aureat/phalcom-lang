//! Core-side linked module artifacts.
//!
//! This layer owns VM-facing compiled metadata, while path resolution and
//! symbol identity remain in `phalcom-modules`. Runtime materialization is
//! intentionally separate and belongs to the module execution work.

pub mod artifact;
pub mod builtin_materialize;
pub mod canonical_artifact;
mod telosc_staging;
mod telosc_compiler_wire;
mod telosc_dependency_plan;
mod package_identity;
pub mod canonical_universe;
pub mod compile;
pub mod compile_session;
pub mod context;
pub mod initialize;
pub mod linkage;
pub mod materialize;
pub mod parity;
pub mod reflection_cache;
pub mod reflection_metadata;
pub mod registry;
pub mod semantic_lowering;
mod static_constant;

pub use parity::{assert_compile_error_parity, assert_compiled_program_parity, assert_semantic_diagnostics_parity, compare_compiled_programs};

pub use artifact::{ClassBlueprint, EnumBlueprint, ModuleMaterializationPlan, RuntimeDeclarationBlueprint, VariantBlueprint};
pub use canonical_universe::{CanonicalCompatibilityIdentity, CanonicalCompilerImageView, CanonicalUniverseProgram, canonical_universe_program};
pub use compile::{
    AnalyzedProgram, CompiledModule, CompiledProgram, EntrySelection, ProgramAnalyzer, ProgramCompilationOptions, ProgramCompileError, ProgramCompiler,
    ProgramSemanticDiagnostics,
};
pub use compile_session::ProgramCompilationSession;
pub use context::ModuleExecutionContext;
pub use linkage::{BindingRef, CompileBindings, LinkedImportInfo, RuntimeLinkedRead, RuntimeStaticConstantRef, TopLevelBindingInfo, TopLevelBindingKind};
pub use reflection_cache::ReflectionCache;
pub use registry::{ModuleFailure, ModuleFailureRef, ModulePlanFingerprint, ModuleRecord, ModuleRegistry, ModuleState, RuntimeProgramId};
pub use semantic_lowering::{
    AnonymousProductConstructionKind, AnonymousProductConstructionLoweringSpec, AssociatedLoweringSpec, CallableReferenceLoweringSpec, EnumLoweringSpec,
    ExecutableBindingSpec, ExecutableFamilyCandidate, ExecutableFamilyCandidateSet, ExecutableFamilyDescriptor, ExecutableFamilyEntry, ExecutableFamilyTarget,
    ExecutableFieldProjection, ExecutableInvocationTarget, ExecutableMatchArm, ExecutablePattern, ExecutableRestMode, ExecutableVariantCandidate,
    FamilyApplicationLoweringSpec, LoweringSite, LoweringSiteKind, MatchLoweringSpec, ModuleLoweringSemantics, PatternSiteLoweringSpec,
    VariantFieldLoweringSpec, VariantLoweringSpec, anonymous_product_type_roots, build_module_lowering_semantics,
    build_module_lowering_semantics_with_runtime_types,
};
