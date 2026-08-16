//! `phalcom-modules` — logical module system, project manifests, source identity, and path visibility for Phalcom.

pub mod builtin;
pub mod error;
pub mod graph;
pub mod identity;
pub mod interface;
pub mod linker;
pub mod manifest;
pub mod metadata;
pub mod project;
pub mod resolver;
pub mod source;
pub mod stabilization;

// Re-export common types
pub use builtin::BuiltinProjectSourceProvider;
pub use error::{InterfaceError, ModuleGraphError, ModuleLoadError, ModuleResolutionError, ProjectError, SourceError};
pub use graph::{
    DependencyPhase, ModuleGraphs, ReferenceEdge, ReferenceGraph, ReferenceKind, RuntimeDependencyEdge, RuntimeDependencyGraph, RuntimeDependencyReason,
    SemanticEdge, SemanticEdgeKind, SemanticGraph, SemanticNodeId, strongly_connected_components,
};
pub use identity::{
    BuiltinProject, ImportRootTarget, ModuleComponent, ModuleId, ModulePath, ProjectIdentity, ProjectSourceIdentity, ResolvedProjectId, SourceId, SourceLocation,
    SyntheticProjectId, SyntheticProjectIdAllocator,
};
pub use interface::{
    DeclarationSurface, ExportSurface, ImportSurface, InterfaceBuilder, LinkedExport, LinkedExportTarget, LinkedModuleInterface, PackagePathSurface,
    UnlinkedExportTarget, UnlinkedModuleInterface,
};
pub use linker::{
    GlobalBindingId, ImportBindingId, LinkError, LinkedModule, LinkedProgram, LinkedReadSpec, ModuleBindingLayout, ModuleLinker, SymbolId, dependency_phase,
    module_path, resolution_key,
};
pub use manifest::{DependencyProvider, DependencySpec, NullDependencyProvider, ProjectManifest};
pub use metadata::{MetadataTarget, ModuleMetadata, ModuleMetadataAttribute};
pub use project::{ProjectUniverse, ResolvedProject, discover_owning_project};
pub use resolver::ModuleResolver;
pub use source::{EntryOwnership, FilesystemSourceProvider, ModuleKind, SourceProvider, SourceUnit};
pub use stabilization::{DunderRole, ResolvedDocumentIdentity, ResolverGeneration, builtin_module_uri, dunder_role};

