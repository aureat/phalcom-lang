//! Typed errors for manifests, projects, source resolution, and visibility.

use crate::identity::ModuleId;
use phalcom_common::range::SourceRange;
use std::path::PathBuf;
use thiserror::Error;

// ── Project / Manifest Errors ──────────────────────────────────────────────

#[derive(Debug, Error, Clone, PartialEq)]
pub enum ProjectError {
    #[error("Invalid project manifest: {0}")]
    InvalidProjectManifest(String),

    #[error("Persistent projects must declare project.namespace explicitly")]
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

// ── Source / Resolution Errors ─────────────────────────────────────────────

#[derive(Debug, Error, Clone, PartialEq)]
pub enum ModuleResolutionError {
    #[error("Module not found: '{0}'")]
    ModuleNotFound(String),

    #[error("Package not found: '{0}'")]
    PackageNotFoundError(String),

    #[error("Invalid module layout: {0}")]
    InvalidModuleLayout(String),

    #[error("Ambiguous module '{name}': both '{kebab_path}' and '{snake_path}' exist on disk")]
    AmbiguousModule { name: String, kebab_path: PathBuf, snake_path: PathBuf },

    #[error("Invalid module name '{0}': {1}")]
    InvalidModuleName(String, InvalidModuleNameError),

    #[error("Non-canonical physical module/package name at '{path}'; expected '{expected}'")]
    NonCanonicalPhysicalName { path: PathBuf, expected: String },

    #[error("Parse error while resolving module {module}: {error}")]
    Parse { module: ModuleId, error: phalcom_ast::error::SyntaxError },

    #[error("Interface error while resolving module {module}: {error}")]
    Interface { module: ModuleId, error: InterfaceError },

    #[error("Unknown import root: '{0}'")]
    UnknownImportRoot(String),

    #[error("the public `core` import root has been retired; import from `universe` instead")]
    LegacyCoreImportRemoved,

    #[error("the `std` builtin package was removed; use `universe.<path>`")]
    LegacyStdImportRemoved,

    #[error("Relative import ascends {dots} levels, which exceeds package depth {depth}")]
    RelativeImportBeyondRoot { dots: usize, depth: usize },

    #[error("Import path '{0}' escapes source root '{1}'")]
    ImportOutsideSourceRoot(PathBuf, PathBuf),

    #[error("Cannot resolve into nested project boundary at '{0}'")]
    NestedProjectBoundary(PathBuf),

    #[error("Duplicate source identity: '{0}'")]
    DuplicateSourceIdentity(String),

    #[error("Module path not exposed: '{path}' in project '{project}' is private")]
    ModulePathNotExposed { path: String, project: String, exposed: Vec<String> },

    #[error("Package surface load failed: {0}")]
    PackageSurface(Box<ModuleLoadError>),

    #[error("Source provider error: {0}")]
    Source(#[from] SourceError),
}

#[derive(Debug, Error, Clone, PartialEq)]
pub enum SourceError {
    #[error("Source reading IO error: {0}")]
    Io(String),

    #[error("Source not found: {0}")]
    NotFound(String),
}

// ── Module Loading Errors ──────────────────────────────────────────────────

#[derive(Debug, Error, Clone, PartialEq)]
pub enum ModuleLoadError {
    #[error(transparent)]
    Resolution(#[from] ModuleResolutionError),

    #[error("Parse error in module {module} ({source}): {error}", source = source.display())]
    Parse {
        module: ModuleId,
        source: PathBuf,
        #[source]
        error: phalcom_ast::error::SyntaxError,
    },

    #[error("Interface error in module {module}: {error}")]
    Interface { module: ModuleId, error: InterfaceError },
}

// ── Visibility / Binding / Interface Errors ────────────────────────────────

#[derive(Debug, Error, Clone, PartialEq)]
pub enum InterfaceError {
    #[error("Unknown import name: module '{module}' does not export '{name}'")]
    UnknownImportName { module: String, name: String, range: SourceRange },

    #[error("Non-exported import: '{name}' is declared in '{module}' but not exported")]
    NonExportedImport { module: String, name: String, range: SourceRange },

    #[error("Duplicate binding '{name}': previous binding at {previous_range:?}")]
    DuplicateBinding {
        name: String,
        previous_range: SourceRange,
        range: SourceRange,
    },

    #[error("dunder name '{name}' is language-reserved in source role {role}")]
    ReservedDunder { name: String, role: &'static str, range: SourceRange },

    #[error("Duplicate import binding: '{name}' is already bound in this scope")]
    DuplicateImportBinding { name: String, range: SourceRange },

    #[error("Duplicate declaration: '{name}' is already declared in this module")]
    DuplicateDeclaration {
        name: String,
        first_range: SourceRange,
        range: SourceRange,
    },

    #[error("Unknown export: '{name}' is not declared in this module")]
    UnknownExport { name: String, range: SourceRange },

    #[error("Duplicate export: '{name}' is exported more than once")]
    DuplicateExport { name: String, range: SourceRange },

    #[error("Invalid expose target: expose operand must be an immediate child (.child), got '{0}'")]
    InvalidExposeTarget(String, SourceRange),

    #[error("Expose outside package: `expose` is only valid in package.ph")]
    ExposeOutsidePackage(SourceRange),

    #[error("Builtin interface collision in module {module}: '{name}'")]
    BuiltinInterfaceCollision { module: String, name: String },

    #[error("Import outside preamble: static imports and exposures must appear in the module dependency preamble")]
    ImportOutsidePreamble(SourceRange),

    #[error("Invalid module metadata: attribute '{name}' is invalid for target ({reason})")]
    InvalidModuleMetadata { name: String, reason: String, range: SourceRange },

    #[error("Module attribute outside header: @! attributes must appear at the very top of the file before imports")]
    ModuleAttributeOutsideHeader(SourceRange),
}

/// Errors raised while validating the linked module graph.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum ModuleGraphError {
    /// Eager runtime initialization would require a cyclic order.
    #[error("cyclic module initialization: {cycle:?}")]
    RuntimeCycle { cycle: Vec<ModuleId> },
}

// ── Name Validation Error ──────────────────────────────────────────────────

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum InvalidModuleNameError {
    #[error("Name cannot be empty")]
    Empty,

    #[error("Invalid leading character in '{0}': must be ASCII alphabetic or '_'")]
    InvalidLeadingChar(String),

    #[error("Invalid character '{1}' in '{0}': logical names must be lowercase ASCII snake_case")]
    InvalidChar(String, char),

    #[error("Invalid physical module/package component '{0}': expected lowercase ASCII kebab-case")]
    InvalidPhysicalForm(String),

    #[error("Logical module component '{0}' is not canonical snake_case")]
    NonCanonicalLogical(String),

    #[error("Physical module component '{0}' is not canonical kebab-case")]
    NonCanonicalPhysical(String),
}
