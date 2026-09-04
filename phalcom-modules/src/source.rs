//! Source provider abstraction and filesystem implementation.

use crate::error::{ModuleResolutionError, SourceError};
use crate::identity::{ModuleComponent, ModuleId, ModulePath, ResolvedProjectId, SourceId, SourceLocation};
use crate::project::ResolvedProject;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

/// Ownership classification of an entry before compilation/linking.
///
/// This classification is authoritative across the compiler, semantic workspace,
/// and LSP. A directory is only a package when `package.ph` exists.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EntryOwnership {
    ProjectOwned { project: ResolvedProjectId },
    StandalonePackageOwned { package_root: PathBuf },
    StandaloneModule { file: PathBuf },
    Inline { synthetic: crate::identity::SyntheticProjectId },
}

/// Canonicalizes a path, resolving symlinks. If `path` does not exist on disk
/// (e.g., an in-memory buffer overlay for an unsaved file), canonicalizes its
/// nearest existing parent directory and appends the remaining relative path.
pub fn canonicalize_path(path: &Path) -> PathBuf {
    if let Ok(c) = path.canonicalize() {
        return c;
    }
    let mut current = path;
    let mut suffixes = Vec::new();
    while let Some(parent) = current.parent() {
        if let Some(file_name) = current.file_name() {
            suffixes.push(file_name);
        }
        if let Ok(canon_parent) = parent.canonicalize() {
            let mut result = canon_parent;
            for suffix in suffixes.into_iter().rev() {
                result.push(suffix);
            }
            return result;
        }
        current = parent;
    }
    path.to_path_buf()
}

/// Authoritative classification of an entry source before compilation/linking.
///
/// Precedence order:
/// 1. Enclosing persistent Project (with `project.toml`) -> `ProjectOwned`
/// 2. Enclosing standalone Package hierarchy (with `package.ph`) -> `StandalonePackageOwned`
/// 3. Otherwise -> `StandaloneModule`
pub fn classify_entry_ownership(
    source_path: &Path,
    universe: &mut crate::project::ProjectUniverse,
) -> Result<EntryOwnership, crate::error::ProjectError> {
    let canonical = canonicalize_path(source_path);

    if let Some(project_root) = crate::project::discover_owning_project(&canonical)? {
        let manifest_path = project_root.join("project.toml");
        let project_id = universe.load_root(&manifest_path)?;
        return Ok(EntryOwnership::ProjectOwned { project: project_id });
    }

    if let Some(package_root) = crate::project::discover_standalone_package_root(&canonical) {
        return Ok(EntryOwnership::StandalonePackageOwned { package_root });
    }

    Ok(EntryOwnership::StandaloneModule { file: canonical })
}

/// Module kind: an ordinary `.ph` file or a package descriptor.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ModuleKind {
    /// An ordinary `.ph` source file.
    Module,
    /// A `package.ph` package descriptor.
    Package,
}

impl ModuleKind {
    /// Whether the unit has package semantics (`package.ph`, exposure, children).
    pub const fn is_package(self) -> bool {
        matches!(self, Self::Package)
    }

    /// Backwards-compatible alias for `is_package`.
    pub const fn is_package_like(self) -> bool {
        self.is_package()
    }
}

/// A located source unit ready for reading and parsing.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SourceUnit {
    pub id: ModuleId,
    pub kind: ModuleKind,
    pub source: SourceLocation,
}

/// Retained parsed module source artifact preventing redundant reparsing.
#[derive(Clone, Debug)]
pub struct ParsedModuleUnit {
    pub id: ModuleId,
    pub kind: ModuleKind,
    pub source: Option<SourceLocation>,
    pub text: Arc<str>,
    pub program: Arc<phalcom_ast::ast::Program>,
}

impl ParsedModuleUnit {
    pub fn new(id: ModuleId, kind: ModuleKind, source: Option<SourceLocation>, text: Arc<str>, program: Arc<phalcom_ast::ast::Program>) -> Self {
        Self {
            id,
            kind,
            source,
            text,
            program,
        }
    }
}

/// An in-memory overlay source artifact for open editor buffers.
#[derive(Clone, Debug)]
pub struct SourceOverlay {
    pub id: ModuleId,
    pub kind: ModuleKind,
    pub source: SourceLocation,
    pub text: Arc<str>,
}

