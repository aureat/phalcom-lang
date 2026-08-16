//! Core-side linked module plans and runtime materialization.

pub mod artifact;
pub mod compile;
pub mod initialize;
pub mod linkage;
pub mod materialize;
pub mod registry;

pub use artifact::{ClassBlueprint, ModuleArtifact, ModuleMaterializationPlan, RuntimeDeclarationBlueprint};
pub use compile::{CompiledModule, CompiledProgram, EntrySelection, ProgramCompileError, ProgramCompiler};
pub use linkage::{BindingRef, CompileBindings, LinkedImportInfo, RuntimeLinkedRead, TopLevelBindingInfo, TopLevelBindingKind};
pub use registry::{
    ModuleFailure, ModuleOwner, ModulePlanFingerprint, ModuleRecord, ModuleRegistry, ModuleRegistryError, ModuleState, RuntimeProgramId,
};
