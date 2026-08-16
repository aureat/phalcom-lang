//! Source-provider authority for persistent projects, standalone packages,
//! standalone modules, inline units, and builtin projects.

use crate::error::{ModuleLoadError, ModuleResolutionError, SourceError};
use crate::identity::{
    BuiltinProject, ModuleComponent, ModuleId, ModulePath, ProjectIdentity, ResolvedProjectId, SourceId, SourceLocation, SyntheticProjectId,
};
use crate::project::{ProjectUniverse, ResolvedProject};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use unicode_normalization::UnicodeNormalization;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EntryOwnership {
    ProjectOwned { project: ResolvedProjectId },
    StandalonePackageOwned { package_root: PathBuf, synthetic: SyntheticProjectId },
    StandaloneModule { file: PathBuf, synthetic: SyntheticProjectId },
    Inline { synthetic: SyntheticProjectId },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ModuleKind { Module, Package }

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SourceUnit {
    pub id: ModuleId,
    pub kind: ModuleKind,
    pub source: SourceLocation,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ResolverGeneration(u64);
impl ResolverGeneration {
    pub const fn initial() -> Self { Self(0) }
    pub const fn next(self) -> Self { Self(self.0.wrapping_add(1)) }
}

pub trait SourceProvider {
    fn locate(&self, id: &ModuleId) -> Result<SourceUnit, ModuleLoadError>;
    fn read(&self, source: &SourceId) -> Result<Arc<str>, SourceError>;
    fn generation(&self) -> ResolverGeneration { ResolverGeneration::initial() }
}

#[derive(Debug)]
struct CachedUnit {
    generation: ResolverGeneration,
    value: Result<SourceUnit, ModuleResolutionError>,
}

#[derive(Debug)]
pub struct FilesystemSourceProvider {
    generation: Mutex<ResolverGeneration>,
    cache: Mutex<HashMap<(ProjectIdentity, ModulePath), CachedUnit>>,
    source_cache: Mutex<HashMap<SourceId, Arc<str>>>,
    module_to_source: Mutex<HashMap<ModuleId, SourceId>>,
    source_to_module: Mutex<HashMap<SourceId, ModuleId>>,
}

impl Default for FilesystemSourceProvider { fn default() -> Self { Self::new() } }
impl FilesystemSourceProvider {
    pub fn new() -> Self {
        Self {
            generation: Mutex::new(ResolverGeneration::initial()), cache: Mutex::new(HashMap::new()),
            source_cache: Mutex::new(HashMap::new()), module_to_source: Mutex::new(HashMap::new()), source_to_module: Mutex::new(HashMap::new()),
        }
    }

    pub fn generation(&self) -> ResolverGeneration { *self.generation.lock().unwrap() }

    pub fn bump_generation(&self) -> ResolverGeneration {
        let mut generation = self.generation.lock().unwrap();
        *generation = generation.next();
        self.cache.lock().unwrap().clear();
        self.source_cache.lock().unwrap().clear();
        self.module_to_source.lock().unwrap().clear();
        self.source_to_module.lock().unwrap().clear();
        *generation
    }

    pub fn clear_cache(&self) { self.bump_generation(); }

    pub fn locate(&self, project: &ResolvedProject, path: &ModulePath) -> Result<SourceUnit, ModuleResolutionError> {
        let id = ModuleId::resolved(project.id, path.clone());
        self.locate_project(project, &id)
    }

    fn locate_project(&self, project: &ResolvedProject, id: &ModuleId) -> Result<SourceUnit, ModuleResolutionError> {
        debug_assert_eq!(id.project, ProjectIdentity::Resolved(project.id));
        self.locate_under_root(id, &project.source_root, Some(&project.root_dir), true)
    }

    fn locate_standalone_package(&self, root: &Path, id: &ModuleId) -> Result<SourceUnit, ModuleResolutionError> {
        self.locate_under_root(id, root, None, true)
    }

    fn locate_under_root(
        &self, id: &ModuleId, canonical_root: &Path, project_root: Option<&Path>, root_is_package: bool,
    ) -> Result<SourceUnit, ModuleResolutionError> {
        let generation = self.generation();
        let key = (id.project, id.path.clone());
        if let Some(cached) = self.cache.lock().unwrap().get(&key) {
            if cached.generation == generation { return cached.value.clone(); }
        }
        let result = self.locate_under_root_uncached(id, canonical_root, project_root, root_is_package);
        self.cache.lock().unwrap().insert(key, CachedUnit { generation, value: result.clone() });
        result
    }

    fn locate_under_root_uncached(
        &self, id: &ModuleId, canonical_root: &Path, project_root: Option<&Path>, root_is_package: bool,
    ) -> Result<SourceUnit, ModuleResolutionError> {
        let root = canonical_root.canonicalize().map_err(|e| ModuleResolutionError::Source(SourceError::from_io(e, format!("canonicalize {}", canonical_root.display()))))?;
        let components = id.path.components();
        if components.is_empty() {
            if !root_is_package { return Err(ModuleResolutionError::PackageNotFoundError(id.to_string())); }
            return self.finish_candidate(id, ModuleKind::Package, &root, &root.join("package.ph"));
        }
        let mut current_dir = root.clone();
        for comp in &components[..components.len() - 1] {
            self.check_portability_in_dir(&current_dir)?;
            let dir = self.require_physical_directory(&current_dir, comp)?;
            if dir.join("project.toml").is_file() && project_root.map(|root| root != dir).unwrap_or(true) {
                return Err(ModuleResolutionError::NestedProjectBoundary(dir));
            }
            if !dir.join("package.ph").is_file() {
                return Err(ModuleResolutionError::PackageNotFoundError(format!("Directory '{}' is missing package.ph", dir.display())));
            }
            current_dir = dir;
        }
        self.check_portability_in_dir(&current_dir)?;
        let last = components.last().expect("non-empty path");
        let physical = last.to_kebab();
        let file = current_dir.join(format!("{physical}.ph"));
        let dir = current_dir.join(&physical);
        let package_file = dir.join("package.ph");
        self.reject_noncanonical_aliases(&current_dir, last, &file, &dir)?;
        let has_file = file.is_file();
        let has_package = package_file.is_file();
        if has_file && has_package {
            return Err(ModuleResolutionError::InvalidModuleLayout(format!(
                "logical component '{}' maps to both '{}' and '{}'", last, file.display(), package_file.display()
            )));
        }
        if has_file { return self.finish_candidate(id, ModuleKind::Module, &root, &file); }
        if has_package {
            if dir.join("project.toml").is_file() && project_root.map(|root| root != dir).unwrap_or(true) {
                return Err(ModuleResolutionError::NestedProjectBoundary(dir));
            }
            return self.finish_candidate(id, ModuleKind::Package, &root, &package_file);
        }
        Err(ModuleResolutionError::ModuleNotFound(id.to_string()))
    }

    fn require_physical_directory(&self, parent: &Path, comp: &ModuleComponent) -> Result<PathBuf, ModuleResolutionError> {
        let expected_name = comp.to_kebab();
        let expected = parent.join(&expected_name);
        self.reject_noncanonical_aliases(parent, comp, &parent.join(format!("{expected_name}.ph")), &expected)?;
        if expected.is_dir() { Ok(expected) } else {
            Err(ModuleResolutionError::PackageNotFoundError(format!("Component '{}' directory '{}' not found", comp, expected.display())))
        }
    }

    fn reject_noncanonical_aliases(
        &self, parent: &Path, logical: &ModuleComponent, expected_file: &Path, expected_dir: &Path,
    ) -> Result<(), ModuleResolutionError> {
        let snake = logical.as_str();
        if snake.contains('_') {
            let snake_file = parent.join(format!("{snake}.ph"));
            let snake_dir = parent.join(snake);
            if snake_file.exists() && snake_file != expected_file {
                return Err(ModuleResolutionError::NonCanonicalPhysicalName { logical: snake.to_string(), expected: logical.to_kebab(), found: snake_file });
            }
            if snake_dir.exists() && snake_dir != expected_dir {
                return Err(ModuleResolutionError::NonCanonicalPhysicalName { logical: snake.to_string(), expected: logical.to_kebab(), found: snake_dir });
            }
        }
        let expected = logical.to_kebab();
        if let Ok(entries) = std::fs::read_dir(parent) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                let stem = name.strip_suffix(".ph").unwrap_or(&name);
                if stem != expected && portable_key(stem) == portable_key(&expected) {
                    return Err(ModuleResolutionError::NonCanonicalPhysicalName {
                        logical: logical.as_str().to_string(), expected: expected.clone(), found: entry.path(),
                    });
                }
            }
        }
        Ok(())
    }

    fn check_portability_in_dir(&self, dir: &Path) -> Result<(), ModuleResolutionError> {
        let Ok(entries) = std::fs::read_dir(dir) else { return Ok(()) };
        let mut seen: HashMap<String, (String, PathBuf)> = HashMap::new();
        for entry in entries.flatten() {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();
            let stem = name.strip_suffix(".ph").unwrap_or(&name).to_string();
            let key = portable_key(&stem);
            if let Some((previous_name, previous_path)) = seen.get(&key) {
                if previous_path != &path {
                    let reason = if previous_name.to_lowercase() == stem.to_lowercase() { "case-fold collision" } else { "Unicode-normalization collision" };
                    return Err(ModuleResolutionError::PortabilityCollision { first: previous_path.clone(), second: path, reason: reason.to_string() });
                }
            } else {
                seen.insert(key, (stem, path));
            }
        }
        Ok(())
    }

    fn finish_candidate(&self, id: &ModuleId, kind: ModuleKind, canonical_root: &Path, candidate: &Path) -> Result<SourceUnit, ModuleResolutionError> {
        if !candidate.is_file() {
            return Err(match kind {
                ModuleKind::Package => ModuleResolutionError::PackageNotFoundError(candidate.display().to_string()),
                ModuleKind::Module => ModuleResolutionError::ModuleNotFound(candidate.display().to_string()),
            });
        }
        let canonical = candidate.canonicalize().map_err(|e| ModuleResolutionError::Source(SourceError::from_io(e, format!("canonicalize {}", candidate.display()))))?;
        if !canonical.starts_with(canonical_root) {
            return Err(ModuleResolutionError::ImportOutsideSourceRoot(canonical, canonical_root.to_path_buf()));
        }
        let source_id = SourceId(canonical.to_string_lossy().into());
        self.bind_source(id, &source_id)?;
        Ok(SourceUnit { id: id.clone(), kind, source: SourceLocation { source_id, display_path: canonical } })
    }

    fn bind_source(&self, module: &ModuleId, source: &SourceId) -> Result<(), ModuleResolutionError> {
        let mut module_to_source = self.module_to_source.lock().unwrap();
        let mut source_to_module = self.source_to_module.lock().unwrap();
        if let Some(existing_source) = module_to_source.get(module) {
            if existing_source != source {
                return Err(ModuleResolutionError::DuplicateSourceIdentity(format!("module {module} maps to both {existing_source} and {source}")));
            }
        }
        if let Some(existing_module) = source_to_module.get(source) {
            if existing_module != module {
                return Err(ModuleResolutionError::DuplicateSourceIdentity(format!("source {source} maps to both {existing_module} and {module}")));
            }
        }
        module_to_source.entry(module.clone()).or_insert_with(|| source.clone());
        source_to_module.entry(source.clone()).or_insert_with(|| module.clone());
        Ok(())
    }

    fn read_file(&self, source: &SourceId) -> Result<Arc<str>, SourceError> {
        if let Some(content) = self.source_cache.lock().unwrap().get(source) { return Ok(content.clone()); }
        let path = Path::new(&*source.0);
        let content = std::fs::read_to_string(path).map_err(|e| SourceError::from_io(e, format!("read {}", path.display())))?;
        let content: Arc<str> = Arc::from(content);
        self.source_cache.lock().unwrap().insert(source.clone(), content.clone());
        Ok(content)
    }
}