impl SourceOverlay {
    pub fn new(id: ModuleId, kind: ModuleKind, source: SourceLocation, text: Arc<str>) -> Self {
        Self { id, kind, source, text }
    }
}

/// Overlay-capable source provider layering in-memory documents over a base provider.
#[derive(Debug)]
pub struct OverlaySourceProvider<P> {
    base: P,
    overlays_by_module: std::sync::RwLock<std::collections::BTreeMap<ModuleId, SourceOverlay>>,
    overlays_by_source: std::sync::RwLock<std::collections::BTreeMap<SourceId, ModuleId>>,
}

impl<P> OverlaySourceProvider<P> {
    pub fn new(base: P) -> Self {
        Self {
            base,
            overlays_by_module: std::sync::RwLock::new(std::collections::BTreeMap::new()),
            overlays_by_source: std::sync::RwLock::new(std::collections::BTreeMap::new()),
        }
    }

    pub fn set_overlay(&self, overlay: SourceOverlay) {
        let mut by_mod = self.overlays_by_module.write().unwrap();
        let mut by_src = self.overlays_by_source.write().unwrap();
        by_src.insert(overlay.source.source_id.clone(), overlay.id.clone());
        by_mod.insert(overlay.id.clone(), overlay);
    }

    pub fn remove_overlay(&self, module: &ModuleId) {
        let mut by_mod = self.overlays_by_module.write().unwrap();
        let mut by_src = self.overlays_by_source.write().unwrap();
        if let Some(overlay) = by_mod.remove(module) {
            by_src.remove(&overlay.source.source_id);
        }
    }

    pub fn clear_overlays(&self) {
        let mut by_mod = self.overlays_by_module.write().unwrap();
        let mut by_src = self.overlays_by_source.write().unwrap();
        by_mod.clear();
        by_src.clear();
    }

    pub fn base(&self) -> &P {
        &self.base
    }
}

impl<P: SourceProvider> SourceProvider for OverlaySourceProvider<P> {
    fn locate(&self, project: &ResolvedProject, path: &ModulePath) -> Result<SourceUnit, ModuleResolutionError> {
        let candidate_id = ModuleId::resolved(project.id, path.clone());
        {
            let by_mod = self.overlays_by_module.read().unwrap();
            if let Some(overlay) = by_mod.get(&candidate_id) {
                return Ok(SourceUnit {
                    id: overlay.id.clone(),
                    kind: overlay.kind,
                    source: overlay.source.clone(),
                });
            }
        }
        self.base.locate(project, path)
    }

    fn read(&self, source: &SourceId) -> Result<Arc<str>, SourceError> {
        {
            let by_src = self.overlays_by_source.read().unwrap();
            if let Some(mod_id) = by_src.get(source) {
                let by_mod = self.overlays_by_module.read().unwrap();
                if let Some(overlay) = by_mod.get(mod_id) {
                    return Ok(overlay.text.clone());
                }
            }
        }
        self.base.read(source)
    }
}

/// Trait abstracting source location and reading.
pub trait SourceProvider {
    fn locate(&self, project: &ResolvedProject, path: &ModulePath) -> Result<SourceUnit, ModuleResolutionError>;

    fn read(&self, source: &SourceId) -> Result<Arc<str>, SourceError>;
}

type ResolutionCache = HashMap<(u64, ResolvedProjectId, ModulePath), Result<SourceUnit, ModuleResolutionError>>;

/// Filesystem source provider with resolution caching and kebab/snake convention handling.
#[derive(Debug)]
pub struct FilesystemSourceProvider {
    generation: AtomicU64,
    cache: Mutex<ResolutionCache>,
    source_cache: Mutex<HashMap<(u64, SourceId), Arc<str>>>,
    source_id_to_module: Mutex<HashMap<(u64, SourceId), ModuleId>>,
}

impl Default for FilesystemSourceProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl FilesystemSourceProvider {
    pub fn new() -> Self {
        Self {
            generation: AtomicU64::new(0),
            cache: Mutex::new(HashMap::new()),
            source_cache: Mutex::new(HashMap::new()),
            source_id_to_module: Mutex::new(HashMap::new()),
        }
    }

