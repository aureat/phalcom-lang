//! Persistent compiler-owned module workspace lifecycle.

use crate::error::{InterfaceError, ModuleLoadError, ModuleResolutionError, ProjectError, SourceError};
use crate::identity::{ModuleComponent, ModuleId, ModulePath, ProjectSourceIdentity, SourceId, SourceLocation, SyntheticProjectId, SyntheticProjectIdAllocator};
use crate::interface::{ImportSurface, InterfaceBuilder};
use crate::linker::{LinkError, LinkedProgram, ModuleLinker};
use crate::manifest::DependencyProvider;
use crate::project::{ProjectUniverse, discover_owning_project};
use crate::resolver::ModuleResolver;
use crate::source::{FilesystemSourceProvider, ModuleKind, OverlaySourceProvider, ParsedModuleUnit, SourceOverlay, SourceProvider};
use phalcom_ast::ast::{ImportPath, ImportRoot};
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::path::PathBuf;
use std::sync::Arc;
use thiserror::Error;

/// Source-lifecycle revision supplied by a workspace client.
#[derive(Clone, Copy, Debug, Default, Eq, Ord, PartialEq, PartialOrd)]
pub struct SourceRevision(pub u64);

/// Canonical source state retained by one persistent module session.
#[derive(Clone, Debug)]
pub struct WorkspaceSourceState {
    pub module: ModuleId,
    pub kind: ModuleKind,
    pub location: SourceLocation,
    pub revision: SourceRevision,
    pub text: Arc<str>,
    pub parsed: Arc<ParsedModuleUnit>,
    pub open_overlay: bool,
}

/// Source mutation at module-infrastructure boundary. It contains no LSP types.
#[derive(Clone, Debug)]
pub enum WorkspaceSourceMutation {
    SetOverlay {
        source: SourceLocation,
        text: Arc<str>,
        revision: SourceRevision,
    },
    RemoveOverlay {
        source: SourceId,
    },
    RefreshDisk {
        source: SourceLocation,
        revision: SourceRevision,
    },
    RemoveSource {
        source: SourceId,
    },
}

/// Products published after one source mutation.
#[derive(Clone, Debug)]
pub struct WorkspaceModuleUpdate {
    pub linked: Arc<LinkedProgram>,
    pub sources: BTreeMap<ModuleId, Arc<ParsedModuleUnit>>,
    pub changed_modules: BTreeSet<ModuleId>,
    pub removed_modules: BTreeSet<ModuleId>,
    pub identity_changes: BTreeSet<ModuleId>,
}