fn portable_key(name: &str) -> String { name.nfc().flat_map(char::to_lowercase).collect() }

pub struct ProjectSourceProvider<'u> { universe: &'u ProjectUniverse, fs: FilesystemSourceProvider }
impl<'u> ProjectSourceProvider<'u> {
    pub fn new(universe: &'u ProjectUniverse) -> Self { Self { universe, fs: FilesystemSourceProvider::new() } }
}
impl SourceProvider for ProjectSourceProvider<'_> {
    fn locate(&self, id: &ModuleId) -> Result<SourceUnit, ModuleLoadError> {
        let ProjectIdentity::Resolved(project_id) = id.project else {
            return Err(ModuleResolutionError::ModuleNotFound(format!("{id} is not owned by a resolved Project")).into());
        };
        let project = self.universe.get_project(project_id).ok_or_else(|| ModuleResolutionError::ModuleNotFound(format!("Project {project_id} not found")))?;
        self.fs.locate_project(project, id).map_err(Into::into)
    }
    fn read(&self, source: &SourceId) -> Result<Arc<str>, SourceError> { self.fs.read_file(source) }
    fn generation(&self) -> ResolverGeneration { self.fs.generation() }
}

pub struct StandalonePackageSourceProvider { synthetic: SyntheticProjectId, package_root: PathBuf, fs: FilesystemSourceProvider }
impl StandalonePackageSourceProvider {
    pub fn new(synthetic: SyntheticProjectId, package_root: impl AsRef<Path>) -> Result<Self, SourceError> {
        let package_root = package_root.as_ref().canonicalize().map_err(|e| SourceError::from_io(e, format!("canonicalize {}", package_root.as_ref().display())))?;
        if !package_root.join("package.ph").is_file() {
            return Err(SourceError::NotFound(format!("{} is not a Package: missing package.ph", package_root.display())));
        }
        Ok(Self { synthetic, package_root, fs: FilesystemSourceProvider::new() })
    }
}
impl SourceProvider for StandalonePackageSourceProvider {
    fn locate(&self, id: &ModuleId) -> Result<SourceUnit, ModuleLoadError> {
        if id.project != ProjectIdentity::Synthetic(self.synthetic) { return Err(ModuleResolutionError::ModuleNotFound(id.to_string()).into()); }
        self.fs.locate_standalone_package(&self.package_root, id).map_err(Into::into)
    }
    fn read(&self, source: &SourceId) -> Result<Arc<str>, SourceError> { self.fs.read_file(source) }
    fn generation(&self) -> ResolverGeneration { self.fs.generation() }
}