    /// Starts a new resolver generation and clears every generation-scoped cache.
    pub fn clear_cache(&self) {
        self.generation.fetch_add(1, Ordering::AcqRel);
        self.cache.lock().unwrap().clear();
        self.source_cache.lock().unwrap().clear();
        self.source_id_to_module.lock().unwrap().clear();
    }

    pub fn generation(&self) -> u64 {
        self.generation.load(Ordering::Acquire)
    }

    fn locate_internal(&self, project: &ResolvedProject, path: &ModulePath) -> Result<SourceUnit, ModuleResolutionError> {
        let components = path.components();
        let mut current_dir = project.source_root.clone();

        // 1. Root package case: `[]` -> `<source-root>/package.ph`
        if components.is_empty() {
            let pkg_file = current_dir.join("package.ph");
            if pkg_file.is_file() {
                let canonical = pkg_file.canonicalize().map_err(|e| SourceError::Io(e.to_string()))?;

                // Confinement check: canonical path must start with project source root
                if !canonical.starts_with(&project.source_root) {
                    return Err(ModuleResolutionError::ImportOutsideSourceRoot(canonical, project.source_root.clone()));
                }

                let module_id = ModuleId {
                    project: project.id.into(),
                    path: path.clone(),
                };
                let source_id = SourceId(canonical.to_string_lossy().into());

                {
                    let mut rev_map = self.source_id_to_module.lock().unwrap();
                    let key = (self.generation(), source_id.clone());
                    if let Some(existing_mod) = rev_map.get(&key) {
                        if existing_mod != &module_id {
                            return Err(ModuleResolutionError::DuplicateSourceIdentity(format!(
                                "source {} maps to both {} and {}",
                                canonical.display(),
                                existing_mod,
                                module_id
                            )));
                        }
                    } else {
                        rev_map.insert(key, module_id.clone());
                    }
                }

                return Ok(SourceUnit {
                    id: module_id,
                    kind: ModuleKind::Package,
                    source: SourceLocation {
                        source_id,
                        display_path: pkg_file,
                    },
                });
            } else {
                return Err(ModuleResolutionError::PackageNotFoundError(format!("{}:<root>", project.name)));
            }
        }

        // 2. Intermediate components: all must be package directories containing `package.ph`
        for comp in &components[..components.len() - 1] {
            let (dir_path, _is_kebab) = self.find_directory(&current_dir, comp)?;

            // Check for nested project boundary: if `project.toml` exists in this subdir and is not project root
            if dir_path != project.root_dir && dir_path.join("project.toml").is_file() {
                return Err(ModuleResolutionError::NestedProjectBoundary(dir_path));
            }

            let pkg_file = dir_path.join("package.ph");
            if !pkg_file.is_file() {
                return Err(ModuleResolutionError::PackageNotFoundError(format!(
                    "Directory '{}' is missing package.ph",
                    dir_path.display()
                )));
            }
            current_dir = dir_path;
        }

        // 3. Final component: can be `<comp>.ph` (Module) or `<comp>/package.ph` (Package)
        let last_comp = components.last().unwrap();
        let (file_candidate, dir_candidate) = self.find_final_candidates(&current_dir, last_comp)?;

        let has_file = file_candidate.as_ref().map(|p| p.is_file()).unwrap_or(false);
        let has_dir_pkg = dir_candidate.as_ref().map(|d| d.join("package.ph").is_file()).unwrap_or(false);

        if has_file && has_dir_pkg {
            return Err(ModuleResolutionError::AmbiguousModule {
                name: last_comp.as_str().to_string(),
                kebab_path: file_candidate.unwrap(),
                snake_path: dir_candidate.unwrap().join("package.ph"),
            });
        }

        if has_file {
            let file_path = file_candidate.unwrap();
            let canonical = file_path.canonicalize().map_err(|e| SourceError::Io(e.to_string()))?;

            // Confinement check: canonical path must start with project source root
            if !canonical.starts_with(&project.source_root) {
                return Err(ModuleResolutionError::ImportOutsideSourceRoot(canonical, project.source_root.clone()));
            }

            let module_id = ModuleId {
                project: project.id.into(),
                path: path.clone(),
            };
            let source_id = SourceId(canonical.to_string_lossy().into());

            {
                let generation = self.generation.load(Ordering::Relaxed);
                let mut rev_map = self.source_id_to_module.lock().unwrap();
                if let Some(existing_mod) = rev_map.get(&(generation, source_id.clone())) {
                    if existing_mod != &module_id {
                        return Err(ModuleResolutionError::DuplicateSourceIdentity(format!(
                            "source {} maps to both {} and {}",
                            canonical.display(),
                            existing_mod,
                            module_id
                        )));
                    }
                } else {
                    rev_map.insert((generation, source_id.clone()), module_id.clone());
                }
            }

            return Ok(SourceUnit {
                id: module_id,
                kind: ModuleKind::Module,
                source: SourceLocation {
                    source_id,
                    display_path: file_path,
                },
            });
        }

        if has_dir_pkg {
            let dir_path = dir_candidate.unwrap();
            if dir_path != project.root_dir && dir_path.join("project.toml").is_file() {
                return Err(ModuleResolutionError::NestedProjectBoundary(dir_path));
            }

            let pkg_file = dir_path.join("package.ph");
            let canonical = pkg_file.canonicalize().map_err(|e| SourceError::Io(e.to_string()))?;

            if !canonical.starts_with(&project.source_root) {
                return Err(ModuleResolutionError::ImportOutsideSourceRoot(canonical, project.source_root.clone()));
            }

            let module_id = ModuleId {
                project: project.id.into(),
                path: path.clone(),
            };
            let source_id = SourceId(canonical.to_string_lossy().into());

            {
                let generation = self.generation.load(Ordering::Relaxed);
                let mut rev_map = self.source_id_to_module.lock().unwrap();
                if let Some(existing_mod) = rev_map.get(&(generation, source_id.clone())) {
                    if existing_mod != &module_id {
                        return Err(ModuleResolutionError::DuplicateSourceIdentity(format!(
                            "source {} maps to both {} and {}",
                            canonical.display(),
                            existing_mod,
                            module_id
                        )));
                    }
                } else {
                    rev_map.insert((generation, source_id.clone()), module_id.clone());
                }
            }

            return Ok(SourceUnit {
                id: module_id,
                kind: ModuleKind::Package,
                source: SourceLocation {
                    source_id,
                    display_path: pkg_file,
                },
            });
        }

        Err(ModuleResolutionError::ModuleNotFound(format!("{}.{}", project.namespace, path)))
    }

