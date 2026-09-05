//! Canonical module workspace diagnostics.

use crate::error::{InterfaceError, ModuleGraphError, ModuleResolutionError};
use crate::identity::ModuleId;
use crate::interface::UnlinkedModuleInterface;
use crate::linker::LinkError;
use phalcom_common::range::SourceRange;
use std::path::PathBuf;

/// Specific category of a module-authored source error.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ModuleDiagnosticKind {
    /// Relative import ascends beyond package root or requires package context.
    RelativeImportBeyondRoot { dots: usize, depth: usize },
    /// Relative import attempted without package context.
    RelativeImportWithoutPackage,
    /// Target module not found.
    ModuleNotFound(String),
    /// Target package not found.
    PackageNotFound(String),
    /// Target module path is private/not exposed by project.
    ModulePathNotExposed { path: String, project: String },
    /// Selected name is absent from target declarations and exports.
    UnknownImportName { module: String, name: String },
    /// Selected name is declared in target module but not exported.
    NonExportedImport { module: String, name: String },
    /// Export references a name not declared in the module.
    UnknownExport { name: String },
    /// Same name exported multiple times.
    DuplicateExport { name: String },
    /// Binding redeclared in module namespace.
    DuplicateBinding { name: String },
    /// Import binding conflicts with another binding in scope.
    DuplicateImportBinding { name: String },
    /// Top-level declaration duplicate.
    DuplicateDeclaration { name: String },
    /// Expose statement outside package.ph.
    ExposeOutsidePackage,
    /// Expose target is invalid or does not exist.
    InvalidExposeTarget(String),
    /// Import or exposure statement outside dependency preamble.
    ImportOutsidePreamble,
    /// Invalid module attribute / metadata.
    InvalidModuleMetadata { name: String, reason: String },
    /// Module attribute outside header.
    ModuleAttributeOutsideHeader,
    /// Import binding collision in linked module.
    BindingCollision { name: String },
    /// Cyclic re-export.
    CyclicReExport { name: String },
    /// Cyclic module runtime initialization.
    RuntimeCycle { cycle: Vec<ModuleId> },
    /// Unresolved import path.
    UnresolvedImport { path: String },
    /// Invalid module name.
    InvalidModuleName(String),
    /// Module imported from outside source root.
    ImportOutsideSourceRoot(PathBuf),
    /// Unknown import root.
    UnknownImportRoot(String),
    /// Syntax/parse error in module source.
    ParseError(String),
    /// General interface error.
    InterfaceError(String),
}

/// A structured module diagnostic with module identity and precise source range.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModuleDiagnostic {
    pub module: ModuleId,
    pub kind: ModuleDiagnosticKind,
    pub range: SourceRange,
    pub message: String,
}

impl ModuleDiagnostic {
    pub fn new(module: ModuleId, kind: ModuleDiagnosticKind, range: SourceRange, message: impl Into<String>) -> Self {
        Self {
            module,
            kind,
            range,
            message: message.into(),
        }
    }

