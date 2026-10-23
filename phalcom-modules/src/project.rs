//! Project universe, resolved projects, and project dependency graph management.

use crate::error::ProjectError;
use crate::identity::{
    ImportRootTarget, ModuleComponent, ModulePath, ProjectSourceIdentity, ResolvedProjectId, SyntheticProjectId, SyntheticProjectIdAllocator,
};
use crate::manifest::{DependencyProvider, DependencySpec, NullDependencyProvider, ProjectManifest};
use phalcom_compiler_artifact::digest::ArtifactDigest;
use phalcom_type_meta::fingerprint::Fingerprint128;
use phalcom_type_meta::identity::StableProjectRef;
use std::collections::{BTreeMap, HashSet};
use std::path::{Path, PathBuf};

fn update_revision_hash(state: &mut u128, bytes: &[u8]) {
    for byte in bytes {
        *state ^= u128::from(*byte);
        *state = state.wrapping_mul(0x1000000000000000000013b);
    }
}

/// A fully resolved project in the `ProjectUniverse`.
#[derive(Debug, Clone)]
pub struct ResolvedProject {
    pub id: ResolvedProjectId,
    pub name: String,                                                      // original name for diagnostics
    pub namespace: ModuleComponent,                                        // canonical snake_case namespace
    pub root_dir: PathBuf,                                                 // directory containing project.toml
    pub source_root: PathBuf,                                              // directory containing source code (e.g. root_dir/src)
    pub entry: Option<ModulePath>,                                         // project-relative entry module path
    pub dependencies: BTreeMap<ModuleComponent, ResolvedProjectId>,        // alias -> resolved project id
    pub import_roots: BTreeMap<ModuleComponent, (ImportRootTarget, bool)>, // root table
    pub source_identity: ProjectSourceIdentity,
    /// True only for a persistent project.toml boundary. Synthetic resolved
    /// roots are standalone Package compatibility contexts, not Projects.
    pub persistent_project: bool,
    /// The validated project manifest, if loaded from a project.toml.
    pub manifest: Option<crate::manifest::ValidatedProjectManifest>,
}

/// A package resolver's validated authority statement for a persistent project.
///
/// The caller must obtain receipts from its configured trusted package resolver.
/// The receipt binds the full stable package identity to the validated artifact
/// digest and an existing project ID. The artifact importer still checks these
/// fields against the artifact and current project universe before registration.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValidatedPackageAuthorityReceipt {
    stable_project: StableProjectRef,
    artifact_digest: ArtifactDigest,
    project_id: ResolvedProjectId,
}

impl ValidatedPackageAuthorityReceipt {
    /// Creates a receipt at the trusted resolver boundary.
    ///
    /// Callers must not construct receipts from artifact contents alone. The
    /// resolver is responsible for authenticating the package and verifying
    /// the artifact digest before issuing this value.
    pub fn from_trusted_resolver(stable_project: StableProjectRef, artifact_digest: ArtifactDigest, project_id: ResolvedProjectId) -> Option<Self> {
        matches!(&stable_project, StableProjectRef::Package { .. }).then_some(Self {
            stable_project,
            artifact_digest,
            project_id,
        })
    }

    pub fn stable_project(&self) -> &StableProjectRef {
        &self.stable_project
    }

    pub fn artifact_digest(&self) -> ArtifactDigest {
        self.artifact_digest
    }

    pub fn project_id(&self) -> ResolvedProjectId {
        self.project_id
    }
}

/// Trusted capability for obtaining package authority receipts during import.
///
/// The capability implementation is supplied by the caller's configured
/// package resolver. A missing receipt is a hard failure for package owners.
pub trait TrustedPackageResolver {
    fn package_authority_receipt(&self, stable_project: &StableProjectRef) -> Option<ValidatedPackageAuthorityReceipt>;
}

#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
pub enum PackageAuthorityRegistrationError {
    #[error("trusted package resolver did not provide an authority receipt")]
    MissingReceipt,
    #[error("package authority receipt names a different stable project")]
    ReceiptProjectMismatch,
    #[error("package authority receipt digest does not match the validated artifact")]
    ArtifactDigestMismatch,
    #[error("package authority receipt targets an unknown project")]
    UnknownProject,
    #[error("package authority receipt targets a non-persistent project")]
    NonPersistentProject,
    #[error("package authority receipt conflicts with an existing binding")]
    ConflictingBinding,
}