pub struct StandaloneModuleSourceProvider { entry: PathBuf, entry_id: ModuleId, source_id: SourceId }
impl StandaloneModuleSourceProvider {
    pub fn new(synthetic: SyntheticProjectId, entry: impl AsRef<Path>) -> Result<Self, ModuleLoadError> {
        let entry = entry.as_ref().canonicalize().map_err(|e| ModuleLoadError::Io {
            module: None, error: SourceError::from_io(e, format!("canonicalize {}", entry.as_ref().display())),
        })?;
        let stem = entry.file_stem().and_then(|s| s.to_str()).ok_or_else(|| ModuleResolutionError::InvalidModuleLayout(format!("invalid module filename {}", entry.display())))?;
        let component = ModuleComponent::from_kebab(stem).map_err(|e| ModuleResolutionError::InvalidModuleName(stem.to_string(), e))?;
        let entry_id = ModuleId::synthetic_in(synthetic, ModulePath::from_components(vec![component]));
        let source_id = SourceId(entry.to_string_lossy().into());
        Ok(Self { entry, entry_id, source_id })
    }
    pub fn entry_id(&self) -> &ModuleId { &self.entry_id }
}
impl SourceProvider for StandaloneModuleSourceProvider {
    fn locate(&self, id: &ModuleId) -> Result<SourceUnit, ModuleLoadError> {
        if id != &self.entry_id {
            return Err(ModuleResolutionError::StandaloneSiblingImport { entry: self.entry.clone(), requested: id.to_string() }.into());
        }
        Ok(SourceUnit { id: id.clone(), kind: ModuleKind::Module, source: SourceLocation { source_id: self.source_id.clone(), display_path: self.entry.clone() } })
    }
    fn read(&self, source: &SourceId) -> Result<Arc<str>, SourceError> {
        if source != &self.source_id { return Err(SourceError::NotFound(source.to_string())); }
        Ok(Arc::from(std::fs::read_to_string(&self.entry).map_err(|e| SourceError::from_io(e, format!("read {}", self.entry.display())))?))
    }
}

