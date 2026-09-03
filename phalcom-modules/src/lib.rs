//! `phalcom-modules` — logical module system, project manifests, source identity, and path visibility for Phalcom.

pub mod artifact;
pub mod builtin;
pub mod builtin_interface;
pub mod declaration;
pub mod dunder;
pub mod error;
pub mod graph;
pub mod identity;
pub mod interface;
pub mod linker;
pub mod manifest;
pub mod metadata;
pub mod package_info;
pub mod project;
pub mod query;
pub mod resolver;
pub mod session;
pub mod source;
pub mod stabilization;

// Re-export common types
pub use artifact::{PackageArtifactProvider, ResolvedPackageArtifact, ResolvedPackageId as PublishedPackageId};
pub use builtin::{UNIVERSE_NODES, UniverseNodeSpec, UniverseSourceProvider};
pub use declaration::{DeclarationBlueprint, DeclarationId, DeclarationKind, DeclarationRealizationError, DeclarationShell, DeclarationShellTable, ShellState};
pub use dunder::{DunderCategory, DunderPolicy, DunderPolicyError, DunderRole};
pub use error::{InterfaceError, ModuleGraphError, ModuleLoadError, ModuleResolutionError, ProjectError, SourceError};
pub use graph::{
    DependencyPhase, ModuleGraphs, ReferenceEdge, ReferenceGraph, ReferenceKind, RuntimeDependencyEdge, RuntimeDependencyGraph, RuntimeDependencyReason,
    SemanticEdge, SemanticEdgeKind, SemanticGraph, SemanticNodeId, strongly_connected_components,
};
pub use identity::{
    ImportRootTarget, ModuleComponent, ModuleId, ModulePath, ProjectIdentity, ProjectRevisionFingerprint, ProjectSourceIdentity, ResolvedProjectId, SourceId,
    SourceLocation, StableModuleKey, StableProjectKey, SyntheticProjectId, SyntheticProjectIdAllocator, universe_module_from_uri, universe_module_uri,
};
pub use interface::{
    DeclarationSurface, ExportSurface, ImportSurface, InterfaceBuilder, LinkedExport, LinkedExportTarget, LinkedModuleInterface, PackagePathSurface,
    UnlinkedExportTarget, UnlinkedModuleInterface,
};
pub use linker::{
    GlobalBindingId, ImportBindingId, LinkError, LinkedModule, LinkedProgram, LinkedReadSpec, ModuleBindingLayout, ModuleLinker, SymbolId, dependency_phase,
    module_path, resolution_key,
};
pub use manifest::{DependencyProvider, DependencySpec, NullDependencyProvider, ProjectManifest, ValidatedProjectManifest};
pub use metadata::{MetadataTarget, ModuleMetadata, ModuleMetadataAttribute};
pub use package_info::{
    PackageArtifactIdentity, PackageAuthorDescriptor, PackageInfoDescriptor, PackageOrigin, PackageRequirementDescriptor, ResolvedProjectDependencyDescriptor,
};
pub use project::{ProjectUniverse, ResolvedProject, discover_owning_project};
pub use query::ModuleQueryFacade;
pub use resolver::ModuleResolver;
pub use session::{
    SourceRevision, WorkspaceModuleSession, WorkspaceModuleSessionError, WorkspaceModuleUpdate, WorkspaceSourceBatchMutation, WorkspaceSourceMutation,
    WorkspaceSourceState,
};
pub use source::{EntryOwnership, FilesystemSourceProvider, ModuleKind, ParsedModuleUnit, SourceProvider, SourceUnit};
pub use stabilization::{ResolvedDocumentIdentity, ResolverGeneration};