impl ResolvedProject {
    /// Returns whether this resolved project is a synthetic standalone package rather than a persistent project.
    pub const fn is_standalone_package(&self) -> bool {
        !self.persistent_project
    }

    /// Returns the precomputed import root table for this project:
    /// Maps each recognized root component (Universe, self namespace, and dependency aliases) to (ImportRootTarget, is_self).
    pub fn import_roots(&self) -> &BTreeMap<ModuleComponent, (ImportRootTarget, bool)> {
        &self.import_roots
    }

    /// Computes a deterministic fingerprint from the project's source tree.
    ///
    /// File paths are sorted before hashing so filesystem enumeration order and
    /// project-graph allocation order cannot affect durable identity.
    pub fn revision_fingerprint(&self) -> crate::identity::ProjectRevisionFingerprint {
        let mut files = Vec::new();
        collect_source_files(&self.source_root, &mut files);
        files.sort();

        let mut state = 0x6c62272e07bb014262b821756295c58du128;
        for path in files {
            let relative = path.strip_prefix(&self.source_root).unwrap_or(&path);
            update_revision_hash(&mut state, relative.to_string_lossy().as_bytes());
            if let Ok(bytes) = std::fs::read(&path) {
                update_revision_hash(&mut state, &bytes);
            } else {
                update_revision_hash(&mut state, b"<unreadable>");
            }
        }
        crate::identity::ProjectRevisionFingerprint::from_bytes(state.to_be_bytes())
    }
}

fn collect_source_files(root: &Path, files: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(root) else { return };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_source_files(&path, files);
        } else if path.is_file() {
            files.push(path);
        }
    }
}

/// The set of all resolved projects participating in a compilation or analysis session.
#[derive(Debug, Clone)]
pub struct ProjectUniverse {
    projects: Vec<ResolvedProject>,
    roots: BTreeMap<ProjectSourceIdentity, ResolvedProjectId>,
    package_authorities: BTreeMap<StableProjectRef, ValidatedPackageAuthorityReceipt>,
    synthetic_ids: SyntheticProjectIdAllocator,
}

impl Default for ProjectUniverse {
    fn default() -> Self {
        Self::new()
    }
}

impl ProjectUniverse {
    pub fn new() -> Self {
        Self {
            projects: Vec::new(),
            roots: BTreeMap::new(),
            package_authorities: BTreeMap::new(),
            synthetic_ids: SyntheticProjectIdAllocator,
        }
    }

    /// Returns a reference to all resolved projects.
    pub fn projects(&self) -> &[ResolvedProject] {
        &self.projects
    }

    /// Gets a resolved project by its non-zero graph-node ID.
    pub fn get_project(&self, id: ResolvedProjectId) -> Option<&ResolvedProject> {
        self.projects.get((id.raw() - 1) as usize)
    }

    /// Resolves a stable persistent project reference by exact identity.
    ///
    /// Source references compare both logical URI and current source fingerprint.
    /// Package references require a previously registered resolver receipt.
    /// Session identities never resolve through the persistent project graph.
    pub fn resolve_stable_project(&self, stable_project: &StableProjectRef) -> Option<crate::identity::ProjectIdentity> {
        match stable_project {
            StableProjectRef::Builtin { namespace, version } if namespace.as_ref() == "universe" && version.as_ref() == "0.1.0" => {
                Some(crate::identity::ProjectIdentity::Universe)
            }
            StableProjectRef::SourceArtifact {
                logical_uri,
                source_fingerprint,
            } => {
                let mut matches = self.projects.iter().filter(|project| {
                    project.persistent_project
                        && project.source_identity.0.to_string_lossy() == **logical_uri
                        && Fingerprint128(project.revision_fingerprint().0) == *source_fingerprint
                });
                let project_id = matches.next()?.id;
                matches.next().is_none().then_some(crate::identity::ProjectIdentity::Resolved(project_id))
            }
            StableProjectRef::Package { .. } => {
                let receipt = self.package_authorities.get(stable_project)?;
                self.get_project(receipt.project_id())
                    .filter(|project| project.persistent_project)
                    .map(|_| crate::identity::ProjectIdentity::Resolved(receipt.project_id()))
            }
            StableProjectRef::Builtin { .. } | StableProjectRef::Session { .. } => None,
        }
    }

