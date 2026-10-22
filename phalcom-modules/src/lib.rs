//! `phalcom-modules` — logical module system, project manifests, source identity, and path visibility for Phalcom.

pub mod artifact;
pub mod authority;
pub mod builtin;
pub mod builtin_interface;
pub mod canonical_products;
pub mod external_authority;
pub use external_authority::{ExternalModuleAuthority, ExternalModuleAuthoritySet, ExternalModuleOrigin};
pub mod compiler_artifact;
pub mod declaration;
pub mod diagnostic;
pub mod dunder;
pub mod error;
pub mod fingerprint;
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
pub mod topology;

// Re-export common types
pub use artifact::{PackageArtifactProvider, ResolvedPackageArtifact, ResolvedPackageId as PublishedPackageId};
pub use builtin::{UNIVERSE_NODES, UniverseNodeSpec, UniverseSourceProvider};
pub use canonical_products::{CanonicalExternalModuleWorld, CanonicalModuleProducts};
pub use compiler_artifact::{
    load_module_world, load_module_world_with_allocation_budget, load_module_world_with_project_catalog_and_allocation_budget,
    project_executable_module_world, project_executable_module_world_v4, project_module_world, stage_artifact_project_authorities,
};
pub use declaration::{DeclarationBlueprint, DeclarationId, DeclarationKind, DeclarationRealizationError, DeclarationShell, DeclarationShellTable, ShellState};
pub use diagnostic::{ModuleDiagnostic, ModuleDiagnosticCode, ModuleDiagnosticKind};
pub use dunder::{DunderCategory, DunderPolicy, DunderPolicyError, DunderRole};
pub use error::{InterfaceError, ModuleGraphError, ModuleLoadError, ModuleResolutionError, ProjectError, SourceError};
pub use fingerprint::{
    InterfaceFingerprint, LinkedInterfaceFingerprint, hash_linked_interface, hash_unlinked_interface, interface_fingerprint, linked_interface_fingerprint,
    linked_interface_input_fingerprint, unlinked_interface_input_fingerprint,
};
pub use graph::{
    DependencyPhase, ModuleGraphs, ReferenceEdge, ReferenceGraph, ReferenceKind, RuntimeDependencyEdge, RuntimeDependencyGraph, RuntimeDependencyReason,
    RuntimeRequirementProducer, RuntimeRequirementProduct, RuntimeRequirementProvenance, RuntimeRequirementTarget, RuntimeRequirementTiming, SemanticEdge,
    SemanticEdgeKind, SemanticGraph, SemanticNodeId, strongly_connected_components,
};
pub use identity::{
    ImportBindingLocalId, ImportRootTarget, ImportSiteId, ImportSiteLocalId, ModuleComponent, ModuleId, ModulePath, ProjectIdentity,
    ProjectRevisionFingerprint, ProjectSourceIdentity, ResolvedProjectId, SourceId, SourceImportBindingId, SourceLocation, StableModuleKey, StableProjectKey,
    SyntheticProjectId, SyntheticProjectIdAllocator, universe_module_from_uri, universe_module_uri,
};
pub use interface::{
    DeclarationSurface, ExportSurface, ImportSurface, InterfaceBuilder, LinkedExport, LinkedExportTarget, LinkedModuleInterface, PackagePathSurface,
    UnlinkedExportTarget, UnlinkedModuleInterface,
};
pub use linker::{
    AvailabilityAuthority, AvailabilityContract, AvailabilityProof, ExecutableLinkedProgram, GlobalBindingId, ImportBindingId, ImportPhaseSummary, LinkError,
    LinkedModule, LinkedProgram, LinkedReadSpec, ModuleBindingLayout, ModuleImportPhaseProduct, ModuleLinker, PhaseEvidenceCoverage,
    RuntimeAvailabilityWitness, RuntimeDependencyObligationKey, RuntimeDependencyRequirement, SemanticLinkedModule, SemanticLinkedProgram, SymbolId, SymbolicImportBinding,
    SymbolicImportTable, SymbolicImportTarget,
    dependency_phase, module_path, resolution_key,
};
pub use manifest::{DependencyProvider, DependencySpec, NullDependencyProvider, ProjectManifest, ValidatedProjectManifest};
pub use metadata::{MetadataTarget, ModuleMetadata, ModuleMetadataAttribute};
pub use package_info::{
    PackageArtifactIdentity, PackageAuthorDescriptor, PackageInfoDescriptor, PackageOrigin, PackageRequirementDescriptor, ResolvedProjectDependencyDescriptor,
};
pub use project::{
    PackageAuthorityRegistrationError, ProjectUniverse, ResolvedProject, TrustedPackageResolver, ValidatedPackageAuthorityReceipt, discover_owning_project,
    discover_standalone_package_root,
};
pub use query::{ModuleQueryFacade, query_fallback_scan_count, reset_query_fallback_scan_count};
pub use resolver::{
    AbsentCandidateFact, ImportPathIdentity, ImportResolutionProduct, ImportResolutionTrace, ModuleResolver, ResolutionFingerprint,
    ResolutionTopologyDependencies, ResolvedImportPrefix,
};
pub use session::{
    SourceRevision, WorkspaceModuleSession, WorkspaceModuleSessionError, WorkspaceModuleStats, WorkspaceModuleUpdate, WorkspaceSourceBatchMutation,
    WorkspaceSourceMutation, WorkspaceSourceState,
};
pub use source::{
    DirEntryKind, DirectorySnapshot, EntryOwnership, FilesystemSourceProvider, ModuleKind, ParsedModuleUnit, SourceProvider, SourceUnit,
    classify_entry_ownership,
};
pub use stabilization::{ResolvedDocumentIdentity, ResolverGeneration};
pub use topology::{ModuleTopology, TopologyDelta, TopologyFingerprint, TopologyNode};
