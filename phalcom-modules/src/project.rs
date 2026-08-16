//! Project universe, resolved projects, and project dependency graph management.

use crate::error::ProjectError;
use crate::identity::{ModuleComponent, ModulePath, ProjectSourceIdentity, ResolvedProjectId};
use crate::manifest::{DependencyProvider, DependencySpec, NullDependencyProvider, ProjectManifest};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::{Path, PathBuf};

/// A fully resolved project in the `ProjectUniverse`.
#[derive(Debug, Clone)]
pub struct ResolvedProject {
    pub id: ResolvedProjectId,
    pub name: String,                                               // original name for diagnostics
    pub namespace: ModuleComponent,                                 // canonical snake_case namespace
    pub root_dir: PathBuf,                                          // directory containing project.toml
    pub source_root: PathBuf,                                       // directory containing source code (e.g. root_dir/src)
    pub entry: Option<ModulePath>,                                  // project-relative entry module path
    pub dependencies: BTreeMap<ModuleComponent, ResolvedProjectId>, // alias -> resolved project id
    pub source_identity: ProjectSourceIdentity,
}

impl ResolvedProject {
    /// Builds the import root table for this project:
    /// Maps each recognized root component (self namespace + dependency aliases) to (ResolvedProjectId, is_self).
    pub fn import_roots(&self) -> HashMap<ModuleComponent, (ResolvedProjectId, bool)> {
        let mut roots = HashMap::new();
        roots.insert(self.namespace.clone(), (self.id, true));
        for (alias, dep_id) in &self.dependencies {
            roots.insert(alias.clone(), (*dep_id, false));
        }
        roots
    }
}

/// The set of all resolved projects participating in a compilation or analysis session.
#[derive(Debug)]
pub struct ProjectUniverse {
    projects: Vec<ResolvedProject>,
    roots: BTreeMap<ProjectSourceIdentity, ResolvedProjectId>,
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
        }
    }

    /// Returns a reference to all resolved projects.
    pub fn projects(&self) -> &[ResolvedProject] {
        &self.projects
    }

    /// Gets a resolved project by its ID.
    pub fn get_project(&self, id: ResolvedProjectId) -> Option<&ResolvedProject> {
        self.projects.get(id.0 as usize)
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
            let mut chain = Vec::new();
            for (name, _) in visiting.iter() {
                chain.push(name.clone());
            }
            chain.push(visiting.first().map(|(n, _)| n.clone()).unwrap_or_else(|| "root".to_string()));
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

        let entry = if let Some(entry_str) = validated.entry {
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
                DependencySpec::Package { package, version } => {
                    let res = dep_provider.resolve_package(&package, &version)?;
                    res.manifest_path
                }
            };

            let dep_id = self.resolve_project_recursive(&dep_manifest_path, dep_provider, visiting, visited_stack)?;

            resolved_dependencies.insert(alias, dep_id);
        }

        visiting.pop();
        visited_stack.remove(&canonical_manifest);

        let next_id = ResolvedProjectId(self.projects.len() as u32);
        let resolved_project = ResolvedProject {
            id: next_id,
            name: validated.name,
            namespace: validated.namespace,
            root_dir,
            source_root,
            entry,
            dependencies: resolved_dependencies,
            source_identity: source_identity.clone(),
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