    /// The unique trusted package identity that owns this native persistent
    /// project, if one has been registered. Receipts are indexed by full stable
    /// identity (including artifact fingerprint), never by a display name.
    ///
    /// Registration enforces the reverse uniqueness invariant: two distinct
    /// package identities cannot claim the same native project ID.
    pub fn stable_package_owner(&self, project_id: ResolvedProjectId) -> Option<&StableProjectRef> {
        self.package_authorities
            .iter()
            .find_map(|(identity, receipt)| (receipt.project_id() == project_id).then_some(identity))
    }

    /// Keeps graph slots and receipts while allowing mutable source manifests
    /// to be loaded again. Artifact projects retain their validated bindings.
    pub(crate) fn workspace_update_candidate(&self, immutable: impl Fn(ResolvedProjectId) -> bool) -> Self {
        let mut candidate = self.clone();
        candidate.roots.retain(|_, id| immutable(*id) || !self.get_project(*id).is_some_and(|project| project.persistent_project));
        candidate
    }

    /// Checks that extending a project catalog did not rebind any existing owner
    /// or discard a validated package receipt. No filesystem access is needed.
    pub fn preserves_authority_of(&self, base: &Self) -> bool {
        base.projects.iter().all(|prior| {
            self.get_project(prior.id).is_some_and(|current| {
                current.source_identity == prior.source_identity
                    && current.persistent_project == prior.persistent_project
                    && current.namespace == prior.namespace
                    && current.dependencies == prior.dependencies
                    && current.import_roots == prior.import_roots
            })
        }) && base.package_authorities.iter().all(|(owner, receipt)| self.package_authorities.get(owner) == Some(receipt))
    }

    /// Checks a package receipt without changing this project universe.
    pub fn validate_package_authority_receipt(
        &self,
        stable_project: &StableProjectRef,
        receipt: &ValidatedPackageAuthorityReceipt,
        expected_artifact_digest: Option<ArtifactDigest>,
    ) -> Result<ResolvedProjectId, PackageAuthorityRegistrationError> {
        if !matches!(stable_project, StableProjectRef::Package { .. }) || receipt.stable_project() != stable_project {
            return Err(PackageAuthorityRegistrationError::ReceiptProjectMismatch);
        }
        if expected_artifact_digest.is_some_and(|expected| receipt.artifact_digest() != expected) {
            return Err(PackageAuthorityRegistrationError::ArtifactDigestMismatch);
        }
        let project = self
            .get_project(receipt.project_id())
            .ok_or(PackageAuthorityRegistrationError::UnknownProject)?;
        if !project.persistent_project {
            return Err(PackageAuthorityRegistrationError::NonPersistentProject);
        }
        if self.package_authorities.get(stable_project).is_some_and(|registered| registered != receipt) {
            return Err(PackageAuthorityRegistrationError::ConflictingBinding);
        }
        // Enforce a bijective persistent package-owner mapping. This prevents
        // distinct artifact identities (including different fingerprints for
        // the same package/version) from aliasing one native receiving ID.
        if self
            .stable_package_owner(receipt.project_id())
            .is_some_and(|existing| existing != stable_project)
        {
            return Err(PackageAuthorityRegistrationError::ConflictingBinding);
        }
        Ok(receipt.project_id())
    }

    /// Registers a receipt after the importer has validated its package artifact.
    pub fn register_package_authority_receipt(
        &mut self,
        stable_project: &StableProjectRef,
        receipt: ValidatedPackageAuthorityReceipt,
        expected_artifact_digest: Option<ArtifactDigest>,
    ) -> Result<ResolvedProjectId, PackageAuthorityRegistrationError> {
        let project_id = self.validate_package_authority_receipt(stable_project, &receipt, expected_artifact_digest)?;
        self.package_authorities.entry(stable_project.clone()).or_insert(receipt);
        Ok(project_id)
    }

    /// Obtains and registers package authority through the caller's trusted resolver.
    ///
    /// `expected_artifact_digest` must be supplied for the package artifact currently
    /// being imported. External package owners may use `None`: their receipt is still
    /// checked against the exact stable identity and an existing persistent project.
    pub fn register_trusted_package_authority(
        &mut self,
        stable_project: &StableProjectRef,
        expected_artifact_digest: Option<ArtifactDigest>,
        resolver: &dyn TrustedPackageResolver,
    ) -> Result<ResolvedProjectId, PackageAuthorityRegistrationError> {
        let receipt = resolver
            .package_authority_receipt(stable_project)
            .ok_or(PackageAuthorityRegistrationError::MissingReceipt)?;
        self.register_package_authority_receipt(stable_project, receipt, expected_artifact_digest)
    }