pub struct InlineSourceProvider { entry_id: ModuleId, source_id: SourceId, text: Arc<str> }
impl InlineSourceProvider {
    pub fn new(synthetic: SyntheticProjectId, text: Arc<str>) -> Self {
        let entry_id = ModuleId::synthetic_in(
            synthetic, ModulePath::from_components(vec![ModuleComponent::from_identifier("inline").expect("canonical")]),
        );
        Self { entry_id, source_id: SourceId(format!("inline:{synthetic}").into()), text }
    }
    pub fn entry_id(&self) -> &ModuleId { &self.entry_id }
}
impl SourceProvider for InlineSourceProvider {
    fn locate(&self, id: &ModuleId) -> Result<SourceUnit, ModuleLoadError> {
        if id != &self.entry_id { return Err(ModuleResolutionError::ModuleNotFound(id.to_string()).into()); }
        Ok(SourceUnit { id: id.clone(), kind: ModuleKind::Module, source: SourceLocation { source_id: self.source_id.clone(), display_path: PathBuf::from("<inline>") } })
    }
    fn read(&self, source: &SourceId) -> Result<Arc<str>, SourceError> {
        if source == &self.source_id { Ok(self.text.clone()) } else { Err(SourceError::NotFound(source.to_string())) }
    }
}

