//! Program-level compiler seam for closed linked module plans.

use super::artifact::ModuleArtifact;
use phalcom_modules::{LinkError, LinkedModule, LinkedProgram, ModuleId, ModuleKind, ProjectUniverse, SourceLocation};
use std::collections::BTreeMap;
use std::sync::Arc;
use thiserror::Error;

/// Entry selection for a program compilation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EntrySelection {
    /// Compile the explicitly selected module.
    Module(ModuleId),
}

/// One VM-independent compiled module.
#[derive(Clone, Debug, PartialEq)]
pub struct CompiledModule {
    /// Module identity.
    pub id: ModuleId,
    /// Source kind from the linked interface.
    pub kind: ModuleKind,
    /// Source location is attached by a source-backed compiler facade when
    /// available; the VM-independent linker itself only knows logical identity.
    pub source: Option<SourceLocation>,
    /// Linked source interface.
    pub interface: Arc<phalcom_modules::LinkedModuleInterface>,
    /// Materialization/initializer artifact.
    pub artifact: ModuleArtifact,
    /// Symbolic reads retained for compiler/runtime lowering.
    pub linked_reads: Vec<phalcom_modules::LinkedReadSpec>,
}

/// Closed compiled program passed to runtime materialization.
#[derive(Clone, Debug)]
pub struct CompiledProgram {
    /// Resolved project universe shared by every linked module.
    pub project_universe: Arc<ProjectUniverse>,
    /// Original linked program plan.
    pub linked: Arc<LinkedProgram>,
    /// Compiled modules keyed by semantic identity.
    pub modules: BTreeMap<ModuleId, CompiledModule>,
    /// Program entry module.
    pub entry: ModuleId,
    /// Precomputed initialization order.
    pub initialization_order: Vec<ModuleId>,
}

/// Program-level compile errors remain structured by phase.
#[derive(Debug, Error)]
pub enum ProgramCompileError {
    /// Static linking failed.
    #[error(transparent)]
    Link(#[from] LinkError),
    /// Requested entry is not part of the linked plan.
    #[error("entry module {0} is not present in linked program")]
    MissingEntry(ModuleId),
}

/// Compiler facade over an already-linked program.
pub struct ProgramCompiler {
    linked: Arc<LinkedProgram>,
}

impl ProgramCompiler {
    /// Creates a compiler for a closed linked plan.
    pub fn new(linked: Arc<LinkedProgram>) -> Self {
        Self { linked }
    }

    /// Validates entry selection and creates per-module artifact shells. No
    /// source import is discovered while this phase runs.
    pub fn compile_entry(&self, entry: EntrySelection) -> Result<CompiledProgram, ProgramCompileError> {
        let EntrySelection::Module(entry) = entry;
        if !self.linked.modules.contains_key(&entry) {
            return Err(ProgramCompileError::MissingEntry(entry));
        }
        let modules = self
            .linked
            .modules
            .iter()
            .map(|(id, module)| (id.clone(), compile_module(id.clone(), module)))
            .collect();
        Ok(CompiledProgram {
            project_universe: self.linked.universe.clone(),
            linked: self.linked.clone(),
            modules,
            entry,
            initialization_order: self.linked.initialization_order.clone(),
        })
    }
}

fn compile_module(id: ModuleId, module: &LinkedModule) -> CompiledModule {
    let artifact = ModuleArtifact::empty(module);
    CompiledModule {
        id,
        kind: module.interface.kind,
        source: None,
        interface: Arc::new(module.interface.clone()),
        linked_reads: module.linked_reads.clone(),
        artifact,
    }
}