    /// Allocates a fresh synthetic execution identity for inline/standalone code.
    pub fn allocate_synthetic_id(&mut self) -> SyntheticProjectId {
        self.synthetic_ids.allocate()
    }

    /// Loads and resolves a root project and its full dependency graph from a `project.toml` file path.
    pub fn load_root(&mut self, manifest_path: impl AsRef<Path>) -> Result<ResolvedProjectId, ProjectError> {
        let dep_provider = NullDependencyProvider;
        self.load_root_with_provider(manifest_path, &dep_provider)
    }

    /// Loads and resolves a root project with a custom dependency provider.
    pub fn load_root_with_provider(
        &mut self,
        manifest_path: impl AsRef<Path>,
        dep_provider: &dyn DependencyProvider,
    ) -> Result<ResolvedProjectId, ProjectError> {
        let manifest_path = manifest_path.as_ref();
        let canonical_manifest = manifest_path
            .canonicalize()
            .map_err(|e| ProjectError::InvalidProjectManifest(format!("Failed to canonicalize {}: {}", manifest_path.display(), e)))?;

        let root_dir = canonical_manifest
            .parent()
            .ok_or_else(|| ProjectError::InvalidProjectManifest("Manifest path has no parent directory".to_string()))?
            .to_path_buf();

        let source_identity = ProjectSourceIdentity::from_path(&root_dir);

        if let Some(&id) = self.roots.get(&source_identity) {
            return Ok(id);
        }

        // We load the graph using DFS and detect cycles
        let mut visiting = Vec::new();
        let mut visited_stack = HashSet::new();

        let root_id = self.resolve_project_recursive(&canonical_manifest, dep_provider, &mut visiting, &mut visited_stack)?;

        Ok(root_id)
    }

    fn resolve_project_recursive(
        &mut self,
        manifest_path: &Path,
        dep_provider: &dyn DependencyProvider,
        visiting: &mut Vec<(String, PathBuf)>, // (display_name, path)
        visited_stack: &mut HashSet<PathBuf>,
    ) -> Result<ResolvedProjectId, ProjectError> {
        let canonical_manifest = manifest_path
            .canonicalize()
            .map_err(|e| ProjectError::InvalidProjectManifest(format!("Failed to canonicalize {}: {}", manifest_path.display(), e)))?;

        let root_dir = canonical_manifest
            .parent()
            .ok_or_else(|| ProjectError::InvalidProjectManifest("Manifest path has no parent directory".to_string()))?
            .to_path_buf();

        let source_identity = ProjectSourceIdentity::from_path(&root_dir);

        if visited_stack.contains(&canonical_manifest) {
            let start_idx = visiting.iter().position(|(_, path)| path == &canonical_manifest).unwrap_or(0);
            let mut chain = Vec::new();
            for (name, _) in &visiting[start_idx..] {
                chain.push(name.clone());
            }
            chain.push(visiting[start_idx].0.clone());
            return Err(ProjectError::ProjectDependencyCycle { chain: chain.join(" → ") });
        }

        if let Some(&existing_id) = self.roots.get(&source_identity) {
            return Ok(existing_id);
        }

        let raw_manifest = ProjectManifest::load_file(&canonical_manifest)?;
        let validated = raw_manifest.validate()?;

        let source_root = if validated.source.is_absolute() {
            validated.source.clone()
        } else {
            root_dir.join(&validated.source)
        };

        let source_root = source_root.canonicalize().map_err(|_| ProjectError::InvalidSourceRoot(source_root.clone()))?;

        if !source_root.is_dir() {
            return Err(ProjectError::InvalidSourceRoot(source_root));
        }

        let root_package_file = source_root.join("package.ph");
        if !root_package_file.is_file() {
            return Err(ProjectError::MissingRootPackage(source_root));
        }

        let entry = if let Some(ref entry_str) = validated.entry {
            let parts: Vec<&str> = entry_str.split('.').collect();
            // First part is namespace; remainder are relative components
            let mut components = Vec::new();
            for part in &parts[1..] {
                let comp = ModuleComponent::from_identifier(part).map_err(|e| ProjectError::InvalidEntry(entry_str.clone(), e.to_string()))?;
                components.push(comp);
            }
            Some(ModulePath::from_components(components))
        } else {
            None
        };

        visiting.push((validated.name.clone(), canonical_manifest.clone()));
        visited_stack.insert(canonical_manifest.clone());

        let mut resolved_dependencies = BTreeMap::new();

        for (alias, (_raw_alias, spec)) in &validated.dependencies {
            let dep_manifest_path = match spec {
                DependencySpec::Path { path } => {
                    let dep_dir = if path.is_absolute() { path.clone() } else { root_dir.join(path) };
                    let dep_manifest = dep_dir.join("project.toml");
                    if !dep_manifest.is_file() {
                        return Err(ProjectError::PathDependencyNotFound(dep_manifest));
                    }
                    dep_manifest
                }
                DependencySpec::Package { package, version } => {
                    let res = dep_provider.resolve_package(package, version)?;
                    res.manifest_path
                }
            };

            let dep_id = self.resolve_project_recursive(&dep_manifest_path, dep_provider, visiting, visited_stack)?;

            resolved_dependencies.insert(alias.clone(), dep_id);
        }

        visiting.pop();
        visited_stack.remove(&canonical_manifest);

        // A workspace update can reload a mutable manifest while retaining its
        // graph slot. New projects still allocate monotonically after all slots.
        let existing = self.projects.iter().find(|project| project.persistent_project && project.source_identity == source_identity).map(|project| project.id);
        let next_id = existing.unwrap_or_else(|| ResolvedProjectId::from_raw((self.projects.len() + 1) as u32));

        let mut import_roots = BTreeMap::new();
        let universe_comp = ModuleComponent::from_identifier("universe").expect("valid identifier");
        import_roots.insert(universe_comp, (ImportRootTarget::Universe, false));
        import_roots.insert(validated.namespace.clone(), (ImportRootTarget::Resolved(next_id), true));
        for (alias, dep_id) in &resolved_dependencies {
            import_roots.insert(alias.clone(), (ImportRootTarget::Resolved(*dep_id), false));
        }

        let resolved_project = ResolvedProject {
            id: next_id,
            name: validated.name.clone(),
            namespace: validated.namespace.clone(),
            root_dir,
            source_root,
            entry,
            dependencies: resolved_dependencies,
            import_roots,
            source_identity: source_identity.clone(),
            persistent_project: true,
            manifest: Some(validated),
        };

        if let Some(existing) = existing {
            self.projects[(existing.raw() - 1) as usize] = resolved_project;
        } else {
            self.projects.push(resolved_project);
        }
        self.roots.insert(source_identity, next_id);

        Ok(next_id)
    }

