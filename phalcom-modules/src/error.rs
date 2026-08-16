//! Typed errors for manifests, projects, source resolution, and visibility.

use crate::identity::{ModuleId, SourceLocation};
use phalcom_ast::error::SyntaxError;
use phalcom_common::range::SourceRange;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error, Clone, PartialEq)]
pub enum ProjectError {
    #[error("Invalid project manifest: {0}")]
    InvalidProjectManifest(String),
    #[error("Persistent projects must declare an explicit canonical snake_case namespace")]
    MissingProjectNamespace,
    #[error("Invalid project namespace '{0}': {1}")]
    InvalidProjectNamespace(String, InvalidModuleNameError),
    #[error("Invalid dependency alias '{0}': {1}")]
    InvalidDependencyAlias(String, InvalidModuleNameError),
    #[error("Import root collision: alias '{alias}' collides with {reason}")]
    ImportRootCollision { alias: String, reason: String },
    #[error("Project dependency cycle detected: {chain}")]
    ProjectDependencyCycle { chain: String },
    #[error("Unresolved package dependency '{package}' ({version_requirement})")]
    UnresolvedPackageDependency { package: String, version_requirement: String },
    #[error("Project path dependency not found: {0}")]
    PathDependencyNotFound(PathBuf),
    #[error("Project source root '{0}' does not exist or is not a directory")]
    InvalidSourceRoot(PathBuf),
    #[error("Project source root '{0}' is missing package.ph")]
    MissingRootPackage(PathBuf),
    #[error("Invalid entry module '{0}': {1}")]
    InvalidEntry(String, String),
}

#[derive(Debug, Error, Clone, PartialEq)]
pub enum ModuleResolutionError {
    #[error("Module not found: '{0}'")]
    ModuleNotFound(String),
    #[error("Package not found: '{0}'")]
    PackageNotFoundError(String),
    #[error("Invalid module layout: {0}")]
    InvalidModuleLayout(String),
    #[error("Non-canonical physical module/package name for logical '{logical}': expected '{expected}', found '{found}'")]
    NonCanonicalPhysicalName { logical: String, expected: String, found: PathBuf },
    #[error("Portable module-name collision between '{first}' and '{second}' ({reason})")]
    PortabilityCollision { first: PathBuf, second: PathBuf, reason: String },
    #[error("Invalid module name '{0}': {1}")]
    InvalidModuleName(String, InvalidModuleNameError),
    #[error("Unknown import root: '{0}'")]
    UnknownImportRoot(String),
    #[error("Relative import ascends {dots} levels, which exceeds package depth {depth}")]
    RelativeImportBeyondRoot { dots: usize, depth: usize },
    #[error("Import path '{0}' escapes source root '{1}'")]
    ImportOutsideSourceRoot(PathBuf, PathBuf),
    #[error("Cannot resolve into nested project boundary at '{0}'")]
    NestedProjectBoundary(PathBuf),
    #[error("Duplicate source identity: '{0}'")]
    DuplicateSourceIdentity(String),
    #[error("Standalone module '{entry}' cannot discover sibling module '{requested}'")]
    StandaloneSiblingImport { entry: PathBuf, requested: String },
    #[error("Module path not exposed: '{path}' in project '{project}' is private")]
    ModulePathNotExposed { path: String, project: String, exposed: Vec<String> },
    #[error("Source provider error: {0}")]
    Source(#[from] SourceError),
}

#[derive(Debug, Error, Clone, PartialEq)]
pub enum SourceError {
    #[error("Source reading IO error ({kind:?}): {message}")]
    Io { kind: std::io::ErrorKind, message: String },
    #[error("Source not found: {0}")]
    NotFound(String),
}

impl SourceError {
    pub fn from_io(error: std::io::Error, context: impl Into<String>) -> Self {
        Self::Io { kind: error.kind(), message: format!("{}: {}", context.into(), error) }
    }
}

#[derive(Debug, Error, Clone, PartialEq)]
pub enum ModuleLoadError {
    #[error(transparent)]
    Resolution(#[from] ModuleResolutionError),
    #[error("Parse error in module {module}: {error}")]
    Parse { module: ModuleId, location: SourceLocation, error: SyntaxError },
    #[error("Interface error in module {module}: {error}")]
    Interface { module: ModuleId, error: InterfaceError },
    #[error("I/O error while loading {module:?}: {error}")]
    Io { module: Option<ModuleId>, error: SourceError },
}

#[derive(Debug, Error, Clone, PartialEq)]
pub enum InterfaceError {
    #[error("Unknown import name: module '{module}' does not export '{name}'")]
    UnknownImportName { module: String, name: String, range: SourceRange },
    #[error("Non-exported import: '{name}' is declared in '{module}' but not exported")]
    NonExportedImport { module: String, name: String, range: SourceRange },
    #[error("Duplicate import binding: '{name}' is already bound in this scope")]
    DuplicateImportBinding { name: String, previous_range: SourceRange, range: SourceRange },
    #[error("Duplicate declaration: '{name}' is already declared in this module")]
    DuplicateDeclaration { name: String, first_range: SourceRange, range: SourceRange },
    #[error("Unknown export: '{name}' is not declared in this module")]
    UnknownExport { name: String, range: SourceRange },
    #[error("Duplicate export: '{name}' is exported more than once")]
    DuplicateExport { name: String, range: SourceRange },
    #[error("Reserved dunder name '{name}' is not legal in {role}")]
    ReservedDunder { name: String, role: String, range: SourceRange },
    #[error("Invalid expose target: expose operand must be an immediate child (.child), got '{0}'")]
    InvalidExposeTarget(String, SourceRange),
    #[error("Expose outside package: `expose` is only valid in package.ph")]
    ExposeOutsidePackage(SourceRange),
    #[error("Import outside preamble: static imports and exposures must appear in the module dependency preamble")]
    ImportOutsidePreamble(SourceRange),
    #[error("Invalid module metadata: attribute '{name}' is invalid for target ({reason})")]
    InvalidModuleMetadata { name: String, reason: String, range: SourceRange },
    #[error("Module attribute outside header: @! attributes must appear at the very top of the file before imports")]
    ModuleAttributeOutsideHeader(SourceRange),
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum ModuleGraphError {
    #[error("cyclic module initialization: {cycle:?}")]
    RuntimeCycle { cycle: Vec<ModuleId> },
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum InvalidModuleNameError {
    #[error("Name cannot be empty")]
    Empty,
    #[error("'{0}' is not canonical snake_case")]
    NonCanonicalSnakeCase(String),
    #[error("'{0}' is not canonical kebab-case")]
    NonCanonicalKebabCase(String),
}
