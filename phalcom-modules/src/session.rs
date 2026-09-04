//! Persistent compiler-owned module workspace lifecycle.

use crate::error::{InterfaceError, ModuleLoadError, ModuleResolutionError, ProjectError, SourceError};
use crate::identity::{
    ModuleId, ModulePath, ProjectSourceIdentity, SourceId, SourceLocation, SyntheticProjectId, SyntheticProjectIdAllocator,
};
use crate::interface::{ImportSurface, InterfaceBuilder};
use crate::linker::{LinkError, LinkedProgram, ModuleLinker};
use crate::manifest::DependencyProvider;
use crate::project::ProjectUniverse;
use crate::resolver::ModuleResolver;
use crate::source::{
    classify_entry_ownership, EntryOwnership, FilesystemSourceProvider, ModuleKind, OverlaySourceProvider, ParsedModuleUnit, SourceOverlay, SourceProvider,
};
use phalcom_ast::ast::Program;
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

/// Heterogeneous source mutation batch at module-infrastructure boundary.
#[derive(Clone, Debug)]
pub enum WorkspaceSourceBatchMutation {
    SetOverlay {
        source: SourceLocation,
        text: Arc<str>,
        revision: SourceRevision,
        recovered_program: Option<Arc<Program>>,
    },
    SetDiskSnapshot {
        source: SourceLocation,
        text: Arc<str>,
        revision: SourceRevision,
        recovered_program: Option<Arc<Program>>,
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

impl From<WorkspaceSourceMutation> for WorkspaceSourceBatchMutation {
    fn from(value: WorkspaceSourceMutation) -> Self {
        match value {
            WorkspaceSourceMutation::SetOverlay { source, text, revision } => Self::SetOverlay {
                source,
                text,
                revision,
                recovered_program: None,
            },
            WorkspaceSourceMutation::RemoveOverlay { source } => Self::RemoveOverlay { source },
            WorkspaceSourceMutation::RefreshDisk { source, revision } => Self::RefreshDisk { source, revision },
            WorkspaceSourceMutation::RemoveSource { source } => Self::RemoveSource { source },
        }
    }
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
    resolved_imports: BTreeMap<(ModuleId, String), ModuleId>,
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
            resolved_imports: BTreeMap::new(),
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

    /// Canonical resolver results keyed by importer and the written logical import path.
    pub fn resolved_imports(&self) -> &BTreeMap<(ModuleId, String), ModuleId> {
        &self.resolved_imports
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
        self.apply_batch(std::iter::once(mutation.into()))
    }

    pub fn apply_batch<I>(&mut self, mutations: I) -> Result<WorkspaceModuleUpdate, WorkspaceModuleSessionError>
    where
        I: IntoIterator<Item = WorkspaceSourceBatchMutation>,
    {
        let mut staged = Self {
            universe: self.universe.clone(),
            provider: OverlaySourceProvider::new(FilesystemSourceProvider::new()),
            project_roots: self.project_roots.clone(),
            modules_by_source: self.modules_by_source.clone(),
            sources_by_module: self.sources_by_module.clone(),
            standalone_projects: self.standalone_projects.clone(),
            synthetic_ids: self.synthetic_ids.clone(),
            linked: self.linked.clone(),
            resolved_imports: self.resolved_imports.clone(),
            generation: self.generation,
        };
        Self::seed_open_overlays(&staged.provider, &staged.sources_by_module);

        let before: BTreeSet<_> = self.sources_by_module.keys().cloned().collect();
        let mut changed = BTreeSet::new();
        let mut identity_changes = BTreeSet::new();
        let mut clear_base_cache = false;

        for mutation in mutations {
            match mutation {
                WorkspaceSourceBatchMutation::SetOverlay {
                    source,
                    text,
                    revision,
                    recovered_program,
                } => {
                    let module = staged.module_for_location(&source)?;
                    let kind = staged.kind_for_source(&module, &source);
                    let text_for_parse = text.clone();
                    let parsed = recovered_program.map_or_else(
                        || parse_source(module.clone(), kind, source.clone(), text_for_parse),
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
                    staged
                        .provider
                        .set_overlay(SourceOverlay::new(module.clone(), kind, source.clone(), parsed.text.clone()));
                    staged.insert_state(module.clone(), kind, source, revision, parsed, true);
                    changed.insert(module);
                }
                WorkspaceSourceBatchMutation::SetDiskSnapshot {
                    source,
                    text,
                    revision,
                    recovered_program,
                } => {
                    let module = staged.module_for_location(&source)?;
                    let kind = staged.kind_for_source(&module, &source);
                    let text_for_parse = text.clone();
                    let parsed = recovered_program.map_or_else(
                        || parse_source(module.clone(), kind, source.clone(), text_for_parse),
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
                    staged.provider.remove_overlay(&module);
                    staged.insert_state(module.clone(), kind, source, revision, parsed, false);
                    changed.insert(module);
                }
                WorkspaceSourceBatchMutation::RemoveOverlay { source } => {
                    let module = staged
                        .modules_by_source
                        .get(&source)
                        .cloned()
                        .ok_or_else(|| WorkspaceModuleSessionError::UnknownSource(source.clone()))?;
                    staged.provider.remove_overlay(&module);
                    let state = staged
                        .sources_by_module
                        .get(&module)
                        .cloned()
                        .ok_or_else(|| WorkspaceModuleSessionError::UnknownSource(source.clone()))?;
                    let text = staged.provider.base().read(&source)?;
                    let parsed = parse_source(module.clone(), state.kind, state.location.clone(), text)?;
                    staged.insert_state(module.clone(), state.kind, state.location, state.revision, parsed, false);
                    changed.insert(module);
                }
                WorkspaceSourceBatchMutation::RefreshDisk { source, revision } => {
                    let module = staged.module_for_location(&source)?;
                    if let Some(existing) = staged.sources_by_module.get(&module) {
                        if existing.open_overlay {
                            staged.provider.remove_overlay(&module);
                        }
                    }
                    staged.provider.base().clear_cache();
                    clear_base_cache = true;
                    let text = staged.provider.base().read(&source.source_id)?;
                    let kind = staged.kind_for_source(&module, &source);
                    let parsed = parse_source(module.clone(), kind, source.clone(), text)?;
                    staged.insert_state(module.clone(), kind, source, revision, parsed, false);
                    changed.insert(module);
                }
                WorkspaceSourceBatchMutation::RemoveSource { source } => {
                    let module = staged
                        .modules_by_source
                        .remove(&source)
                        .ok_or_else(|| WorkspaceModuleSessionError::UnknownSource(source.clone()))?;
                    staged.provider.remove_overlay(&module);
                    staged.provider.base().clear_cache();
                    clear_base_cache = true;
                    staged.sources_by_module.remove(&module);
                    identity_changes.insert(module);
                }
            }
        }

        staged.generation = staged.generation.saturating_add(1);
        let after: BTreeSet<_> = staged.sources_by_module.keys().cloned().collect();
        let removed_modules = before.difference(&after).cloned().collect();
        let update = staged.rebuild(changed, removed_modules, identity_changes)?;

        self.universe = staged.universe;
        self.project_roots = staged.project_roots;
        self.modules_by_source = staged.modules_by_source;
        self.sources_by_module = staged.sources_by_module;
        self.standalone_projects = staged.standalone_projects;
        self.synthetic_ids = staged.synthetic_ids;
        self.linked = staged.linked;
        self.resolved_imports = staged.resolved_imports;
        self.generation = staged.generation;
        self.provider.clear_overlays();
        if clear_base_cache {
            self.provider.base().clear_cache();
        }
        Self::seed_open_overlays(&self.provider, &self.sources_by_module);
        Ok(update)
    }

    /// Applies several source overlays before rebuilding interfaces and links once.
    pub fn set_overlays<I>(&mut self, overlays: I) -> Result<WorkspaceModuleUpdate, WorkspaceModuleSessionError>
    where
        I: IntoIterator<Item = (SourceLocation, Arc<str>, SourceRevision)>,
    {
        self.apply_batch(overlays.into_iter().map(|(source, text, revision)| WorkspaceSourceBatchMutation::SetOverlay {
            source,
            text,
            revision,
            recovered_program: None,
        }))
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
        self.apply_batch(
            overlays
                .into_iter()
                .map(|(source, text, revision, recovered_program)| WorkspaceSourceBatchMutation::SetOverlay {
                    source,
                    text,
                    revision,
                    recovered_program: Some(recovered_program),
                }),
        )
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

        let path = crate::source::canonicalize_path(&location.display_path);
        let ownership = classify_entry_ownership(&path, &mut self.universe)?;
        match ownership {
            EntryOwnership::ProjectOwned { project } => {
                let project_ref = self.universe.get_project(project).expect("loaded project is present");
                self.project_roots.insert(ProjectSourceIdentity::from_path(&project_ref.root_dir), project);
                let unit = crate::source::resolve_source_path(project_ref, &path).map_err(WorkspaceModuleSessionError::from)?;
                Ok(unit.id)
            }
            EntryOwnership::StandalonePackageOwned { package_root } => {
                let project_id = self.universe.load_standalone_package(&package_root, None)?;
                self.project_roots.insert(ProjectSourceIdentity::from_path(&package_root), project_id);
                let project_ref = self.universe.get_project(project_id).expect("loaded package is present");
                let unit = crate::source::resolve_source_path(project_ref, &path).map_err(WorkspaceModuleSessionError::from)?;
                Ok(unit.id)
            }
            EntryOwnership::StandaloneModule { file: _ } => {
                let synthetic = *self
                    .standalone_projects
                    .entry(location.source_id.clone())
                    .or_insert_with(|| self.synthetic_ids.allocate());
                Ok(ModuleId::synthetic(synthetic, ModulePath::root()))
            }
            EntryOwnership::Inline { synthetic } => {
                Ok(ModuleId::synthetic(synthetic, ModulePath::root()))
            }
        }
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

    fn seed_open_overlays(provider: &OverlaySourceProvider<FilesystemSourceProvider>, sources_by_module: &BTreeMap<ModuleId, WorkspaceSourceState>) {
        for state in sources_by_module.values().filter(|state| state.open_overlay) {
            provider.set_overlay(SourceOverlay::new(state.module.clone(), state.kind, state.location.clone(), state.text.clone()));
        }
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
                entry: ModuleId::universe_root(),
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
        self.resolved_imports = resolved;
        self.linked = Some(linked.clone());
        Ok(WorkspaceModuleUpdate {
            linked,
            sources: parsed_sources,
            changed_modules,
            removed_modules,
            identity_changes,
        })
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    fn write_source(root: &std::path::Path, name: &str, text: &str) -> SourceLocation {
        let path = root.join(name);
        let content = if text.ends_with('\n') { text.to_string() } else { format!("{text}\n") };
        fs::write(&path, content).unwrap();
        SourceLocation {
            source_id: SourceId(path.to_string_lossy().into()),
            display_path: path,
        }
    }

    fn parse_program(text: &str) -> Arc<Program> {
        let owned = if text.ends_with('\n') { text.to_string() } else { format!("{text}\n") };
        let result = phalcom_ast::parse(&owned, 0);
        assert!(result.errors.is_empty(), "test fixture should parse cleanly");
        Arc::new(result.program)
    }

    #[test]
    fn batch_updates_multiple_sources_with_one_generation_increment() {
        let root = tempdir().unwrap();
        let a = write_source(root.path(), "a.ph", "class A {}");
        let b = write_source(root.path(), "b.ph", "class B {}");
        let mut session = WorkspaceModuleSession::new();

        let update = session
            .apply_batch([
                WorkspaceSourceBatchMutation::SetOverlay {
                    source: a.clone(),
                    text: Arc::from("class A2 {}\n"),
                    revision: SourceRevision(1),
                    recovered_program: None,
                },
                WorkspaceSourceBatchMutation::SetOverlay {
                    source: b.clone(),
                    text: Arc::from("class B2 {}\n"),
                    revision: SourceRevision(2),
                    recovered_program: None,
                },
            ])
            .unwrap();

        assert_eq!(session.generation(), 1);
        assert_eq!(session.sources().len(), 2);
        assert_eq!(update.changed_modules.len(), 2);
        assert!(update.removed_modules.is_empty());
    }

    #[test]
    fn batch_update_and_remove_rebuild_once_and_publish_consistent_sources() {
        let root = tempdir().unwrap();
        let a = write_source(root.path(), "a.ph", "class A {}");
        let b = write_source(root.path(), "b.ph", "class B {}");
        let mut session = WorkspaceModuleSession::new();

        session
            .apply_batch([
                WorkspaceSourceBatchMutation::SetOverlay {
                    source: a.clone(),
                    text: Arc::from("class A {}\n"),
                    revision: SourceRevision(1),
                    recovered_program: None,
                },
                WorkspaceSourceBatchMutation::SetOverlay {
                    source: b.clone(),
                    text: Arc::from("class B {}\n"),
                    revision: SourceRevision(2),
                    recovered_program: None,
                },
            ])
            .unwrap();

        let a_module = session.module_for_source(&a.source_id).cloned().unwrap();
        let b_module = session.module_for_source(&b.source_id).cloned().unwrap();
        let generation_before = session.generation();

        let update = session
            .apply_batch([
                WorkspaceSourceBatchMutation::SetOverlay {
                    source: a.clone(),
                    text: Arc::from("class A2 {}\n"),
                    revision: SourceRevision(3),
                    recovered_program: None,
                },
                WorkspaceSourceBatchMutation::RemoveSource { source: b.source_id.clone() },
            ])
            .unwrap();

        assert_eq!(session.generation(), generation_before + 1);
        assert_eq!(update.changed_modules, BTreeSet::from([a_module.clone()]));
        assert_eq!(update.removed_modules, BTreeSet::from([b_module.clone()]));
        assert_eq!(session.source(&a_module).unwrap().text.as_ref(), "class A2 {}\n");
        assert!(session.source(&b_module).is_none());
        assert!(session.module_for_source(&b.source_id).is_none());
    }

    #[test]
    fn batch_accepts_recovered_program_for_invalid_live_text() {
        let root = tempdir().unwrap();
        let source = write_source(root.path(), "a.ph", "class A {}");
        let mut session = WorkspaceModuleSession::new();
        let recovered_program = parse_program("class Recovery {}");
        let invalid_text: Arc<str> = Arc::from("class A {");

        let update = session
            .apply_batch([WorkspaceSourceBatchMutation::SetOverlay {
                source: source.clone(),
                text: invalid_text.clone(),
                revision: SourceRevision(1),
                recovered_program: Some(recovered_program.clone()),
            }])
            .unwrap();

        let module = session.module_for_source(&source.source_id).cloned().unwrap();
        let state = session.source(&module).unwrap();
        assert_eq!(update.changed_modules, BTreeSet::from([module.clone()]));
        assert_eq!(state.text.as_ref(), invalid_text.as_ref());
        assert!(Arc::ptr_eq(&state.parsed.program, &recovered_program));
        assert!(Arc::ptr_eq(&update.sources[&module].program, &recovered_program));
    }

    #[test]
    fn batch_rolls_back_when_rebuild_fails() {
        let root = tempdir().unwrap();
        let a = write_source(root.path(), "a.ph", "class A {}");
        let b = write_source(root.path(), "b.ph", "class B {}");
        let mut session = WorkspaceModuleSession::new();

        session
            .apply_batch([
                WorkspaceSourceBatchMutation::SetOverlay {
                    source: a.clone(),
                    text: Arc::from("class A {}\n"),
                    revision: SourceRevision(1),
                    recovered_program: None,
                },
                WorkspaceSourceBatchMutation::SetOverlay {
                    source: b.clone(),
                    text: Arc::from("class B {}\n"),
                    revision: SourceRevision(2),
                    recovered_program: None,
                },
            ])
            .unwrap();

        let a_module = session.module_for_source(&a.source_id).cloned().unwrap();
        let b_module = session.module_for_source(&b.source_id).cloned().unwrap();
        let generation_before = session.generation();
        let linked_before = session.linked().cloned().unwrap();
        let a_before = session.source(&a_module).unwrap().text.clone();
        let b_before = session.source(&b_module).unwrap().text.clone();

        let err = session
            .apply_batch([
                WorkspaceSourceBatchMutation::SetOverlay {
                    source: a.clone(),
                    text: Arc::from("class A2 {}\n"),
                    revision: SourceRevision(3),
                    recovered_program: None,
                },
                WorkspaceSourceBatchMutation::SetOverlay {
                    source: b.clone(),
                    text: Arc::from("class B {} class B {}\n"),
                    revision: SourceRevision(4),
                    recovered_program: None,
                },
            ])
            .unwrap_err();

        assert!(matches!(
            err,
            WorkspaceModuleSessionError::Interface(_) | WorkspaceModuleSessionError::Parse { .. } | WorkspaceModuleSessionError::Link(_)
        ));
        assert_eq!(session.generation(), generation_before);
        assert!(Arc::ptr_eq(session.linked().unwrap(), &linked_before));
        assert_eq!(session.source(&a_module).unwrap().text, a_before);
        assert_eq!(session.source(&b_module).unwrap().text, b_before);
    }
}
