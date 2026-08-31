//! Core-side linked module artifacts.
//!
//! This layer owns VM-facing compiled metadata, while path resolution and
//! symbol identity remain in `phalcom-modules`. Runtime materialization is
//! intentionally separate and belongs to the module execution work.

pub mod artifact;
pub mod builtin_materialize;
pub mod compile;
pub mod context;
pub mod initialize;
pub mod linkage;
pub mod materialize;
pub mod reflection_cache;
pub mod registry;
pub mod semantic_lowering;

pub use artifact::{ClassBlueprint, EnumBlueprint, ModuleMaterializationPlan, RuntimeDeclarationBlueprint, VariantBlueprint};
pub use compile::{
    AnalyzedProgram, CompiledModule, CompiledProgram, EntrySelection, ProgramAnalyzer, ProgramCompileError, ProgramCompiler, ProgramSemanticDiagnostics,
};
pub use context::ModuleExecutionContext;
pub use linkage::{BindingRef, CompileBindings, LinkedImportInfo, RuntimeLinkedRead, TopLevelBindingInfo, TopLevelBindingKind};
pub use reflection_cache::ReflectionCache;
pub use registry::{ModuleFailure, ModuleFailureRef, ModulePlanFingerprint, ModuleRecord, ModuleRegistry, ModuleState, RuntimeProgramId};
pub use semantic_lowering::{
    AssociatedLoweringSpec, EnumLoweringSpec, ExecutableFamilyCandidate, ExecutableFamilyCandidateSet, ExecutableFamilyDescriptor, ExecutableFamilyEntry,
    ExecutableFamilyTarget, ExecutableInvocationTarget, ExecutableRestMode, FamilyApplicationLoweringSpec, LoweringSite, LoweringSiteKind,
    ModuleLoweringSemantics, VariantFieldLoweringSpec, VariantLoweringSpec, build_module_lowering_semantics,
};