    /// Loads a legacy synthetic compatibility context without a `project.toml`.
    ///
    /// This helper does not establish filesystem package ownership and must not
    /// be used for package consumers. Use [`Self::load_standalone_package`] when
    /// `package.ph` semantics are required.
    pub fn load_synthetic_root(&mut self, name: &str, source_root: impl AsRef<Path>, entry_component: &str) -> Result<ResolvedProjectId, ProjectError> {
        let source_root = source_root.as_ref();
        let canonical_root = source_root
            .canonicalize()
            .map_err(|e| ProjectError::InvalidProjectManifest(format!("Failed to canonicalize {}: {}", source_root.display(), e)))?;

        let source_identity = ProjectSourceIdentity::from_path(&canonical_root);
        if let Some(&id) = self.roots.get(&source_identity) {
            return Ok(id);
        }

        let namespace = ModuleComponent::from_identifier(&name.replace('-', "_"))
            .map_err(|e| ProjectError::InvalidProjectManifest(format!("Invalid project identifier: {e}")))?;
        let entry_comp = ModuleComponent::from_identifier(&entry_component.replace('-', "_"))
            .map_err(|e| ProjectError::InvalidProjectManifest(format!("Invalid entry identifier: {e}")))?;
        let entry = Some(ModulePath::from_components(vec![entry_comp]));

        let next_id = ResolvedProjectId::from_raw((self.projects.len() + 1) as u32);

        let mut import_roots = BTreeMap::new();
        let universe_comp = ModuleComponent::from_identifier("universe").expect("valid identifier");
        import_roots.insert(universe_comp, (ImportRootTarget::Universe, false));
        import_roots.insert(namespace.clone(), (ImportRootTarget::Resolved(next_id), true));

        let resolved_project = ResolvedProject {
            id: next_id,
            name: name.to_string(),
            namespace,
            root_dir: canonical_root.clone(),
            source_root: canonical_root,
            entry,
            dependencies: BTreeMap::new(),
            import_roots,
            source_identity: source_identity.clone(),
            persistent_project: false,
            manifest: None,
        };

        self.projects.push(resolved_project);
        self.roots.insert(source_identity, next_id);
        Ok(next_id)
    }