    /// Converts an `InterfaceError` into a `ModuleDiagnostic`.
    pub fn from_interface_error(module: ModuleId, error: InterfaceError) -> Self {
        match error {
            InterfaceError::UnknownImportName { module: target, name, range } => Self {
                module,
                kind: ModuleDiagnosticKind::UnknownImportName { module: target.clone(), name: name.clone() },
                range,
                message: format!("Unknown import name: module '{}' does not export '{}'", target, name),
            },
            InterfaceError::NonExportedImport { module: target, name, range } => Self {
                module,
                kind: ModuleDiagnosticKind::NonExportedImport { module: target.clone(), name: name.clone() },
                range,
                message: format!("Non-exported import: '{}' is declared in '{}' but not exported", name, target),
            },
            InterfaceError::DuplicateBinding { name, range, .. } => Self {
                module,
                kind: ModuleDiagnosticKind::DuplicateBinding { name: name.clone() },
                range,
                message: format!("Duplicate binding '{}'", name),
            },
            InterfaceError::DuplicateImportBinding { name, range } => Self {
                module,
                kind: ModuleDiagnosticKind::DuplicateImportBinding { name: name.clone() },
                range,
                message: format!("Duplicate import binding '{}'", name),
            },
            InterfaceError::DuplicateDeclaration { name, range, .. } => Self {
                module,
                kind: ModuleDiagnosticKind::DuplicateDeclaration { name: name.clone() },
                range,
                message: format!("Duplicate declaration '{}'", name),
            },
            InterfaceError::UnknownExport { name, range } => Self {
                module,
                kind: ModuleDiagnosticKind::UnknownExport { name: name.clone() },
                range,
                message: format!("Unknown export '{}' is not declared in this module", name),
            },
            InterfaceError::DuplicateExport { name, range } => Self {
                module,
                kind: ModuleDiagnosticKind::DuplicateExport { name: name.clone() },
                range,
                message: format!("Duplicate export '{}'", name),
            },
            InterfaceError::InvalidExposeTarget(target, range) => Self {
                module,
                kind: ModuleDiagnosticKind::InvalidExposeTarget(target.clone()),
                range,
                message: format!("Invalid expose target: '{}'", target),
            },
            InterfaceError::ExposeOutsidePackage(range) => Self {
                module,
                kind: ModuleDiagnosticKind::ExposeOutsidePackage,
                range,
                message: "Expose outside package: `expose` is only valid in package.ph".to_string(),
            },
            InterfaceError::ImportOutsidePreamble(range) => Self {
                module,
                kind: ModuleDiagnosticKind::ImportOutsidePreamble,
                range,
                message: "Import outside preamble: static imports and exposures must appear in the module dependency preamble".to_string(),
            },
            InterfaceError::InvalidModuleMetadata { name, reason, range } => Self {
                module,
                kind: ModuleDiagnosticKind::InvalidModuleMetadata { name: name.clone(), reason: reason.clone() },
                range,
                message: format!("Invalid module metadata: attribute '{}' is invalid ({})", name, reason),
            },
            InterfaceError::ModuleAttributeOutsideHeader(range) => Self {
                module,
                kind: ModuleDiagnosticKind::ModuleAttributeOutsideHeader,
                range,
                message: "Module attribute outside header".to_string(),
            },
            InterfaceError::ReservedDunder { name, role, range } => Self {
                module,
                kind: ModuleDiagnosticKind::InterfaceError(format!("dunder name '{}' is reserved in role {}", name, role)),
                range,
                message: format!("dunder name '{}' is language-reserved in source role {}", name, role),
            },
            InterfaceError::BuiltinInterfaceCollision { module: target, name } => Self {
                module,
                kind: ModuleDiagnosticKind::BindingCollision { name: name.clone() },
                range: SourceRange::default(),
                message: format!("Builtin interface collision in module {}: '{}'", target, name),
            },
        }
    }

