//! Project universe, resolved projects, and project dependency graph management.

use crate::error::ProjectError;
use crate::identity::{
    BuiltinProject, ImportRootTarget, ModuleComponent, ModulePath, ProjectSourceIdentity, ResolvedProjectId,
    SyntheticProjectId, SyntheticProjectIdAllocator,
};
use crate::manifest::{DependencyProvider, DependencySpec, NullDependencyProvider, ProjectManifest};
use std::collections::{BTreeMap, HashSet};
use std::path::{Path, PathBuf};

/// A fully resolved persistent project in the `ProjectUniverse`.
#[derive(Debug, Clone)]
pub struct ResolvedProject {
    pub id: ResolvedProjectId,
    pub name: String,
    pub namespace: ModuleComponent,
    pub root_dir: PathBuf,
    pub source_root: PathBuf,
    pub entry: Option<ModulePath>,
    pub dependencies: BTreeMap<ModuleComponent, ResolvedProjectId>,
    pub import_roots: BTreeMap<ModuleComponent, (ImportRootTarget, bool)>,
    pub source_identity: ProjectSourceIdentity,
}

impl ResolvedProject {
    pub fn import_roots(&self) -> &BTreeMap<ModuleComponent, (ImportRootTarget, bool)> {
        &self.import_roots
    }
}

/// Persistent project graph plus the session allocator for synthetic ownership
/// domains. Synthetic units are deliberately not inserted into `projects`.
#[derive(Debug)]
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
            synthetic_ids: SyntheticProjectIdAllocator::new(),
        }
    }

    pub fn projects(&self) -> &[ResolvedProject] {
        &self.projects
    }

    pub fn get_project(&self, id: ResolvedProjectId) -> Option<&ResolvedProject> {
        self.projects.get(id.index())
    }

    /// Allocates a fresh synthetic ownership domain for a standalone package,
    /// standalone module, inline unit, REPL, or other ephemeral source family.
    pub fn allocate_synthetic_id(&mut self) -> SyntheticProjectId {
        self.synthetic_ids.allocate()
    }

    pub fn load_root(&mut self, manifest_path: impl AsRef<Path>) -> Result<ResolvedProjectId, ProjectError> {
        let dep_provider = NullDependencyProvider;
        self.load_root_with_provider(manifest_path, &dep_provider)
    }

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

        let mut visiting = Vec::new();
        let mut visited_stack = HashSet::new();
        self.resolve_project_recursive(&canonical_manifest, dep_provider, &mut visiting, &mut visited_stack)
    }

    fn resolve_project_recursive(
        &mut self,
        manifest_path: &Path,
        dep_provider: &dyn DependencyProvider,
        visiting: &mut Vec<(String, PathBuf)>,
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
            let mut chain = visiting[start_idx..].iter().map(|(name, _)| name.clone()).collect::<Vec<_>>();
            if let Some((name, _)) = visiting.get(start_idx) {
                chain.push(name.clone());
            }
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
        if !source_root.join("package.ph").is_file() {
            return Err(ProjectError::MissingRootPackage(source_root));
        }

        let entry = if let Some(entry_str) = validated.entry.clone() {
            let parts = entry_str.split('.').collect::<Vec<_>>();
            let mut components = Vec::new();
            for part in &parts[1..] {
                components.push(
                    ModuleComponent::from_identifier(part)
                        .map_err(|e| ProjectError::InvalidEntry(entry_str.clone(), e.to_string()))?,
                );
            }
            Some(ModulePath::from_components(components))
        } else {
            None
        };

        visiting.push((validated.display_name.clone(), canonical_manifest.clone()));
        visited_stack.insert(canonical_manifest.clone());

        let mut resolved_dependencies = BTreeMap::new();
        for (alias, (_raw_alias, spec)) in validated.dependencies {
            let dep_manifest_path = match spec {
                DependencySpec::Path { path } => {
                    let dep_dir = if path.is_absolute() { path } else { root_dir.join(path) };
                    let dep_manifest = dep_dir.join("project.toml");
                    if !dep_manifest.is_file() {
                        return Err(ProjectError::PathDependencyNotFound(dep_manifest));
                    }
                    dep_manifest
                }
                DependencySpec::Package { package, version } => dep_provider.resolve_package(&package, &version)?.manifest_path,
            };
            let dep_id = self.resolve_project_recursive(&dep_manifest_path, dep_provider, visiting, visited_stack)?;
            resolved_dependencies.insert(alias, dep_id);
        }

        visiting.pop();
        visited_stack.remove(&canonical_manifest);

        let next_id = ResolvedProjectId::from_index(self.projects.len());
        let mut import_roots = BTreeMap::new();
        for builtin in [BuiltinProject::Universe, BuiltinProject::Std] {
            let component = ModuleComponent::from_identifier(builtin.root_name()).expect("builtin root is canonical");
            import_roots.insert(component, (ImportRootTarget::Builtin(builtin), false));
        }
        // Transitional compatibility: `core` is a complete alias for the
        // builtin Universe root and never owns a separate semantic identity.
        import_roots.insert(
            ModuleComponent::from_identifier("core").expect("canonical compatibility root"),
            (ImportRootTarget::Builtin(BuiltinProject::Universe), false),
        );
        import_roots.insert(validated.namespace.clone(), (ImportRootTarget::Resolved(next_id), true));
        for (alias, dep_id) in &resolved_dependencies {
            import_roots.insert(alias.clone(), (ImportRootTarget::Resolved(*dep_id), false));
        }

        let resolved_project = ResolvedProject {
            id: next_id,
            name: validated.display_name,
            namespace: validated.namespace,
            root_dir,
            source_root,
            entry,
            dependencies: resolved_dependencies,
            import_roots,
            source_identity: source_identity.clone(),
        };

        self.projects.push(resolved_project);
        self.roots.insert(source_identity, next_id);
        Ok(next_id)
    }
}

/// Discovers the nearest enclosing persistent project root containing
/// `project.toml`.
pub fn discover_owning_project(source_path: &Path) -> Result<Option<PathBuf>, ProjectError> {
    let mut current = if source_path.is_file() {
        source_path.parent().map(Path::to_path_buf)
    } else {
        Some(source_path.to_path_buf())
    };

    while let Some(dir) = current {
        if dir.join("project.toml").is_file() {
            return Ok(Some(dir));
        }
        current = dir.parent().map(Path::to_path_buf);
    }
    Ok(None)
}

/// Discovers the outermost contiguous standalone-package root owning a source.
/// A gap without `package.ph` ends package authority; a persistent project
/// boundary always wins and therefore terminates standalone discovery.
pub fn discover_standalone_package_root(source_path: &Path) -> Result<Option<PathBuf>, ProjectError> {
    let canonical = source_path
        .canonicalize()
        .map_err(|e| ProjectError::InvalidProjectManifest(format!("Failed to canonicalize {}: {e}", source_path.display())))?;
    let mut current = if canonical.is_file() {
        canonical.parent().map(Path::to_path_buf)
    } else {
        Some(canonical)
    };
    let mut outermost = None;

    while let Some(dir) = current {
        if dir.join("project.toml").is_file() {
            break;
        }
        if dir.join("package.ph").is_file() {
            outermost = Some(dir.clone());
        } else if outermost.is_some() {
            break;
        }
        current = dir.parent().map(Path::to_path_buf);
    }
    Ok(outermost)
}
