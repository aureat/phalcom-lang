//! Native heap object representations for reflection and modularity descriptors.

use crate::heap::ObjRef;
use crate::interner::Symbol;
use std::collections::HashMap;

/// A live project development environment (`Project`).
#[derive(Debug, Clone)]
pub struct ProjectObject {
    pub name: String,
    pub namespace: Symbol,
    pub manifest: ObjRef,
    pub root_package: ObjRef,
    pub dependencies: ObjRef,
    pub development_entry: Option<ObjRef>,
    pub identity: ObjRef,
}

/// Validated project development configuration (`ProjectManifest`).
#[derive(Debug, Clone)]
pub struct ProjectManifestObject {
    pub name: String,
    pub namespace: Symbol,
    pub version: Option<String>,
    pub authors: ObjRef,
    pub description: Option<String>,
    pub license: Option<String>,
    pub homepage: Option<ObjRef>,
    pub repository: Option<ObjRef>,
    pub source: String,
    pub entry: Option<String>,
    pub default_entry: Option<String>,
    pub dependency_declarations: ObjRef,
}

/// Durable descriptive metadata for a root package artifact (`PackageInfo`).
#[derive(Debug, Clone)]
pub struct PackageInfoObject {
    pub name: String,
    pub namespace: Symbol,
    pub version: Option<String>,
    pub authors: ObjRef,
    pub description: Option<String>,
    pub license: Option<String>,
    pub homepage: Option<ObjRef>,
    pub repository: Option<ObjRef>,
    pub requirements: ObjRef,
    pub default_entry: Option<String>,
    pub identity: ObjRef,
}

/// Structured package author metadata (`PackageAuthor`).
#[derive(Debug, Clone)]
pub struct PackageAuthorObject {
    pub name: String,
    pub email: Option<String>,
    pub url: Option<ObjRef>,
}

/// Durable unresolved package requirement (`PackageRequirement`).
#[derive(Debug, Clone)]
pub struct PackageRequirementObject {
    pub alias: Symbol,
    pub package: String,
    pub version_requirement: String,
    pub optional: bool,
}

/// Resolved dependency inside an active development project (`ResolvedProjectDependency`).
#[derive(Debug, Clone)]
pub struct ResolvedProjectDependencyObject {
    pub alias: Symbol,
    pub requirement: Option<ObjRef>,
    pub package_info: ObjRef,
    pub root_package: ObjRef,
    pub origin_sym: Symbol,
}

/// Module runtime dependency reference (`ModuleDependency`).
#[derive(Debug, Clone)]
pub struct ModuleDependencyObject {
    pub module: ObjRef,
    pub phase_sym: Symbol,
    pub reason_sym: Symbol,
}

/// Reflective view of a module's public export surface (`ExportTable`).
#[derive(Debug, Clone)]
pub struct ExportTableObject {
    pub module: ObjRef,
    pub names: Vec<Symbol>,
    pub names_tuple: ObjRef,
    pub descriptors: HashMap<Symbol, ObjRef>,
}

/// Reflected individual export binding or module re-export (`Export`).
#[derive(Debug, Clone)]
pub struct ExportObject {
    pub module: ObjRef,
    pub name: Symbol,
    pub kind_sym: Symbol,
}

/// Exposed child module table for a Package (`ChildModuleTable`).
#[derive(Debug, Clone)]
pub struct ChildModuleTableObject {
    pub package: ObjRef,
    pub names: Vec<Symbol>,
    pub names_tuple: ObjRef,
    pub children: HashMap<Symbol, ObjRef>,
}

/// Opaque module identity (`ModuleIdentity`).
#[derive(Debug, Clone)]
pub struct ModuleIdentityObject {
    pub id_str: String,
    pub uri: ObjRef,
}

/// Opaque package artifact identity (`PackageIdentity`).
#[derive(Debug, Clone)]
pub struct PackageIdentityObject {
    pub identity_str: String,
}

/// Opaque project identity (`ProjectIdentity`).
#[derive(Debug, Clone)]
pub struct ProjectIdentityObject {
    pub identity_str: String,
}

/// Canonical logical URI (`Uri`).
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct UriObject {
    pub uri_str: String,
}