pub struct BuiltinProjectSourceProvider { builtin: BuiltinProject }
impl BuiltinProjectSourceProvider {
    pub const fn new(builtin: BuiltinProject) -> Self { Self { builtin } }

    pub fn virtual_uri(id: &ModuleId) -> Option<String> {
        let ProjectIdentity::Builtin(project) = id.project else { return None };
        let mut uri = format!("phalcom://{}/", project.root_name());
        if !id.path.is_root() { uri.push_str(&id.path.components().iter().map(ModuleComponent::as_str).collect::<Vec<_>>().join("/")); }
        Some(uri)
    }

    fn catalog(&self, path: &ModulePath) -> Option<(ModuleKind, &'static str)> {
        let parts = path.components().iter().map(ModuleComponent::as_str).collect::<Vec<_>>();
        match (self.builtin, parts.as_slice()) {
            (BuiltinProject::Universe, []) => Some((ModuleKind::Package,
                "expose .reflection\nlet Object = None\nlet Class = None\nlet Int = None\nlet Float = None\nlet String = None\nlet Bool = None\nlet Symbol = None\nlet Option = None\nlet Some = None\nlet List = None\nlet Map = None\nlet Set = None\nlet Tuple = None\nlet Record = None\nlet Range = None\nlet Bytes = None\nlet Function = None\nlet Module = None\nlet Package = None\nlet Error = None\nlet Fiber = None\nexport Object, Class, Int, Float, String, Bool, Symbol, Option, Some, List, Map, Set, Tuple, Record, Range, Bytes, Function, Module, Package, Error, Fiber\n")),
            (BuiltinProject::Universe, ["reflection"]) => Some((ModuleKind::Package, "expose .selector\nlet Selector = None\nexport Selector\n")),
            (BuiltinProject::Universe, ["reflection", "selector"]) => Some((ModuleKind::Module, "let Selector = None\nexport Selector\n")),
            (BuiltinProject::Std, []) => Some((ModuleKind::Package, "expose .json\n")),
            (BuiltinProject::Std, ["json"]) => Some((ModuleKind::Package, "let available = false\nexport available\n")),
            _ => None,
        }
    }
}
impl SourceProvider for BuiltinProjectSourceProvider {
    fn locate(&self, id: &ModuleId) -> Result<SourceUnit, ModuleLoadError> {
        if id.project != ProjectIdentity::Builtin(self.builtin) { return Err(ModuleResolutionError::ModuleNotFound(id.to_string()).into()); }
        let Some((kind, _)) = self.catalog(&id.path) else { return Err(ModuleResolutionError::ModuleNotFound(id.to_string()).into()); };
        let uri = Self::virtual_uri(id).expect("builtin id has virtual URI");
        Ok(SourceUnit {
            id: id.clone(), kind,
            source: SourceLocation { source_id: SourceId(format!("builtin:{}:{}", self.builtin.root_name(), id.path).into()), display_path: PathBuf::from(uri) },
        })
    }
    fn read(&self, source: &SourceId) -> Result<Arc<str>, SourceError> {
        let prefix = format!("builtin:{}:", self.builtin.root_name());
        let Some(path_text) = source.0.strip_prefix(&prefix) else { return Err(SourceError::NotFound(source.to_string())); };
        let path = if path_text.is_empty() { ModulePath::root() } else {
            let mut components = Vec::new();
            for part in path_text.split('.') { components.push(ModuleComponent::from_identifier(part).map_err(|e| SourceError::NotFound(e.to_string()))?); }
            ModulePath::from_components(components)
        };
        self.catalog(&path).map(|(_, text)| Arc::<str>::from(text)).ok_or_else(|| SourceError::NotFound(source.to_string()))
    }
}