    /// Looks up the one canonical physical spelling for a logical component.
    fn find_directory(&self, parent: &Path, comp: &ModuleComponent) -> Result<(PathBuf, bool), ModuleResolutionError> {
        let logical = comp.as_str();
        let physical = comp.to_kebab();
        if logical != physical {
            let noncanonical = parent.join(logical);
            if noncanonical.is_dir() {
                return Err(ModuleResolutionError::NonCanonicalPhysicalName {
                    path: noncanonical,
                    expected: physical,
                });
            }
        }
        let canonical = parent.join(&physical);
        if canonical.is_dir() {
            Ok((canonical, true))
        } else {
            Err(ModuleResolutionError::PackageNotFoundError(format!(
                "Component '{}' directory not found in {}",
                logical,
                parent.display()
            )))
        }
    }

    /// Finds the canonical physical module and package candidates.
    fn find_final_candidates(&self, parent: &Path, comp: &ModuleComponent) -> Result<(Option<PathBuf>, Option<PathBuf>), ModuleResolutionError> {
        let logical = comp.as_str();
        let physical = comp.to_kebab();
        if logical != physical {
            let noncanonical_file = parent.join(format!("{logical}.ph"));
            if noncanonical_file.is_file() {
                return Err(ModuleResolutionError::NonCanonicalPhysicalName {
                    path: noncanonical_file,
                    expected: format!("{physical}.ph"),
                });
            }
            let noncanonical_dir = parent.join(logical);
            if noncanonical_dir.is_dir() {
                return Err(ModuleResolutionError::NonCanonicalPhysicalName {
                    path: noncanonical_dir,
                    expected: physical,
                });
            }
        }
        let file = parent.join(format!("{physical}.ph"));
        let dir = parent.join(&physical);
        Ok((file.is_file().then_some(file), dir.is_dir().then_some(dir)))
    }
}

