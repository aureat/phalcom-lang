//! Published package artifact boundary.
//!
//! A package is the atomic publication, distribution, and consumption artifact.
//! A project provides the development and workspace container holding one root
//! package and zero or more nested subpackages.

use crate::error::ProjectError;
use crate::identity::{ModuleId, ModulePath, ResolvedProjectId};
use crate::interface::LinkedModuleInterface;
use crate::manifest::ProjectManifest;
use std::collections::{BTreeMap, BTreeSet};

/// Type alias reflecting that a published package ID is a resolved package identity.
pub type ResolvedPackageId = ResolvedProjectId;

/// An immutable, linked artifact representing a published or prepared package.
#[derive(Clone, Debug, PartialEq)]
pub struct ResolvedPackageArtifact {
    /// Authoritative package identity.
    pub package_id: ResolvedPackageId,
    /// Verified manifest metadata describing the package.
    pub manifest: ProjectManifest,
    /// Root package module identity (`package.ph`).
    pub root_package: ModuleId,
    /// Fully linked module interfaces belonging to this package artifact.
    pub interfaces: BTreeMap<ModuleId, LinkedModuleInterface>,
    /// Hierarchical paths exposed for consumer imports.
    pub exposed_paths: BTreeSet<ModulePath>,
    /// Resolved dependency packages required by this artifact.
    pub dependency_artifacts: BTreeMap<String, ResolvedPackageId>,
}

impl ResolvedPackageArtifact {
    /// Creates a new package artifact container.
    pub fn new(
        package_id: ResolvedPackageId,
        manifest: ProjectManifest,
        root_package: ModuleId,
        interfaces: BTreeMap<ModuleId, LinkedModuleInterface>,
        exposed_paths: BTreeSet<ModulePath>,
        dependency_artifacts: BTreeMap<String, ResolvedPackageId>,
    ) -> Self {
        Self {
            package_id,
            manifest,
            root_package,
            interfaces,
            exposed_paths,
            dependency_artifacts,
        }
    }

    /// Returns `true` if `path` is exposed for external consumption by this package.
    pub fn is_path_exposed(&self, path: &ModulePath) -> bool {
        self.exposed_paths.contains(path)
    }
}

/// Provider trait for loading and querying package artifacts.
pub trait PackageArtifactProvider {
    /// Retrieves a published package artifact by ID.
    fn get_package_artifact(&self, id: ResolvedPackageId) -> Result<Option<&ResolvedPackageArtifact>, ProjectError>;
}