pub enum UserSourceProvider<'u> {
    Project(ProjectSourceProvider<'u>), StandalonePackage(StandalonePackageSourceProvider), StandaloneModule(StandaloneModuleSourceProvider), Inline(InlineSourceProvider),
}

pub struct SessionSourceProvider<'u> {
    user: UserSourceProvider<'u>, universe: BuiltinProjectSourceProvider, std: BuiltinProjectSourceProvider,
}
impl<'u> SessionSourceProvider<'u> {
    pub fn project(universe: &'u ProjectUniverse) -> Self {
        Self { user: UserSourceProvider::Project(ProjectSourceProvider::new(universe)), universe: BuiltinProjectSourceProvider::new(BuiltinProject::Universe), std: BuiltinProjectSourceProvider::new(BuiltinProject::Std) }
    }
    pub fn standalone_package(synthetic: SyntheticProjectId, root: impl AsRef<Path>) -> Result<Self, SourceError> {
        Ok(Self { user: UserSourceProvider::StandalonePackage(StandalonePackageSourceProvider::new(synthetic, root)?), universe: BuiltinProjectSourceProvider::new(BuiltinProject::Universe), std: BuiltinProjectSourceProvider::new(BuiltinProject::Std) })
    }
    pub fn standalone_module(synthetic: SyntheticProjectId, entry: impl AsRef<Path>) -> Result<Self, ModuleLoadError> {
        Ok(Self { user: UserSourceProvider::StandaloneModule(StandaloneModuleSourceProvider::new(synthetic, entry)?), universe: BuiltinProjectSourceProvider::new(BuiltinProject::Universe), std: BuiltinProjectSourceProvider::new(BuiltinProject::Std) })
    }
    pub fn inline(synthetic: SyntheticProjectId, text: Arc<str>) -> Self {
        Self { user: UserSourceProvider::Inline(InlineSourceProvider::new(synthetic, text)), universe: BuiltinProjectSourceProvider::new(BuiltinProject::Universe), std: BuiltinProjectSourceProvider::new(BuiltinProject::Std) }
    }
    pub fn entry_id(&self) -> Option<&ModuleId> {
        match &self.user { UserSourceProvider::StandaloneModule(provider) => Some(provider.entry_id()), UserSourceProvider::Inline(provider) => Some(provider.entry_id()), _ => None }
    }
}
impl SourceProvider for SessionSourceProvider<'_> {
    fn locate(&self, id: &ModuleId) -> Result<SourceUnit, ModuleLoadError> {
        match id.project {
            ProjectIdentity::Builtin(BuiltinProject::Universe) => self.universe.locate(id),
            ProjectIdentity::Builtin(BuiltinProject::Std) => self.std.locate(id),
            _ => match &self.user {
                UserSourceProvider::Project(provider) => provider.locate(id),
                UserSourceProvider::StandalonePackage(provider) => provider.locate(id),
                UserSourceProvider::StandaloneModule(provider) => provider.locate(id),
                UserSourceProvider::Inline(provider) => provider.locate(id),
            },
        }
    }
    fn read(&self, source: &SourceId) -> Result<Arc<str>, SourceError> {
        if source.0.starts_with("builtin:universe:") { return self.universe.read(source); }
        if source.0.starts_with("builtin:std:") { return self.std.read(source); }
        match &self.user {
            UserSourceProvider::Project(provider) => provider.read(source),
            UserSourceProvider::StandalonePackage(provider) => provider.read(source),
            UserSourceProvider::StandaloneModule(provider) => provider.read(source),
            UserSourceProvider::Inline(provider) => provider.read(source),
        }
    }
    fn generation(&self) -> ResolverGeneration {
        match &self.user {
            UserSourceProvider::Project(provider) => provider.generation(),
            UserSourceProvider::StandalonePackage(provider) => provider.generation(),
            UserSourceProvider::StandaloneModule(provider) => provider.generation(),
            UserSourceProvider::Inline(provider) => provider.generation(),
        }
    }
}
