//! Project universe, resolved projects, and project dependency graph management.

use crate::error::ProjectError;
use crate::identity::{
    ImportRootTarget, ModuleComponent, ModulePath, ProjectSourceIdentity, ResolvedProjectId, SyntheticProjectId, SyntheticProjectIdAllocator,
};
use crate::manifest::{DependencyProvider, DependencySpec, NullDependencyProvider, ProjectManifest};
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

        let next_id = ResolvedProjectId::from_raw((self.projects.len() + 1) as u32);

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

        self.projects.push(resolved_project);
        self.roots.insert(source_identity, next_id);

        Ok(next_id)
    }

    /// Loads a synthetic single-module or package project without a `project.toml`.
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
