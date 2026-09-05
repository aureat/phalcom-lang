//! Persistent compiler-owned module workspace lifecycle.

use crate::diagnostic::ModuleDiagnostic;
use crate::error::{InterfaceError, ModuleLoadError, ModuleResolutionError, ProjectError, SourceError};
use crate::identity::{
    ModuleId, ModulePath, ProjectSourceIdentity, SourceId, SourceLocation, SyntheticProjectId, SyntheticProjectIdAllocator,
};
use crate::interface::{ImportSurface, InterfaceBuilder, UnlinkedModuleInterface};
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
    pub interfaces: BTreeMap<ModuleId, Arc<UnlinkedModuleInterface>>,
    pub diagnostics: BTreeMap<ModuleId, Vec<ModuleDiagnostic>>,
    pub blocked_modules: BTreeSet<ModuleId>,
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
    interfaces: BTreeMap<ModuleId, (Arc<UnlinkedModuleInterface>, crate::fingerprint::InterfaceFingerprint)>,
    import_products: BTreeMap<(ModuleId, String), Arc<crate::resolver::ImportResolutionProduct>>,
    reverse_importers: BTreeMap<ModuleId, BTreeSet<ModuleId>>,
    linked_modules: BTreeMap<ModuleId, (crate::linker::LinkedModule, crate::fingerprint::LinkedInterfaceFingerprint)>,
    linked: Option<Arc<LinkedProgram>>,
    resolved_imports: BTreeMap<(ModuleId, String), ModuleId>,
    diagnostics: BTreeMap<ModuleId, Vec<ModuleDiagnostic>>,
    blocked_modules: BTreeSet<ModuleId>,
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
            interfaces: BTreeMap::new(),
            import_products: BTreeMap::new(),
            reverse_importers: BTreeMap::new(),
            linked_modules: BTreeMap::new(),
            linked: None,
            resolved_imports: BTreeMap::new(),
            diagnostics: BTreeMap::new(),
            blocked_modules: BTreeSet::new(),
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

    pub fn interfaces(&self) -> &BTreeMap<ModuleId, (Arc<UnlinkedModuleInterface>, crate::fingerprint::InterfaceFingerprint)> {
        &self.interfaces
    }

    pub fn interface(&self, module: &ModuleId) -> Option<&Arc<UnlinkedModuleInterface>> {
        self.interfaces.get(module).map(|(iface, _)| iface)
    }

    pub fn import_products(&self) -> &BTreeMap<(ModuleId, String), Arc<crate::resolver::ImportResolutionProduct>> {
        &self.import_products
    }

    pub fn reverse_importers(&self) -> &BTreeMap<ModuleId, BTreeSet<ModuleId>> {
        &self.reverse_importers
    }

    pub fn linked_modules(&self) -> &BTreeMap<ModuleId, (crate::linker::LinkedModule, crate::fingerprint::LinkedInterfaceFingerprint)> {
        &self.linked_modules
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

    pub fn diagnostics(&self) -> &BTreeMap<ModuleId, Vec<ModuleDiagnostic>> {
        &self.diagnostics
    }

    pub fn blocked_modules(&self) -> &BTreeSet<ModuleId> {
        &self.blocked_modules
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
        self.interfaces.clear();
        self.import_products.clear();
        self.reverse_importers.clear();
        self.linked_modules.clear();
        self.linked = None;
        self.resolved_imports.clear();
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
        enum OverlayOp {
            Set(SourceOverlay),
            Remove(ModuleId),
        }

        let mut mutated_sources = BTreeMap::new();
        let mut removed_sources = BTreeSet::new();
        let mut mutated_modules_by_source = BTreeMap::new();
        let mut removed_modules_by_source = BTreeSet::new();
        let mut mutated_project_roots = BTreeMap::new();
        let mut mutated_standalone_projects = BTreeMap::new();
        let mut synthetic_ids = self.synthetic_ids.clone();
        let mut universe_override: Option<ProjectUniverse> = None;
        let mut changed_modules = BTreeSet::new();
        let mut identity_changes = BTreeSet::new();
        let mut overlay_ops = Vec::new();
        let mut content_invalidations = BTreeSet::new();
        let mut topology_invalidations = false;
        let mut purged_identities = BTreeSet::new();

        let get_module_for_source = |source_id: &SourceId,
                                     mutated_modules: &BTreeMap<SourceId, ModuleId>,
                                     removed_modules: &BTreeSet<SourceId>,
                                     committed_modules: &BTreeMap<SourceId, ModuleId>|
         -> Option<ModuleId> {
            if removed_modules.contains(source_id) {
                return None;
            }
            if let Some(m) = mutated_modules.get(source_id) {
                return Some(m.clone());
            }
            committed_modules.get(source_id).cloned()
        };

        for mutation in mutations {
            match mutation {
                WorkspaceSourceBatchMutation::SetOverlay {
                    source,
                    text,
                    revision,
                    recovered_program,
                } => {
                    content_invalidations.insert(source.source_id.clone());
                    let module = Self::resolve_module_for_location_delta(
                        &source,
                        &self.universe,
                        &self.modules_by_source,
                        &self.standalone_projects,
                        &mut mutated_modules_by_source,
                        &removed_modules_by_source,
                        &mut mutated_project_roots,
                        &mut mutated_standalone_projects,
                        &mut synthetic_ids,
                        &mut universe_override,
                    )?;
                    let kind = Self::kind_for_source_delta(
                        &module,
                        &source,
                        &mutated_sources,
                        &removed_sources,
                        &self.sources_by_module,
                    );
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
                    overlay_ops.push(OverlayOp::Set(SourceOverlay::new(
                        module.clone(),
                        kind,
                        source.clone(),
                        parsed.text.clone(),
                    )));
                    let state = WorkspaceSourceState {
                        module: module.clone(),
                        kind,
                        location: source.clone(),
                        revision,
                        text: parsed.text.clone(),
                        parsed,
                        open_overlay: true,
                    };
                    mutated_modules_by_source.insert(source.source_id.clone(), module.clone());
                    removed_modules_by_source.remove(&source.source_id);
                    if !self.sources_by_module.contains_key(&module) && !mutated_sources.contains_key(&module) {
                        topology_invalidations = true;
                    }
                    mutated_sources.insert(module.clone(), state);
                    removed_sources.remove(&module);
                    changed_modules.insert(module);
                }
                WorkspaceSourceBatchMutation::SetDiskSnapshot {
                    source,
                    text,
                    revision,
                    recovered_program,
                } => {
                    content_invalidations.insert(source.source_id.clone());
                    let module = Self::resolve_module_for_location_delta(
                        &source,
                        &self.universe,
                        &self.modules_by_source,
                        &self.standalone_projects,
                        &mut mutated_modules_by_source,
                        &removed_modules_by_source,
                        &mut mutated_project_roots,
                        &mut mutated_standalone_projects,
                        &mut synthetic_ids,
                        &mut universe_override,
                    )?;
                    let kind = Self::kind_for_source_delta(
                        &module,
                        &source,
                        &mutated_sources,
                        &removed_sources,
                        &self.sources_by_module,
                    );
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
                    overlay_ops.push(OverlayOp::Remove(module.clone()));
                    let state = WorkspaceSourceState {
                        module: module.clone(),
                        kind,
                        location: source.clone(),
                        revision,
                        text: parsed.text.clone(),
                        parsed,
                        open_overlay: false,
                    };
                    mutated_modules_by_source.insert(source.source_id.clone(), module.clone());
                    removed_modules_by_source.remove(&source.source_id);
                    if !self.sources_by_module.contains_key(&module) && !mutated_sources.contains_key(&module) {
                        topology_invalidations = true;
                    }
                    mutated_sources.insert(module.clone(), state);
                    removed_sources.remove(&module);
                    changed_modules.insert(module);
                }
                WorkspaceSourceBatchMutation::RemoveOverlay { source } => {
                    let module = get_module_for_source(
                        &source,
                        &mutated_modules_by_source,
                        &removed_modules_by_source,
                        &self.modules_by_source,
                    )
                    .ok_or_else(|| WorkspaceModuleSessionError::UnknownSource(source.clone()))?;
                    let state = if let Some(s) = mutated_sources.get(&module) {
                        s.clone()
                    } else if !removed_sources.contains(&module) {
                        self.sources_by_module
                            .get(&module)
                            .cloned()
                            .ok_or_else(|| WorkspaceModuleSessionError::UnknownSource(source.clone()))?
                    } else {
                        return Err(WorkspaceModuleSessionError::UnknownSource(source.clone()));
                    };
                    overlay_ops.push(OverlayOp::Remove(module.clone()));
                    let text = self.provider.base().read(&source)?;
                    let parsed = parse_source(module.clone(), state.kind, state.location.clone(), text)?;
                    let updated = WorkspaceSourceState {
                        module: module.clone(),
                        kind: state.kind,
                        location: state.location,
                        revision: state.revision,
                        text: parsed.text.clone(),
                        parsed,
                        open_overlay: false,
                    };
                    mutated_sources.insert(module.clone(), updated);
                    changed_modules.insert(module);
                }
                WorkspaceSourceBatchMutation::RefreshDisk { source, revision } => {
                    content_invalidations.insert(source.source_id.clone());
                    self.provider.base().invalidate_source_content(&source.source_id);
                    let module = Self::resolve_module_for_location_delta(
                        &source,
                        &self.universe,
                        &self.modules_by_source,
                        &self.standalone_projects,
                        &mut mutated_modules_by_source,
                        &removed_modules_by_source,
                        &mut mutated_project_roots,
                        &mut mutated_standalone_projects,
                        &mut synthetic_ids,
                        &mut universe_override,
                    )?;
                    let had_overlay = mutated_sources
                        .get(&module)
                        .map(|s| s.open_overlay)
                        .or_else(|| self.sources_by_module.get(&module).map(|s| s.open_overlay))
                        .unwrap_or(false);
                    if had_overlay {
                        overlay_ops.push(OverlayOp::Remove(module.clone()));
                    }
                    let text = self.provider.base().read(&source.source_id)?;
                    let kind = Self::kind_for_source_delta(
                        &module,
                        &source,
                        &mutated_sources,
                        &removed_sources,
                        &self.sources_by_module,
                    );
                    let parsed = parse_source(module.clone(), kind, source.clone(), text)?;
                    let state = WorkspaceSourceState {
                        module: module.clone(),
                        kind,
                        location: source.clone(),
                        revision,
                        text: parsed.text.clone(),
                        parsed,
                        open_overlay: false,
                    };
                    mutated_modules_by_source.insert(source.source_id.clone(), module.clone());
                    removed_modules_by_source.remove(&source.source_id);
                    mutated_sources.insert(module.clone(), state);
                    removed_sources.remove(&module);
                    changed_modules.insert(module);
                }
                WorkspaceSourceBatchMutation::RemoveSource { source } => {
                    let module = get_module_for_source(
                        &source,
                        &mutated_modules_by_source,
                        &removed_modules_by_source,
                        &self.modules_by_source,
                    )
                    .ok_or_else(|| WorkspaceModuleSessionError::UnknownSource(source.clone()))?;
                    removed_modules_by_source.insert(source.clone());
                    mutated_modules_by_source.remove(&source);
                    removed_sources.insert(module.clone());
                    mutated_sources.remove(&module);
                    overlay_ops.push(OverlayOp::Remove(module.clone()));
                    purged_identities.insert(source);
                    topology_invalidations = true;
                    identity_changes.insert(module);
                }
            }
        }

        // Apply invalidations to provider base cache
        for id in content_invalidations {
            self.provider.base().invalidate_source_content(&id);
        }
        for id in purged_identities {
            self.provider.base().purge_source_identity(&id);
        }
        if topology_invalidations {
            self.provider.base().invalidate_topology();
        }

        // Apply overlay operations
        for op in overlay_ops {
            match op {
                OverlayOp::Set(overlay) => self.provider.set_overlay(overlay),
                OverlayOp::Remove(module) => self.provider.remove_overlay(&module),
            }
        }

        // Apply mutated maps to self
        for id in removed_modules_by_source {
            self.modules_by_source.remove(&id);
        }
        self.modules_by_source.extend(mutated_modules_by_source);

        let removed_modules = removed_sources;
        for id in &removed_modules {
            self.sources_by_module.remove(id);
        }
        self.sources_by_module.extend(mutated_sources);

        self.project_roots.extend(mutated_project_roots);
        self.standalone_projects.extend(mutated_standalone_projects);
        self.synthetic_ids = synthetic_ids;
        if let Some(u) = universe_override {
            self.universe = u;
        }

        self.generation = self.generation.saturating_add(1);
        self.rebuild(changed_modules, removed_modules, identity_changes)
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

    fn resolve_module_for_location_delta(
        location: &SourceLocation,
        base_universe: &ProjectUniverse,
        committed_modules: &BTreeMap<SourceId, ModuleId>,
        committed_standalone: &BTreeMap<SourceId, SyntheticProjectId>,
        mutated_modules: &mut BTreeMap<SourceId, ModuleId>,
        removed_modules: &BTreeSet<SourceId>,
        project_roots: &mut BTreeMap<ProjectSourceIdentity, crate::identity::ResolvedProjectId>,
        standalone_projects: &mut BTreeMap<SourceId, SyntheticProjectId>,
        synthetic_ids: &mut SyntheticProjectIdAllocator,
        universe_override: &mut Option<ProjectUniverse>,
    ) -> Result<ModuleId, WorkspaceModuleSessionError> {
        if !removed_modules.contains(&location.source_id) {
            if let Some(m) = mutated_modules.get(&location.source_id) {
                return Ok(m.clone());
            }
            if let Some(m) = committed_modules.get(&location.source_id) {
                return Ok(m.clone());
            }
        }

        let path = crate::source::canonicalize_path(&location.display_path);
        let u = universe_override.get_or_insert_with(|| base_universe.clone());
        let ownership = classify_entry_ownership(&path, u)?;
        match ownership {
            EntryOwnership::ProjectOwned { project } => {
                let project_ref = u.get_project(project).expect("loaded project is present");
                project_roots.insert(ProjectSourceIdentity::from_path(&project_ref.root_dir), project);
                let unit = crate::source::resolve_source_path(project_ref, &path).map_err(WorkspaceModuleSessionError::from)?;
                mutated_modules.insert(location.source_id.clone(), unit.id.clone());
                Ok(unit.id)
            }
            EntryOwnership::StandalonePackageOwned { package_root } => {
                let project_id = u.load_standalone_package(&package_root, None)?;
                project_roots.insert(ProjectSourceIdentity::from_path(&package_root), project_id);
                let project_ref = u.get_project(project_id).expect("loaded package is present");
                let unit = crate::source::resolve_source_path(project_ref, &path).map_err(WorkspaceModuleSessionError::from)?;
                mutated_modules.insert(location.source_id.clone(), unit.id.clone());
                Ok(unit.id)
            }
            EntryOwnership::StandaloneModule { file: _ } => {
                let sid = if let Some(sid) = standalone_projects.get(&location.source_id).copied() {
                    sid
                } else if let Some(sid) = committed_standalone.get(&location.source_id).copied() {
                    sid
                } else {
                    let allocated = synthetic_ids.allocate();
                    standalone_projects.insert(location.source_id.clone(), allocated);
                    allocated
                };
                let mid = ModuleId::synthetic(sid, ModulePath::root());
                mutated_modules.insert(location.source_id.clone(), mid.clone());
                Ok(mid)
            }
            EntryOwnership::Inline { synthetic } => {
                Ok(ModuleId::synthetic(synthetic, ModulePath::root()))
            }
        }
    }

    fn kind_for_source_delta(
        module: &ModuleId,
        location: &SourceLocation,
        mutated_sources: &BTreeMap<ModuleId, WorkspaceSourceState>,
        removed_sources: &BTreeSet<ModuleId>,
        committed_sources: &BTreeMap<ModuleId, WorkspaceSourceState>,
    ) -> ModuleKind {
        if let Some(s) = mutated_sources.get(module) {
            return s.kind;
        }
        if !removed_sources.contains(module) {
            if let Some(s) = committed_sources.get(module) {
                return s.kind;
            }
        }
        if location.display_path.file_name().is_some_and(|name| name == "package.ph") {
            ModuleKind::Package
        } else {
            ModuleKind::Module
        }
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
        // 1. Invalidate removed modules from products
        for removed in &removed_modules {
            self.interfaces.remove(removed);
            self.linked_modules.remove(removed);
            self.import_products.retain(|(importer, _), _| importer != removed);
            self.resolved_imports.retain(|(importer, _), _| importer != removed);
            self.reverse_importers.remove(removed);
            self.diagnostics.remove(removed);
            self.blocked_modules.remove(removed);
            for importers in self.reverse_importers.values_mut() {
                importers.remove(removed);
            }
        }

        let mut diagnostics: BTreeMap<ModuleId, Vec<ModuleDiagnostic>> = BTreeMap::new();
        let mut blocked_modules: BTreeSet<ModuleId> = BTreeSet::new();

        // 2. Build or check unlinked interfaces for changed/new modules
        let mut modules_with_changed_interface = BTreeSet::new();
        for (module, state) in &self.sources_by_module {
            if !changed_modules.contains(module) && self.interfaces.contains_key(module) {
                continue;
            }
            match InterfaceBuilder::build(module.clone(), state.kind, &state.parsed.program) {
                Ok(interface) => {
                    let new_fp = interface.fingerprint();
                    let old_fp = self.interfaces.get(module).map(|(_, fp)| *fp);
                    let fp_changed = old_fp != Some(new_fp);
                    self.interfaces.insert(module.clone(), (Arc::new(interface), new_fp));
                    if fp_changed {
                        modules_with_changed_interface.insert(module.clone());
                    }
                }
                Err(err) => {
                    let diag = ModuleDiagnostic::from_interface_error(module.clone(), err);
                    diagnostics.entry(module.clone()).or_default().push(diag);
                    blocked_modules.insert(module.clone());
                }
            }
        }

        // 3. Body-only edit short-circuit:
        // If no interface fingerprints changed, no modules were removed, no identities changed,
        // and we already have a valid linked program, STOP PROPAGATION!
        if modules_with_changed_interface.is_empty()
            && removed_modules.is_empty()
            && identity_changes.is_empty()
            && self.linked.is_some()
        {
            let sources = self
                .sources_by_module
                .iter()
                .map(|(id, state)| (id.clone(), state.parsed.clone()))
                .collect();
            let interfaces = self
                .interfaces
                .iter()
                .map(|(id, (iface, _))| (id.clone(), iface.clone()))
                .collect();
            return Ok(WorkspaceModuleUpdate {
                linked: self.linked.clone().unwrap(),
                sources,
                interfaces,
                diagnostics: self.diagnostics.clone(),
                blocked_modules: self.blocked_modules.clone(),
                changed_modules,
                removed_modules,
                identity_changes,
            });
        }

        // 4. Determine which modules require import resolution re-evaluation
        let mut modules_to_resolve = modules_with_changed_interface.clone();
        for removed in &removed_modules {
            if let Some(importers) = self.reverse_importers.get(removed) {
                modules_to_resolve.extend(importers.iter().cloned());
            }
        }
        for changed in &modules_with_changed_interface {
            if let Some(importers) = self.reverse_importers.get(changed) {
                modules_to_resolve.extend(importers.iter().cloned());
            }
        }

        let mut parsed_sources = self
            .sources_by_module
            .iter()
            .map(|(id, state)| (id.clone(), state.parsed.clone()))
            .collect::<BTreeMap<_, _>>();

        let mut resolver = ModuleResolver::new(&self.universe, &self.provider);
        let mut queue = VecDeque::from_iter(self.sources_by_module.keys().cloned());

        while let Some(module) = queue.pop_front() {
            let Some((interface, _)) = self.interfaces.get(&module).cloned() else {
                continue;
            };
            let must_recompute = modules_to_resolve.contains(&module);

            for import in &interface.imports {
                let (path_syntax, path_str, import_range) = match import {
                    ImportSurface::Module(decl) => (&decl.path, decl.path.to_string(), decl.range),
                    ImportSurface::Selective(decl) => (&decl.path, decl.path.to_string(), decl.range),
                    ImportSurface::ReExport(decl) => (&decl.path, decl.path.to_string(), decl.range),
                };
                let key = (module.clone(), path_str);

                let target_id = if !must_recompute && self.resolved_imports.contains_key(&key) {
                    self.resolved_imports.get(&key).cloned().unwrap()
                } else {
                    let product = resolver.resolve_import_product(&module, path_syntax);
                    let target_res = product.target.clone();
                    self.import_products.insert(key.clone(), Arc::new(product));

                    match target_res {
                        Ok(target) => {
                            self.reverse_importers.entry(target.clone()).or_default().insert(module.clone());
                            self.resolved_imports.insert(key.clone(), target.clone());
                            target
                        }
                        Err(error) => {
                            self.resolved_imports.remove(&key);
                            let diag = ModuleDiagnostic::from_resolution_error(module.clone(), error, import_range);
                            diagnostics.entry(module.clone()).or_default().push(diag);
                            blocked_modules.insert(module.clone());
                            continue;
                        }
                    }
                };

                if !parsed_sources.contains_key(&target_id) {
                    let loaded = resolver.load_parsed(&target_id)?;
                    let loaded_iface = InterfaceBuilder::build(target_id.clone(), loaded.kind, &loaded.program)?;
                    let fp = loaded_iface.fingerprint();
                    self.interfaces.insert(target_id.clone(), (Arc::new(loaded_iface), fp));
                    parsed_sources.insert(target_id.clone(), loaded);
                    queue.push_back(target_id);
                }
            }
        }

        // 5. Link affected components in tolerant mode
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
            let interfaces_for_linker: BTreeMap<ModuleId, UnlinkedModuleInterface> = self
                .interfaces
                .iter()
                .filter(|(id, _)| parsed_sources.contains_key(id))
                .map(|(id, (iface, _))| (id.clone(), (**iface).clone()))
                .collect();
            let linker = ModuleLinker::new(Arc::new(self.universe.clone()), interfaces_for_linker);
            let mut modules = BTreeMap::new();
            let mut graphs = crate::graph::ModuleGraphs::default();
            let mut visited_components = BTreeSet::new();

            for component_entry in parsed_sources.keys() {
                if visited_components.contains(component_entry) {
                    continue;
                }
                let component = linker.link_component_tolerant(component_entry.clone(), &self.resolved_imports);
                for err in component.diagnostics {
                    let err_module = err.module().cloned().unwrap_or_else(|| component_entry.clone());
                    let target_iface = self.interfaces.get(&err_module).map(|(iface, _)| &**iface);
                    let diag = ModuleDiagnostic::from_link_error(err, target_iface);
                    diagnostics.entry(err_module).or_default().push(diag);
                }
                blocked_modules.extend(component.blocked_modules);

                for (mod_id, linked_mod) in component.program.modules {
                    visited_components.insert(mod_id.clone());
                    let linked_fp = linked_mod.interface.fingerprint();
                    self.linked_modules.insert(mod_id.clone(), (linked_mod.clone(), linked_fp));
                    modules.insert(mod_id, linked_mod);
                }
                graphs.merge_from(&component.program.graphs);
            }
            let initialization_order = graphs.runtime.initialization_order().unwrap_or_default();
            Arc::new(LinkedProgram {
                universe: Arc::new(self.universe.clone()),
                modules,
                graphs,
                entry,
                initialization_order,
            })
        };

        // 6. Update state for any newly discovered transitive modules
        for (module, parsed) in &parsed_sources {
            if self.sources_by_module.contains_key(module) {
                continue;
            }
            if let Some(location) = parsed.source.clone() {
                self.insert_state(
                    module.clone(),
                    parsed.kind,
                    location,
                    SourceRevision::default(),
                    parsed.clone(),
                    false,
                );
            }
        }

        self.linked = Some(linked.clone());
        self.diagnostics = diagnostics.clone();
        self.blocked_modules = blocked_modules.clone();
        let interfaces = self
            .interfaces
            .iter()
            .map(|(id, (iface, _))| (id.clone(), iface.clone()))
            .collect();
        Ok(WorkspaceModuleUpdate {
            linked,
            sources: parsed_sources,
            interfaces,
            diagnostics,
            blocked_modules,
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
