//! Program-level compiler seam for closed linked module plans.

use super::artifact::ModuleArtifact;
use super::registry::{ModulePlanFingerprint, RuntimeProgramId};
use phalcom_modules::{
    LinkError, LinkedModule, LinkedProgram, ModuleComponent, ModuleId, ModuleKind, ModuleLinker, ModulePath, ModuleResolutionError, ModuleResolver,
    ProjectError, ProjectUniverse, SessionSourceProvider, SourceError, SourceLocation, SourceProvider, discover_owning_project,
    discover_standalone_package_root,
};
use std::collections::{BTreeMap, HashSet};
use std::path::{Component, Path, PathBuf};
use std::sync::Arc;
use thiserror::Error;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EntrySelection {
    ModuleId(ModuleId),
    Project(PathBuf),
    Package(PathBuf),
    Module(PathBuf),
    Inline(Arc<str>),
}

#[derive(Clone, Debug, PartialEq)]
pub struct CompiledModule {
    pub id: ModuleId,
    pub kind: ModuleKind,
    pub source: Option<SourceLocation>,
    pub source_text: Option<Arc<str>>,
    pub interface: Arc<phalcom_modules::LinkedModuleInterface>,
    pub artifact: ModuleArtifact,
    pub linked_reads: Vec<phalcom_modules::LinkedReadSpec>,
    pub plan_fingerprint: ModulePlanFingerprint,
}

#[derive(Clone, Debug)]
pub struct CompiledProgram {
    pub runtime_id: RuntimeProgramId,
    pub project_universe: Arc<ProjectUniverse>,
    pub linked: Arc<LinkedProgram>,
    pub modules: BTreeMap<ModuleId, CompiledModule>,
    pub entry: ModuleId,
    pub initialization_order: Vec<ModuleId>,
}