#[derive(Debug, Error)]
pub enum WorkspaceModuleSessionError {
    #[error(transparent)]
    Project(#[from] ProjectError),
    #[error(transparent)]
    Module(#[from] ModuleLoadError),
    #[error(transparent)]
    Resolution(#[from] ModuleResolutionError),
    #[error(transparent)]
    Interface(#[from] InterfaceError),
    #[error(transparent)]
    Link(#[from] LinkError),
    #[error(transparent)]
    Source(#[from] SourceError),
    #[error("source {source_id} has invalid syntax: {error}")]
    Parse {
        source_id: SourceId,
        error: phalcom_ast::error::SyntaxError,
    },
    #[error("source {0} is not registered in workspace session")]
    UnknownSource(SourceId),
    #[error("source path is not a supported Phalcom module: {0}")]
    InvalidSourcePath(PathBuf),
}

/// Persistent owner of project identity, source overlays, module identity, and linking.
#[derive(Debug)]
pub struct WorkspaceModuleSession {
    universe: ProjectUniverse,
    provider: OverlaySourceProvider<FilesystemSourceProvider>,
    project_roots: BTreeMap<ProjectSourceIdentity, crate::identity::ResolvedProjectId>,
    modules_by_source: BTreeMap<SourceId, ModuleId>,
    sources_by_module: BTreeMap<ModuleId, WorkspaceSourceState>,
    standalone_projects: BTreeMap<SourceId, SyntheticProjectId>,
    synthetic_ids: SyntheticProjectIdAllocator,
    linked: Option<Arc<LinkedProgram>>,
    generation: u64,
}

impl Default for WorkspaceModuleSession {
    fn default() -> Self {
        Self::new()
    }
}

impl WorkspaceModuleSession {
    pub fn new() -> Self {
        Self {
            universe: ProjectUniverse::new(),
            provider: OverlaySourceProvider::new(FilesystemSourceProvider::new()),
            project_roots: BTreeMap::new(),
            modules_by_source: BTreeMap::new(),
            sources_by_module: BTreeMap::new(),
            standalone_projects: BTreeMap::new(),
            synthetic_ids: SyntheticProjectIdAllocator,
            linked: None,
            generation: 0,
        }
    }

    pub fn universe(&self) -> &ProjectUniverse {
        &self.universe
    }

    pub fn provider(&self) -> &OverlaySourceProvider<FilesystemSourceProvider> {
        &self.provider
    }

    pub fn generation(&self) -> u64 {
        self.generation
    }

    pub fn linked(&self) -> Option<&Arc<LinkedProgram>> {
        self.linked.as_ref()
    }

    pub fn source(&self, id: &ModuleId) -> Option<&WorkspaceSourceState> {
        self.sources_by_module.get(id)
    }

    pub fn module_for_source(&self, source: &SourceId) -> Option<&ModuleId> {
        self.modules_by_source.get(source)
    }

    pub fn sources(&self) -> &BTreeMap<ModuleId, WorkspaceSourceState> {
        &self.sources_by_module
    }

    pub fn set_workspace_roots(
        &mut self,
        roots: &[PathBuf],
        dependency_provider: &dyn DependencyProvider,
    ) -> Result<WorkspaceModuleUpdate, WorkspaceModuleSessionError> {
        let previous_states = std::mem::take(&mut self.sources_by_module);
        let removed_modules = previous_states.keys().cloned().collect::<BTreeSet<_>>();
        self.modules_by_source.clear();
        self.provider.clear_overlays();
        self.universe = ProjectUniverse::new();
        self.project_roots.clear();
        for root in roots {
            let canonical = root.canonicalize().unwrap_or_else(|_| root.clone());
            let id = self.universe.load_root_with_provider(canonical.join("project.toml"), dependency_provider)?;
            self.project_roots.insert(ProjectSourceIdentity::from_path(canonical), id);
        }
        self.provider.base().clear_cache();
        let mut changed = BTreeSet::new();
        for state in previous_states.into_values() {
            let module = self.module_for_location(&state.location)?;
            let parsed = parse_source(module.clone(), state.kind, state.location.clone(), state.text.clone())?;
            if state.open_overlay {
                self.provider
                    .set_overlay(SourceOverlay::new(module.clone(), state.kind, state.location.clone(), parsed.text.clone()));
            }
            self.insert_state(module.clone(), state.kind, state.location, state.revision, parsed, state.open_overlay);
            changed.insert(module);
        }
        self.generation = self.generation.saturating_add(1);
        self.rebuild(changed, removed_modules.clone(), removed_modules)
    }

    pub fn apply(&mut self, mutation: WorkspaceSourceMutation) -> Result<WorkspaceModuleUpdate, WorkspaceModuleSessionError> {
        let before: BTreeSet<_> = self.sources_by_module.keys().cloned().collect();
        let mut changed = BTreeSet::new();
        let mut identity_changes = BTreeSet::new();

        match mutation {
            WorkspaceSourceMutation::SetOverlay { source, text, revision } => {
                let module = self.module_for_location(&source)?;
                let kind = self.kind_for_source(&module, &source);
                let parsed = parse_source(module.clone(), kind, source.clone(), text)?;
                self.provider
                    .set_overlay(SourceOverlay::new(module.clone(), kind, source.clone(), parsed.text.clone()));
                self.insert_state(module.clone(), kind, source, revision, parsed, true);
                changed.insert(module);
            }
            WorkspaceSourceMutation::RemoveOverlay { source } => {
                let module = self
                    .modules_by_source
                    .get(&source)
                    .cloned()
                    .ok_or_else(|| WorkspaceModuleSessionError::UnknownSource(source.clone()))?;
                self.provider.remove_overlay(&module);
                let state = self
                    .sources_by_module
                    .get(&module)
                    .cloned()
                    .ok_or_else(|| WorkspaceModuleSessionError::UnknownSource(source.clone()))?;
                let text = self.provider.base().read(&source)?;
                let parsed = parse_source(module.clone(), state.kind, state.location.clone(), text)?;
                self.insert_state(module.clone(), state.kind, state.location, state.revision, parsed, false);
                changed.insert(module);
            }
            WorkspaceSourceMutation::RefreshDisk { source, revision } => {
                let module = self.module_for_location(&source)?;
                if let Some(existing) = self.sources_by_module.get(&module) {
                    if existing.open_overlay {
                        self.provider.remove_overlay(&module);
                    }
                }
                self.provider.base().clear_cache();
                let text = self.provider.base().read(&source.source_id)?;
                let kind = self.kind_for_source(&module, &source);
                let parsed = parse_source(module.clone(), kind, source.clone(), text)?;
                self.insert_state(module.clone(), kind, source, revision, parsed, false);
                changed.insert(module);
            }
            WorkspaceSourceMutation::RemoveSource { source } => {
                let module = self
                    .modules_by_source
                    .remove(&source)
                    .ok_or_else(|| WorkspaceModuleSessionError::UnknownSource(source.clone()))?;
                self.provider.remove_overlay(&module);
                self.provider.base().clear_cache();
                self.sources_by_module.remove(&module);
                identity_changes.insert(module);
            }
        }

        self.generation = self.generation.saturating_add(1);
        let after: BTreeSet<_> = self.sources_by_module.keys().cloned().collect();
        let removed_modules = before.difference(&after).cloned().collect();
        self.rebuild(changed, removed_modules, identity_changes)
    }

    /// Applies several source overlays before rebuilding interfaces and links once.
    pub fn set_overlays<I>(&mut self, overlays: I) -> Result<WorkspaceModuleUpdate, WorkspaceModuleSessionError>
    where
        I: IntoIterator<Item = (SourceLocation, Arc<str>, SourceRevision)>,
    {
        self.set_overlay_batch(overlays.into_iter().map(|(source, text, revision)| (source, text, revision, None)))
    }

    /// Applies source overlays with parser output supplied by the boundary
    /// that already performed syntax recovery. The module session still owns
    /// source/module identity and linking; it accepts the recovered program
    /// as the source artifact so a syntax-error edit can publish semantic
    /// products for its valid portions without replacing the live text.
    pub fn set_overlays_with_programs<I>(&mut self, overlays: I) -> Result<WorkspaceModuleUpdate, WorkspaceModuleSessionError>
    where
        I: IntoIterator<Item = (SourceLocation, Arc<str>, SourceRevision, Arc<phalcom_ast::ast::Program>)>,
    {
        self.set_overlay_batch(
            overlays
                .into_iter()
                .map(|(source, text, revision, program)| (source, text, revision, Some(program))),
        )
    }

    fn set_overlay_batch<I>(&mut self, overlays: I) -> Result<WorkspaceModuleUpdate, WorkspaceModuleSessionError>
    where
        I: IntoIterator<Item = (SourceLocation, Arc<str>, SourceRevision, Option<Arc<phalcom_ast::ast::Program>>)>,
    {
        let before: BTreeSet<_> = self.sources_by_module.keys().cloned().collect();
        let previous_modules_by_source = self.modules_by_source.clone();
        let previous_sources_by_module = self.sources_by_module.clone();
        let previous_linked = self.linked.clone();
        let previous_generation = self.generation;
        let mut changed = BTreeSet::new();
        // Parse and resolve every item before mutating overlay/state maps. A
        // malformed batch therefore leaves the previous coherent publication
        // and source lifecycle intact.
        let mut staged = Vec::new();
        for (source, text, revision, program) in overlays {
            let module = self.module_for_location(&source)?;
            let kind = self.kind_for_source(&module, &source);
            let parsed = program.map_or_else(
                || parse_source(module.clone(), kind, source.clone(), text.clone()),
                |program| {
                    Ok(Arc::new(ParsedModuleUnit::new(
                        module.clone(),
                        kind,
                        Some(source.clone()),
                        text.clone(),
                        program,
                    )))
                },
            )?;
            staged.push((module, kind, source, revision, parsed));
        }
        for (module, kind, source, revision, parsed) in staged {
            self.provider
                .set_overlay(SourceOverlay::new(module.clone(), kind, source.clone(), parsed.text.clone()));
            self.insert_state(module.clone(), kind, source, revision, parsed, true);
            changed.insert(module);
        }
        self.generation = self.generation.saturating_add(1);
        let after: BTreeSet<_> = self.sources_by_module.keys().cloned().collect();
        let removed_modules = before.difference(&after).cloned().collect();
        match self.rebuild(changed, removed_modules, BTreeSet::new()) {
            Ok(update) => Ok(update),
            Err(error) => {
                self.modules_by_source = previous_modules_by_source;
                self.sources_by_module = previous_sources_by_module;
                self.linked = previous_linked;
                self.generation = previous_generation;
                self.provider.clear_overlays();
                for state in self.sources_by_module.values().filter(|state| state.open_overlay) {
                    self.provider
                        .set_overlay(SourceOverlay::new(state.module.clone(), state.kind, state.location.clone(), state.text.clone()));
                }
                Err(error)
            }
        }
    }

    pub fn set_overlay(
        &mut self,
        source: SourceLocation,
        text: Arc<str>,
        revision: SourceRevision,
    ) -> Result<WorkspaceModuleUpdate, WorkspaceModuleSessionError> {
        self.apply(WorkspaceSourceMutation::SetOverlay { source, text, revision })
    }

    pub fn remove_overlay(&mut self, source: SourceId) -> Result<WorkspaceModuleUpdate, WorkspaceModuleSessionError> {
        self.apply(WorkspaceSourceMutation::RemoveOverlay { source })
    }

    pub fn refresh_disk(&mut self, source: SourceLocation, revision: SourceRevision) -> Result<WorkspaceModuleUpdate, WorkspaceModuleSessionError> {
        self.apply(WorkspaceSourceMutation::RefreshDisk { source, revision })
    }

    pub fn remove_source(&mut self, source: SourceId) -> Result<WorkspaceModuleUpdate, WorkspaceModuleSessionError> {
        self.apply(WorkspaceSourceMutation::RemoveSource { source })
    }

    fn module_for_location(&mut self, location: &SourceLocation) -> Result<ModuleId, WorkspaceModuleSessionError> {
        if let Some(module) = self.modules_by_source.get(&location.source_id) {
            return Ok(module.clone());
        }

        let path = location.display_path.canonicalize().unwrap_or_else(|_| location.display_path.clone());
        if let Some(root) = discover_owning_project(&path)? {
            let root = root.canonicalize().unwrap_or(root);
            let project = if let Some(id) = self.project_roots.get(&ProjectSourceIdentity::from_path(root.clone())) {
                *id
            } else {
                let id = self.universe.load_root(root.join("project.toml"))?;
                self.project_roots.insert(ProjectSourceIdentity::from_path(root.clone()), id);
                id
            };
            let project_ref = self.universe.get_project(project).expect("loaded project is present");
            let unit = crate::source::resolve_source_path(project_ref, &path).map_err(WorkspaceModuleSessionError::from)?;
            return Ok(unit.id);
        }

        let root = path
            .parent()
            .ok_or_else(|| WorkspaceModuleSessionError::InvalidSourcePath(path.clone()))?
            .to_path_buf();
        if !root.is_dir() {
            let synthetic = *self
                .standalone_projects
                .entry(location.source_id.clone())
                .or_insert_with(|| self.synthetic_ids.allocate());
            return Ok(ModuleId::synthetic(synthetic, ModulePath::root()));
        }
        let root_identity = ProjectSourceIdentity::from_path(root.clone());
        let project = if let Some(id) = self.project_roots.get(&root_identity) {
            *id
        } else {
            let name = root
                .file_name()
                .and_then(|name| name.to_str())
                .map(|name| name.replace('-', "_"))
                .filter(|name| ModuleComponent::from_identifier(name).is_ok())
                .unwrap_or_else(|| "standalone".to_string());
            let id = self.universe.load_synthetic_root(&name, &root, "main")?;
            self.project_roots.insert(root_identity, id);
            id
        };
        let relative = path
            .strip_prefix(&root)
            .map_err(|_| WorkspaceModuleSessionError::InvalidSourcePath(path.clone()))?;
        let file_name = relative
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| WorkspaceModuleSessionError::InvalidSourcePath(path.clone()))?;
        let mut components = relative
            .parent()
            .into_iter()
            .flat_map(|parent| parent.iter())
            .map(|component| component.to_string_lossy().replace('-', "_"))
            .map(|component| ModuleComponent::from_identifier(&component))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_| WorkspaceModuleSessionError::InvalidSourcePath(path.clone()))?;
        if file_name != "package.ph" {
            let stem = file_name
                .strip_suffix(".ph")
                .ok_or_else(|| WorkspaceModuleSessionError::InvalidSourcePath(path.clone()))?
                .replace('-', "_");
            components.push(ModuleComponent::from_identifier(&stem).map_err(|_| WorkspaceModuleSessionError::InvalidSourcePath(path.clone()))?);
        }
        Ok(ModuleId::resolved(project, ModulePath::from_components(components)))
    }

    fn kind_for_source(&self, module: &ModuleId, location: &SourceLocation) -> ModuleKind {
        self.sources_by_module.get(module).map(|state| state.kind).unwrap_or_else(|| {
            if location.display_path.file_name().is_some_and(|name| name == "package.ph") {
                ModuleKind::Package
            } else {
                ModuleKind::Module
            }
        })
    }

    fn insert_state(
        &mut self,
        module: ModuleId,
        kind: ModuleKind,
        location: SourceLocation,
        revision: SourceRevision,
        parsed: Arc<ParsedModuleUnit>,
        open_overlay: bool,
    ) {
        self.modules_by_source.insert(location.source_id.clone(), module.clone());
        self.sources_by_module.insert(
            module.clone(),
            WorkspaceSourceState {
                module,
                kind,
                location,
                revision,
                text: parsed.text.clone(),
                parsed,
                open_overlay,
            },
        );
    }

    fn rebuild(
        &mut self,
        changed_modules: BTreeSet<ModuleId>,
        removed_modules: BTreeSet<ModuleId>,
        identity_changes: BTreeSet<ModuleId>,
    ) -> Result<WorkspaceModuleUpdate, WorkspaceModuleSessionError> {
        let mut parsed_sources = self
            .sources_by_module
            .iter()
            .map(|(id, state)| (id.clone(), state.parsed.clone()))
            .collect::<BTreeMap<_, _>>();
        let mut interfaces = BTreeMap::new();
        let mut resolved = BTreeMap::new();
        let mut queue = VecDeque::from_iter(parsed_sources.keys().cloned());
        let mut resolver = ModuleResolver::new(&self.universe, &self.provider);

        while let Some(module) = queue.pop_front() {
            let parsed = parsed_sources.get(&module).expect("queued source exists").clone();
            let interface = InterfaceBuilder::build(module.clone(), parsed.kind, &parsed.program)?;
            for import in &interface.imports {
                let path = match import {
                    ImportSurface::Module(decl) => &decl.path,
                    ImportSurface::Selective(decl) => &decl.path,
                    ImportSurface::ReExport(decl) => &decl.path,
                };
                let target = match resolver.resolve_import(&module, path) {
                    Ok(unit) => unit.id,
                    Err(error) if module.project.as_synthetic().is_some() => self
                        .resolve_standalone_import(&module, path)
                        .ok_or(WorkspaceModuleSessionError::Resolution(error))?,
                    Err(ModuleResolutionError::ModuleNotFound(_)) => continue,
                    Err(error) => return Err(WorkspaceModuleSessionError::Resolution(error)),
                };
                resolved.insert((module.clone(), path.to_string()), target.clone());
                if !parsed_sources.contains_key(&target) {
                    let loaded = resolver.load_parsed(&target)?;
                    parsed_sources.insert(target.clone(), loaded);
                    queue.push_back(target);
                }
            }
            interfaces.insert(module, interface);
        }

        let linked = if parsed_sources.is_empty() {
            Arc::new(LinkedProgram {
                universe: Arc::new(self.universe.clone()),
                modules: BTreeMap::new(),
                graphs: crate::graph::ModuleGraphs::default(),
                entry: ModuleId::core(),
                initialization_order: Vec::new(),
            })
        } else {
            let entry = parsed_sources.keys().next().cloned().expect("non-empty source map");
            let linker = ModuleLinker::new(Arc::new(self.universe.clone()), interfaces);
            let mut modules = BTreeMap::new();
            let mut graphs = crate::graph::ModuleGraphs::default();
            for component_entry in parsed_sources.keys() {
                let component = linker.link_with_unresolved_imports(component_entry.clone(), &resolved)?;
                modules.extend(component.modules);
                graphs.merge_from(&component.graphs);
            }
            let initialization_order = graphs.runtime.initialization_order().map_err(LinkError::RuntimeCycle)?;
            Arc::new(LinkedProgram {
                universe: Arc::new(self.universe.clone()),
                modules,
                graphs,
                entry,
                initialization_order,
            })
        };

        for (module, parsed) in &parsed_sources {
            if self.sources_by_module.contains_key(module) {
                continue;
            }
            if let Some(location) = parsed.source.clone() {
                self.insert_state(module.clone(), parsed.kind, location, SourceRevision::default(), parsed.clone(), false);
            }
        }
        self.linked = Some(linked.clone());
        Ok(WorkspaceModuleUpdate {
            linked,
            sources: parsed_sources,
            changed_modules,
            removed_modules,
            identity_changes,
        })
    }

    fn resolve_standalone_import(&self, importer: &ModuleId, path: &ImportPath) -> Option<ModuleId> {
        let ImportRoot::Relative { dots, .. } = path.root else {
            return None;
        };
        let importer_source = self.sources_by_module.get(importer)?.location.display_path.clone();
        let mut directory = importer_source.parent()?.to_path_buf();
        for _ in 1..dots {
            directory = directory.parent()?.to_path_buf();
        }
        for segment in &path.segments {
            directory.push(segment.name.replace('_', "-"));
        }
        let candidate = if path.segments.is_empty() {
            directory.join("package.ph")
        } else {
            directory.with_extension("ph")
        };
        let source_id = SourceId(candidate.to_string_lossy().into());
        if let Some(module) = self.modules_by_source.get(&source_id) {
            return Some(module.clone());
        }
        let canonical = candidate.canonicalize().ok()?;
        let canonical_source = SourceId(canonical.to_string_lossy().into());
        self.modules_by_source.get(&canonical_source).cloned()
    }
}

fn parse_source(module: ModuleId, kind: ModuleKind, location: SourceLocation, text: Arc<str>) -> Result<Arc<ParsedModuleUnit>, WorkspaceModuleSessionError> {
    let result = phalcom_ast::parse(&text, 0);
    if let Some(error) = result.errors.into_iter().next() {
        return Err(WorkspaceModuleSessionError::Parse {
            source_id: location.source_id,
            error,
        });
    }
    Ok(Arc::new(ParsedModuleUnit::new(module, kind, Some(location), text, Arc::new(result.program))))
}