    /// Loads a standalone package rooted at `package_root`.
    ///
    /// Requires `package.ph` to exist in `package_root`.
    pub fn load_standalone_package(&mut self, package_root: impl AsRef<Path>, entry: Option<&str>) -> Result<ResolvedProjectId, ProjectError> {
        let package_root = package_root.as_ref();
        let canonical_root = package_root
            .canonicalize()
            .map_err(|e| ProjectError::InvalidProjectManifest(format!("Failed to canonicalize {}: {}", package_root.display(), e)))?;

        let pkg_file = canonical_root.join("package.ph");
        if !pkg_file.is_file() {
            return Err(ProjectError::MissingRootPackage(canonical_root));
        }

        let source_identity = ProjectSourceIdentity::from_path(&canonical_root);
        if let Some(&id) = self.roots.get(&source_identity) {
            return Ok(id);
        }

        let raw_name = canonical_root.file_name().and_then(|n| n.to_str()).unwrap_or("package");
        let safe_name = raw_name.replace('-', "_");
        let namespace = ModuleComponent::from_identifier(&safe_name).unwrap_or_else(|_| ModuleComponent::from_identifier("package").expect("valid identifier"));

        let entry_path = if let Some(entry_name) = entry {
            let entry_comp = ModuleComponent::from_identifier(&entry_name.replace('-', "_"))
                .map_err(|e| ProjectError::InvalidProjectManifest(format!("Invalid entry identifier: {e}")))?;
            Some(ModulePath::from_components(vec![entry_comp]))
        } else {
            None
        };

        let next_id = ResolvedProjectId::from_raw((self.projects.len() + 1) as u32);

        let mut import_roots = BTreeMap::new();
        let universe_comp = ModuleComponent::from_identifier("universe").expect("valid identifier");
        import_roots.insert(universe_comp, (ImportRootTarget::Universe, false));
        import_roots.insert(namespace.clone(), (ImportRootTarget::Resolved(next_id), true));

        let resolved_project = ResolvedProject {
            id: next_id,
            name: raw_name.to_string(),
            namespace,
            root_dir: canonical_root.clone(),
            source_root: canonical_root,
            entry: entry_path,
            dependencies: BTreeMap::new(),
            import_roots,
            source_identity: source_identity.clone(),
            persistent_project: false,
            manifest: None,
        };

        self.projects.push(resolved_project);
        self.roots.insert(source_identity, next_id);
        Ok(next_id)
    }
}

/// Discovers the nearest enclosing project root directory containing `project.toml`.
pub fn discover_owning_project(source_path: &Path) -> Result<Option<PathBuf>, ProjectError> {
    let mut current = if source_path.is_file() {
        source_path.parent().map(|p| p.to_path_buf())
    } else {
        Some(source_path.to_path_buf())
    };

    while let Some(dir) = current {
        let manifest = dir.join("project.toml");
        if manifest.is_file() {
            return Ok(Some(dir));
        }
        current = dir.parent().map(|p| p.to_path_buf());
    }

    Ok(None)
}

/// Discovers the root directory of an enclosing standalone package.
///
/// A standalone package hierarchy is established only by contiguous `package.ph`
/// files. It walks up directory ancestors starting from `source_path`'s parent
/// (or `source_path` itself if it is a directory), continuing as long as each
/// directory contains `package.ph` and does not encounter a `project.toml` boundary.
pub fn discover_standalone_package_root(source_path: &Path) -> Option<PathBuf> {
    let canonical = crate::source::canonicalize_path(source_path);
    let mut current_dir = if canonical.is_dir() {
        if !canonical.join("package.ph").is_file() {
            return None;
        }
        canonical
    } else if canonical.file_name().is_some_and(|n| n == "package.ph") {
        canonical.parent()?.to_path_buf()
    } else {
        let parent = canonical.parent()?;
        if !parent.join("package.ph").is_file() {
            return None;
        }
        parent.to_path_buf()
    };

    let mut root = current_dir.clone();
    while let Some(parent) = current_dir.parent() {
        if parent.join("project.toml").is_file() {
            break;
        }
        if parent.join("package.ph").is_file() {
            root = parent.to_path_buf();
            current_dir = parent.to_path_buf();
        } else {
            break;
        }
    }

    Some(root)
}
