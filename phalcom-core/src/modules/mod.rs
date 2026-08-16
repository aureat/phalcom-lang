//! Core-side linked module artifacts.
//!
//! This layer owns VM-facing compiled metadata, while path resolution and
//! symbol identity remain in `phalcom-modules`. Runtime materialization is
//! intentionally separate and belongs to the module execution work.

pub mod artifact;
pub mod compile;
pub mod initialize;
pub mod linkage;
pub mod materialize;
pub mod registry;

pub use artifact::{ClassBlueprint, ModuleArtifact, RuntimeDeclarationBlueprint};
pub use compile::{CompiledModule, CompiledProgram, EntrySelection, ProgramCompileError, ProgramCompiler};
pub use linkage::{BindingRef, CompileBindings, LinkedImportInfo, RuntimeLinkedRead, TopLevelBindingInfo, TopLevelBindingKind};
pub use registry::{ModuleFailure, ModuleRecord, ModuleRegistry, ModuleState};