    /// Converts a `ModuleResolutionError` into a `ModuleDiagnostic` with a given source range.
    pub fn from_resolution_error(module: ModuleId, error: ModuleResolutionError, range: SourceRange) -> Self {
        match error {
            ModuleResolutionError::ModuleNotFound(name) => Self {
                module,
                kind: ModuleDiagnosticKind::ModuleNotFound(name.clone()),
                range,
                message: format!("Module not found: '{}'", name),
            },
            ModuleResolutionError::PackageNotFoundError(name) => Self {
                module,
                kind: ModuleDiagnosticKind::PackageNotFound(name.clone()),
                range,
                message: format!("Package not found: '{}'", name),
            },
            ModuleResolutionError::RelativeImportBeyondRoot { dots, depth } => Self {
                module,
                kind: ModuleDiagnosticKind::RelativeImportBeyondRoot { dots, depth },
                range,
                message: format!("Relative import ascends {} levels, which exceeds package depth {}", dots, depth),
            },
            ModuleResolutionError::ImportOutsideSourceRoot(path, root) => Self {
                module,
                kind: ModuleDiagnosticKind::ImportOutsideSourceRoot(path.clone()),
                range,
                message: format!("Import path '{}' escapes source root '{}'", path.display(), root.display()),
            },
            ModuleResolutionError::ModulePathNotExposed { path, project, .. } => Self {
                module,
                kind: ModuleDiagnosticKind::ModulePathNotExposed { path: path.clone(), project: project.clone() },
                range,
                message: format!("Module path not exposed: '{}' in project '{}' is private", path, project),
            },
            ModuleResolutionError::UnknownImportRoot(root) => Self {
                module,
                kind: ModuleDiagnosticKind::UnknownImportRoot(root.clone()),
                range,
                message: format!("Unknown import root: '{}'", root),
            },
            ModuleResolutionError::InvalidModuleName(name, _) => Self {
                module,
                kind: ModuleDiagnosticKind::InvalidModuleName(name.clone()),
                range,
                message: format!("Invalid module name: '{}'", name),
            },
            other => Self {
                module,
                kind: ModuleDiagnosticKind::InterfaceError(other.to_string()),
                range,
                message: other.to_string(),
            },
        }
    }

    /// Converts a `LinkError` into a `ModuleDiagnostic`.
    pub fn from_link_error(error: LinkError, target_interface: Option<&UnlinkedModuleInterface>) -> Self {
        match error {
            LinkError::UnresolvedImport { module, path, range } => Self {
                module,
                kind: ModuleDiagnosticKind::UnresolvedImport { path: path.clone() },
                range,
                message: format!("unresolved import '{}'", path),
            },
            LinkError::MissingExport { module, name, range } => {
                let is_private = target_interface.map_or(false, |iface| iface.declarations.contains_key(&name));
                if is_private {
                    Self {
                        module: module.clone(),
                        kind: ModuleDiagnosticKind::NonExportedImport { module: module.to_string(), name: name.clone() },
                        range,
                        message: format!("module {} declares '{}' but does not export it", module, name),
                    }
                } else {
                    Self {
                        module: module.clone(),
                        kind: ModuleDiagnosticKind::UnknownImportName { module: module.to_string(), name: name.clone() },
                        range,
                        message: format!("module {} does not export '{}'", module, name),
                    }
                }
            }
            LinkError::MissingBinding { module, name, range } => Self {
                module: module.clone(),
                kind: ModuleDiagnosticKind::UnknownImportName { module: module.to_string(), name: name.clone() },
                range,
                message: format!("module {} has no binding '{}'", module, name),
            },
            LinkError::BindingCollision { module, name, range } => Self {
                module,
                kind: ModuleDiagnosticKind::BindingCollision { name: name.clone() },
                range,
                message: format!("colliding binding '{}'", name),
            },
            LinkError::MissingModule { module } => Self {
                module: module.clone(),
                kind: ModuleDiagnosticKind::ModuleNotFound(module.to_string()),
                range: SourceRange::default(),
                message: format!("module {} is absent from the link universe", module),
            },
            LinkError::CyclicReExport { module, name } => Self {
                module: module.clone(),
                kind: ModuleDiagnosticKind::CyclicReExport { name: name.clone() },
                range: SourceRange::default(),
                message: format!("cyclic re-export involving module {} and '{}'", module, name),
            },
            LinkError::RuntimeCycle(ModuleGraphError::RuntimeCycle { cycle }) => {
                let primary_module = cycle.first().cloned().unwrap_or_else(ModuleId::universe_root);
                Self {
                    module: primary_module,
                    kind: ModuleDiagnosticKind::RuntimeCycle { cycle },
                    range: SourceRange::default(),
                    message: "cyclic module initialization".to_string(),
                }
            }
        }
    }
}
