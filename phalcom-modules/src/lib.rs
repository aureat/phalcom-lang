//! `phalcom-modules` — logical module system, project manifests, source identity, and path visibility for Phalcom.

pub mod error;
pub mod identity;
pub mod interface;
pub mod manifest;
pub mod metadata;
pub mod project;
pub mod resolver;
pub mod source;

// Re-export common types
pub use error::{InterfaceError, ModuleResolutionError, ProjectError, SourceError};
pub use identity::{ModuleComponent, ModuleId, ModulePath, ProjectSourceIdentity, ResolvedProjectId, SourceId, SourceLocation};
pub use interface::{DeclarationSurface, ExportSurface, ImportSurface, InterfaceBuilder, PackagePathSurface, UnlinkedModuleInterface};
pub use manifest::{DependencyProvider, DependencySpec, NullDependencyProvider, ProjectManifest};
pub use metadata::{MetadataTarget, ModuleMetadata, ModuleMetadataAttribute};
pub use project::{ProjectUniverse, ResolvedProject, discover_owning_project};
pub use resolver::ModuleResolver;
pub use source::{FilesystemSourceProvider, ModuleKind, SourceProvider, SourceUnit};