impl SourceProvider for FilesystemSourceProvider {
    fn locate(&self, project: &ResolvedProject, path: &ModulePath) -> Result<SourceUnit, ModuleResolutionError> {
        let key = (self.generation(), project.id, path.clone());
        {
            let cache = self.cache.lock().unwrap();
            if let Some(res) = cache.get(&key) {
                return res.clone();
            }
        }

        let result = self.locate_internal(project, path);
        let mut cache = self.cache.lock().unwrap();
        cache.insert(key, result.clone());
        result
    }

    fn read(&self, source: &SourceId) -> Result<Arc<str>, SourceError> {
        {
            let cache = self.source_cache.lock().unwrap();
            if let Some(content) = cache.get(&(self.generation(), source.clone())) {
                return Ok(content.clone());
            }
        }

        let path = Path::new(&*source.0);
        let content = std::fs::read_to_string(path).map_err(|e| SourceError::Io(format!("Failed to read {}: {}", path.display(), e)))?;

        let arc: Arc<str> = Arc::from(content);
        let mut cache = self.source_cache.lock().unwrap();
        cache.insert((self.generation(), source.clone()), arc.clone());
        Ok(arc)
    }
}

/// Canonical reverse physical-to-logical source path resolution.
pub fn resolve_source_path(project: &ResolvedProject, path: &Path) -> Result<SourceUnit, ModuleResolutionError> {
    let canonical_path = canonicalize_path(path);
    let canonical_source_root = canonicalize_path(&project.source_root);

    let relative = match canonical_path.strip_prefix(&canonical_source_root) {
        Ok(r) => r,
        Err(_) => return Err(ModuleResolutionError::ImportOutsideSourceRoot(canonical_path, canonical_source_root)),
    };

    // Check for nested project.toml boundary between source root and path
    let mut check_dir = canonical_path.parent();
    while let Some(dir) = check_dir {
        if dir == canonical_source_root {
            break;
        }
        if dir.join("project.toml").is_file() {
            return Err(ModuleResolutionError::NestedProjectBoundary(dir.to_path_buf()));
        }
        check_dir = dir.parent();
    }

    let file_name = relative
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| ModuleResolutionError::InvalidModuleLayout(format!("invalid file path {}", path.display())))?;

    let (kind, components_iter, stem_opt) = if file_name == "package.ph" {
        (ModuleKind::Package, relative.parent(), None)
    } else if let Some(stem) = file_name.strip_suffix(".ph") {
        (ModuleKind::Module, relative.parent(), Some(stem))
    } else {
        return Err(ModuleResolutionError::InvalidModuleLayout(format!(
            "source file must end in .ph: {}",
            path.display()
        )));
    };

    let mut components = Vec::new();
    if let Some(parent) = components_iter {
        for comp in parent.iter() {
            let s = comp
                .to_str()
                .ok_or_else(|| ModuleResolutionError::InvalidModuleLayout(format!("non-utf8 path component in {}", path.display())))?;
            if !s.is_empty() {
                let comp_id = ModuleComponent::from_kebab(s).map_err(|e| ModuleResolutionError::InvalidModuleName(s.to_string(), e))?;
                if comp_id.as_str() != s && comp_id.to_kebab() != s {
                    return Err(ModuleResolutionError::NonCanonicalPhysicalName {
                        path: canonical_path.clone(),
                        expected: comp_id.to_kebab(),
                    });
                }
                components.push(comp_id);
            }
        }
    }

    if let Some(stem) = stem_opt {
        let comp_id = ModuleComponent::from_kebab(stem).map_err(|e| ModuleResolutionError::InvalidModuleName(stem.to_string(), e))?;
        if comp_id.as_str() != stem && comp_id.to_kebab() != stem {
            return Err(ModuleResolutionError::NonCanonicalPhysicalName {
                path: canonical_path.clone(),
                expected: format!("{}.ph", comp_id.to_kebab()),
            });
        }
        components.push(comp_id);
    }

    let module_path = ModulePath::from_components(components);
    let module_id = ModuleId::resolved(project.id, module_path);
    let source_id = SourceId(canonical_path.to_string_lossy().into());
    let source_location = SourceLocation {
        source_id,
        display_path: canonical_path,
    };

    Ok(SourceUnit {
        id: module_id,
        kind,
        source: source_location,
    })
}