#[derive(Debug, Error, Clone)]
pub enum ProgramCompileError {
    #[error(transparent)]
    Link(#[from] LinkError),
    #[error("entry module {0} is not present in linked program")]
    MissingEntry(ModuleId),
    #[error("project manifest at '{0}' is not executable (no entry declared)")]
    ProjectNotExecutable(String),
    #[error("package at '{0}' is not executable (requires package.ph and main.ph)")]
    PackageNotExecutable(String),
    #[error("project error: {0}")]
    Project(#[from] ProjectError),
    #[error("resolution error: {0}")]
    Resolution(#[from] ModuleResolutionError),
    #[error("module load error: {0}")]
    ModuleLoad(#[from] phalcom_modules::ModuleLoadError),
    #[error("source error: {0}")]
    Source(#[from] SourceError),
    #[error("io error: {0}")]
    Io(String),
}

pub struct ProgramCompiler {
    linked: Option<Arc<LinkedProgram>>,
}

impl ProgramCompiler {
    pub fn new(linked: Arc<LinkedProgram>) -> Self { Self { linked: Some(linked) } }

    pub fn compile_entry_selection(entry: EntrySelection) -> Result<CompiledProgram, ProgramCompileError> {
        match entry {
            EntrySelection::ModuleId(entry_id) => Err(ProgramCompileError::Io(format!(
                "EntrySelection::ModuleId({entry_id}) requires an existing linked universe; use ProgramCompiler::new(linked).compile_entry(...)"
            ))),
            EntrySelection::Project(root_dir) => Self::compile_project(root_dir),
            EntrySelection::Package(package_dir) => Self::compile_standalone_package(package_dir),
            EntrySelection::Module(file_path) => Self::compile_file(file_path),
            EntrySelection::Inline(source_text) => Self::compile_inline(source_text),
        }
    }

    fn compile_project(root_dir: PathBuf) -> Result<CompiledProgram, ProgramCompileError> {
        let manifest_path = if root_dir.ends_with("project.toml") { root_dir } else { root_dir.join("project.toml") };
        if !manifest_path.is_file() { return Err(ProgramCompileError::Io(format!("project manifest not found at {}", manifest_path.display()))); }
        let mut universe = ProjectUniverse::new();
        let root_id = universe.load_root(&manifest_path)?;
        let project = universe.get_project(root_id).ok_or_else(|| ProgramCompileError::Io(format!("project {root_id} not found after resolution")))?;
        let entry_path = project.entry.clone().ok_or_else(|| ProgramCompileError::ProjectNotExecutable(manifest_path.display().to_string()))?;
        let entry_id = ModuleId::resolved(root_id, entry_path);
        let universe = Arc::new(universe);
        let provider = SessionSourceProvider::project(universe.as_ref());
        Self::discover_and_link(universe.clone(), &provider, entry_id)
    }

    fn compile_standalone_package(package_dir: PathBuf) -> Result<CompiledProgram, ProgramCompileError> {
        let package_file = package_dir.join("package.ph");
        let main_file = package_dir.join("main.ph");
        if !package_file.is_file() || !main_file.is_file() {
            return Err(ProgramCompileError::PackageNotExecutable(package_dir.display().to_string()));
        }
        let mut universe = ProjectUniverse::new();
        let synthetic = universe.allocate_synthetic_id();
        let entry_id = ModuleId::synthetic_in(
            synthetic,
            ModulePath::from_components(vec![ModuleComponent::from_identifier("main").expect("canonical entry component")]),
        );
        let universe = Arc::new(universe);
        let provider = SessionSourceProvider::standalone_package(synthetic, &package_dir)?;
        Self::discover_and_link(universe, &provider, entry_id)
    }

    fn compile_file(file_path: PathBuf) -> Result<CompiledProgram, ProgramCompileError> {
        if !file_path.is_file() { return Err(ProgramCompileError::Io(format!("module file not found at {}", file_path.display()))); }
        let canonical_file = file_path.canonicalize().map_err(|e| ProgramCompileError::Io(format!("{}: {e}", file_path.display())))?;
        if let Some(project_root) = discover_owning_project(&canonical_file)? {
            let mut universe = ProjectUniverse::new();
            let root_id = universe.load_root(project_root.join("project.toml"))?;
            let project = universe.get_project(root_id).ok_or_else(|| ProgramCompileError::Io(format!("project {root_id} not found after resolution")))?;
            let rel_path = canonical_file.strip_prefix(&project.source_root).map_err(|_| ProgramCompileError::Io(format!(
                "file {} is not under project source root {}", canonical_file.display(), project.source_root.display()
            )))?;
            let entry_id = ModuleId::resolved(root_id, relative_path_to_module_path(rel_path)?);
            let universe = Arc::new(universe);
            let provider = SessionSourceProvider::project(universe.as_ref());
            return Self::discover_and_link(universe.clone(), &provider, entry_id);
        }
        if let Some(package_root) = discover_standalone_package_root(&canonical_file)? {
            let rel_path = canonical_file.strip_prefix(&package_root).map_err(|_| ProgramCompileError::Io(format!(
                "file {} is not under package root {}", canonical_file.display(), package_root.display()
            )))?;
            let mut universe = ProjectUniverse::new();
            let synthetic = universe.allocate_synthetic_id();
            let entry_id = ModuleId::synthetic_in(synthetic, relative_path_to_module_path(rel_path)?);
            let universe = Arc::new(universe);
            let provider = SessionSourceProvider::standalone_package(synthetic, &package_root)?;
            return Self::discover_and_link(universe, &provider, entry_id);
        }
        let mut universe = ProjectUniverse::new();
        let synthetic = universe.allocate_synthetic_id();
        let provider = SessionSourceProvider::standalone_module(synthetic, &canonical_file)?;
        let entry_id = provider.entry_id().cloned().ok_or_else(|| ProgramCompileError::Io("standalone module provider has no entry identity".to_string()))?;
        Self::discover_and_link(Arc::new(universe), &provider, entry_id)
    }

    fn compile_inline(source_text: Arc<str>) -> Result<CompiledProgram, ProgramCompileError> {
        let mut universe = ProjectUniverse::new();
        let synthetic = universe.allocate_synthetic_id();
        let provider = SessionSourceProvider::inline(synthetic, source_text);
        let entry_id = provider.entry_id().cloned().ok_or_else(|| ProgramCompileError::Io("inline provider has no entry identity".to_string()))?;
        Self::discover_and_link(Arc::new(universe), &provider, entry_id)
    }

    fn discover_and_link<P: SourceProvider + ?Sized>(
        universe: Arc<ProjectUniverse>, source_provider: &P, entry: ModuleId,
    ) -> Result<CompiledProgram, ProgramCompileError> {
        let mut resolver = ModuleResolver::new(universe.as_ref(), source_provider);
        let mut interfaces = BTreeMap::new();
        let mut resolved = BTreeMap::new();
        let mut visited = HashSet::from([entry.clone()]);
        let mut pending = vec![entry.clone()];
        let mut source_locations = BTreeMap::new();
        let mut source_texts = BTreeMap::new();
        while let Some(current_id) = pending.pop() {
            let unit = source_provider.locate(&current_id)?;
            let text = source_provider.read(&unit.source.source_id)?;
            source_locations.insert(current_id.clone(), unit.source.clone());
            source_texts.insert(current_id.clone(), text);
            let interface = resolver.load_interface(&current_id)?;
            for import_surface in &interface.imports {
                let import_path = match import_surface {
                    phalcom_modules::ImportSurface::Module(m) => &m.path,
                    phalcom_modules::ImportSurface::Selective(s) => &s.path,
                    phalcom_modules::ImportSurface::ReExport(r) => &r.path,
                };
                let target_unit = resolver.resolve_import(&current_id, import_path)?;
                resolved.insert((current_id.clone(), import_path.to_string()), target_unit.id.clone());
                if visited.insert(target_unit.id.clone()) { pending.push(target_unit.id); }
            }
            interfaces.insert(current_id, interface);
        }
        let linker = ModuleLinker::new(universe.clone(), interfaces);
        let linked = Arc::new(linker.link(entry.clone(), &resolved)?);
        let modules = linked.modules.iter().map(|(id, module)| {
            let source = source_locations.get(id).cloned();
            let source_text = source_texts.get(id).cloned();
            (id.clone(), compile_module(id.clone(), module, source, source_text))
        }).collect();
        Ok(CompiledProgram {
            runtime_id: RuntimeProgramId::fresh(), project_universe: universe, linked: linked.clone(), modules, entry,
            initialization_order: linked.initialization_order.clone(),
        })
    }

    pub fn compile_entry(&self, entry: EntrySelection) -> Result<CompiledProgram, ProgramCompileError> {
        if let Some(linked) = &self.linked {
            let entry_id = match entry { EntrySelection::ModuleId(id) => id, other => return Self::compile_entry_selection(other) };
            if !linked.modules.contains_key(&entry_id) { return Err(ProgramCompileError::MissingEntry(entry_id)); }
            let modules = linked.modules.iter().map(|(id, module)| (id.clone(), compile_module(id.clone(), module, None, None))).collect();
            return Ok(CompiledProgram {
                runtime_id: RuntimeProgramId::fresh(), project_universe: linked.universe.clone(), linked: linked.clone(), modules, entry: entry_id,
                initialization_order: linked.initialization_order.clone(),
            });
        }
        Self::compile_entry_selection(entry)
    }
}

fn relative_path_to_module_path(rel_path: &Path) -> Result<ModulePath, ProgramCompileError> {
    let mut components = Vec::new();
    if let Some(parent) = rel_path.parent() {
        for component in parent.components() {
            if let Component::Normal(name) = component {
                let name = name.to_str().ok_or_else(|| ProgramCompileError::Io(format!("non-UTF-8 module path {}", rel_path.display())))?;
                if !name.is_empty() { components.push(ModuleComponent::from_kebab(name).map_err(|e| ProgramCompileError::Io(e.to_string()))?); }
            }
        }
    }
    let file_stem = rel_path.file_stem().and_then(|name| name.to_str()).ok_or_else(|| ProgramCompileError::Io(format!("invalid module filename {}", rel_path.display())))?;
    if file_stem != "package" { components.push(ModuleComponent::from_kebab(file_stem).map_err(|e| ProgramCompileError::Io(e.to_string()))?); }
    Ok(ModulePath::from_components(components))
}

fn compile_module(id: ModuleId, module: &LinkedModule, source: Option<SourceLocation>, source_text: Option<Arc<str>>) -> CompiledModule {
    let fingerprint_material = format!("{id:?}\n{module:?}\n{source:?}\n{source_text:?}");
    CompiledModule {
        id, kind: module.interface.kind, source, source_text, interface: Arc::new(module.interface.clone()), artifact: ModuleArtifact::empty(module),
        linked_reads: module.linked_reads.clone(), plan_fingerprint: ModulePlanFingerprint::from_bytes(fingerprint_material.as_bytes()),
    }
}
