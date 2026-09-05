//! Persistent compiler-owned module workspace lifecycle.

use crate::diagnostic::ModuleDiagnostic;
use crate::error::{InterfaceError, ModuleLoadError, ModuleResolutionError, ProjectError, SourceError};
use crate::identity::{
    ImportSiteId, ModuleId, ModulePath, ProjectSourceIdentity, SourceId, SourceLocation, SyntheticProjectId, SyntheticProjectIdAllocator,
};
use crate::interface::{InterfaceBuilder, UnlinkedModuleInterface};
use crate::linker::{LinkError, LinkedProgram, ModuleLinker};
use crate::manifest::DependencyProvider;
use crate::project::ProjectUniverse;
use crate::resolver::ModuleResolver;
use crate::source::{
    classify_entry_ownership, EntryOwnership, FilesystemSourceProvider, ModuleKind, OverlaySourceProvider, ParsedModuleUnit, SourceOverlay, SourceProvider,
};
use crate::stabilization::ResolverGeneration;
use crate::topology::{ModuleTopology, TopologyDelta};
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

/// Deterministic identifier for a connected linked component.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ComponentId(pub ModuleId);

/// Retained linking product for one connected component.
#[derive(Clone, Debug)]
pub struct ComponentLinkedProduct {
    pub component_id: ComponentId,
    pub members: BTreeSet<ModuleId>,
    pub modules: BTreeMap<ModuleId, crate::linker::LinkedModule>,
    pub graphs: crate::graph::ModuleGraphs,
    pub blocked_modules: BTreeSet<ModuleId>,
    pub diagnostics: Vec<ModuleDiagnostic>,
    pub public_fingerprints: BTreeMap<ModuleId, crate::fingerprint::LinkedInterfaceFingerprint>,
    pub private_fingerprints: BTreeMap<ModuleId, crate::fingerprint::LinkedDependencyFingerprint>,
}

/// Module-owned deterministic work counts for one workspace update.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct WorkspaceModuleStats {
    pub interfaces_built: usize,
    pub interfaces_reused: usize,
    pub import_sites_considered: usize,
    pub import_sites_validated: usize,
    pub import_sites_reused: usize,
    pub imports_resolved: usize,
    pub import_resolutions_reused: usize,
    pub negative_resolutions_reused: usize,
    pub linked_modules_recomputed: usize,
    pub linked_modules_reused: usize,
    pub linked_components: usize,
    pub linked_components_recomputed: usize,
    pub linked_components_reused: usize,
    pub ownership_lookups: usize,
    pub ownership_cache_hits: usize,
    pub filesystem_resolution_hits: usize,
    pub filesystem_resolution_misses: usize,
    pub topology_invalidations: usize,
    pub changed_sources: usize,
    pub affected_modules: usize,
    pub identity_changes: usize,
    pub purged_products: usize,
}

/// Products published after one source mutation.
#[derive(Clone, Debug)]
pub struct WorkspaceModuleUpdate {
    pub linked: Arc<LinkedProgram>,
    pub sources: BTreeMap<ModuleId, Arc<ParsedModuleUnit>>,
    pub interfaces: BTreeMap<ModuleId, Arc<UnlinkedModuleInterface>>,
    pub import_products: BTreeMap<ImportSiteId, Arc<crate::resolver::ImportResolutionProduct>>,
    pub diagnostics: BTreeMap<ModuleId, Vec<ModuleDiagnostic>>,
    pub blocked_modules: BTreeSet<ModuleId>,
    pub changed_modules: BTreeSet<ModuleId>,
    pub removed_modules: BTreeSet<ModuleId>,
    pub identity_changes: BTreeSet<ModuleId>,
    pub stats: WorkspaceModuleStats,
    pub topology: Arc<ModuleTopology>,
    pub reverse_importers: Arc<BTreeMap<ModuleId, BTreeSet<ModuleId>>>,
    pub sites_by_importer: Arc<BTreeMap<ModuleId, BTreeSet<ImportSiteId>>>,
    pub reverse_site_importers: Arc<BTreeMap<ModuleId, BTreeSet<ImportSiteId>>>,
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

/// A staged overlay wrapper over any SourceProvider that intercepts overlay reads
/// without mutating the underlying provider before commit.
#[derive(Debug)]
pub struct StagedOverlayProvider<'a, P: SourceProvider> {
    pub base: &'a OverlaySourceProvider<P>,
    pub staged_overlays: &'a BTreeMap<ModuleId, SourceOverlay>,
    pub removed_overlays: &'a BTreeSet<ModuleId>,
    pub staged_by_source: &'a BTreeMap<SourceId, ModuleId>,
    pub removed_by_source: &'a BTreeSet<SourceId>,
}

impl<'a, P: SourceProvider> SourceProvider for StagedOverlayProvider<'a, P> {
    fn locate(&self, project: &crate::project::ResolvedProject, path: &crate::identity::ModulePath) -> Result<crate::source::SourceUnit, ModuleResolutionError> {
        let candidate_id = ModuleId::resolved(project.id, path.clone());
        if !self.removed_overlays.contains(&candidate_id) {
            if let Some(staged) = self.staged_overlays.get(&candidate_id) {
                return Ok(crate::source::SourceUnit {
                    id: staged.id.clone(),
                    kind: staged.kind,
                    source: staged.source.clone(),
                });
            }
        }
        let default_unit = self.base.locate(project, path)?;
        if !self.removed_overlays.contains(&default_unit.id) {
            if let Some(staged) = self.staged_overlays.get(&default_unit.id) {
                return Ok(crate::source::SourceUnit {
                    id: staged.id.clone(),
                    kind: staged.kind,
                    source: staged.source.clone(),
                });
            }
        }
        Ok(default_unit)
    }

    fn read(&self, source: &SourceId) -> Result<Arc<str>, SourceError> {
        if !self.removed_by_source.contains(source) {
            if let Some(module) = self.staged_by_source.get(source) {
                if let Some(staged) = self.staged_overlays.get(module) {
                    return Ok(staged.text.clone());
                }
            }
        }
        if self.removed_by_source.contains(source) {
            return self.base.base().read(source);
        }
        self.base.read(source)
    }
}

pub struct RebuildOutput {
    pub linked: Arc<LinkedProgram>,
    pub parsed_sources: BTreeMap<ModuleId, Arc<ParsedModuleUnit>>,
    pub new_discovered_sources: Vec<WorkspaceSourceState>,
    pub interfaces: BTreeMap<ModuleId, (Arc<UnlinkedModuleInterface>, crate::fingerprint::InterfaceFingerprint)>,
    pub import_products: BTreeMap<ImportSiteId, Arc<crate::resolver::ImportResolutionProduct>>,
    pub resolved_imports: BTreeMap<(ModuleId, String), ModuleId>,
    pub reverse_importers: Arc<BTreeMap<ModuleId, BTreeSet<ModuleId>>>,
    pub sites_by_importer: Arc<BTreeMap<ModuleId, BTreeSet<ImportSiteId>>>,
    pub reverse_site_importers: Arc<BTreeMap<ModuleId, BTreeSet<ImportSiteId>>>,
    pub linked_modules: BTreeMap<ModuleId, (crate::linker::LinkedModule, crate::fingerprint::LinkedInterfaceFingerprint)>,
    pub linked_dependency_fingerprints: BTreeMap<ModuleId, crate::fingerprint::LinkedDependencyFingerprint>,
    pub retained_components: BTreeMap<ComponentId, Arc<ComponentLinkedProduct>>,
    pub module_components: BTreeMap<ModuleId, ComponentId>,
    pub diagnostics: BTreeMap<ModuleId, Vec<ModuleDiagnostic>>,
    pub blocked_modules: BTreeSet<ModuleId>,
    pub stats: WorkspaceModuleStats,
    pub topology: Arc<ModuleTopology>,
}

/// Helper function to atomically reconcile forward and reverse dependencies for an importer.
fn reconcile_site_dependencies(
    importer: &ModuleId,
    new_site_products: &BTreeMap<ImportSiteId, Arc<crate::resolver::ImportResolutionProduct>>,
    import_products: &mut BTreeMap<ImportSiteId, Arc<crate::resolver::ImportResolutionProduct>>,
    sites_by_importer: &mut BTreeMap<ModuleId, BTreeSet<ImportSiteId>>,
    reverse_site_importers: &mut BTreeMap<ModuleId, BTreeSet<ImportSiteId>>,
    resolved_imports: &mut BTreeMap<(ModuleId, String), ModuleId>,
    reverse_importers: &mut BTreeMap<ModuleId, BTreeSet<ModuleId>>,
) {
    if let Some(old_sites) = sites_by_importer.remove(importer) {
        for site in &old_sites {
            if let Some(prod) = import_products.remove(site) {
                if let Ok(target) = &prod.target {
                    if let Some(site_set) = reverse_site_importers.get_mut(target) {
                        site_set.remove(site);
                    }
                    if let Some(importers) = reverse_importers.get_mut(target) {
                        if !reverse_site_importers.get(target).is_some_and(|s| s.iter().any(|st| &st.importer == importer)) {
                            importers.remove(importer);
                        }
                    }
                }
            }
        }
    }

    let keys_to_remove: Vec<_> = resolved_imports
        .range((importer.clone(), String::new())..=(importer.clone(), "\u{10ffff}".to_string()))
        .map(|(k, _)| k.clone())
        .collect();
    for key in keys_to_remove {
        resolved_imports.remove(&key);
    }

    let mut current_importer_sites = BTreeSet::new();
    for (site, product) in new_site_products {
        import_products.insert(site.clone(), product.clone());
        current_importer_sites.insert(site.clone());
        if let Ok(target) = &product.target {
            reverse_site_importers.entry(target.clone()).or_default().insert(site.clone());
            reverse_importers.entry(target.clone()).or_default().insert(importer.clone());
            resolved_imports.insert((importer.clone(), product.written_path.written.clone()), target.clone());
        }
    }

    if !current_importer_sites.is_empty() {
        sites_by_importer.insert(importer.clone(), current_importer_sites);
    }
}

/// Validates cross-index consistency between modules_by_source and sources_by_module.
fn validate_cross_index_consistency(
    modules_by_source: &BTreeMap<SourceId, ModuleId>,
    sources_by_module: &BTreeMap<ModuleId, WorkspaceSourceState>,
) -> Result<(), WorkspaceModuleSessionError> {
    for (source_id, module_id) in modules_by_source {
        let Some(source_state) = sources_by_module.get(module_id) else {
            return Err(WorkspaceModuleSessionError::UnknownSource(source_id.clone()));
        };
        if &source_state.location.source_id != source_id {
            return Err(WorkspaceModuleSessionError::UnknownSource(source_id.clone()));
        }
    }
    for (module_id, source_state) in sources_by_module {
        let Some(mapped_mod) = modules_by_source.get(&source_state.location.source_id) else {
            return Err(WorkspaceModuleSessionError::UnknownSource(source_state.location.source_id.clone()));
        };
        if mapped_mod != module_id {
            return Err(WorkspaceModuleSessionError::UnknownSource(source_state.location.source_id.clone()));
        }
    }
    Ok(())
}

/// Partitions interfaces and import products into connected components based on import/re-export/package relationships.
fn compute_connected_components(
    interfaces: &BTreeMap<ModuleId, (Arc<UnlinkedModuleInterface>, crate::fingerprint::InterfaceFingerprint)>,
    import_products: &BTreeMap<ImportSiteId, Arc<crate::resolver::ImportResolutionProduct>>,
) -> (BTreeMap<ComponentId, BTreeSet<ModuleId>>, BTreeMap<ModuleId, ComponentId>) {
    let mut adjacency: BTreeMap<ModuleId, BTreeSet<ModuleId>> = BTreeMap::new();
    for module in interfaces.keys() {
        adjacency.entry(module.clone()).or_default();
    }

    for (site, product) in import_products {
        let importer = &site.importer;
        if !interfaces.contains_key(importer) {
            continue;
        }
        if let Ok(target) = &product.target {
            if interfaces.contains_key(target) {
                adjacency.entry(importer.clone()).or_default().insert(target.clone());
                adjacency.entry(target.clone()).or_default().insert(importer.clone());
            }
        }
    }

    for (module, (iface, _)) in interfaces {
        if let Some(project_id) = module.project.as_resolved() {
            let mut curr_path = module.path.parent();
            while let Some(parent) = curr_path {
                let pkg_id = ModuleId::resolved(project_id, parent.clone());
                if interfaces.contains_key(&pkg_id) {
                    adjacency.entry(module.clone()).or_default().insert(pkg_id.clone());
                    adjacency.entry(pkg_id.clone()).or_default().insert(module.clone());
                }
                curr_path = parent.parent();
            }
            let root_id = ModuleId::resolved(project_id, ModulePath::root());
            if interfaces.contains_key(&root_id) {
                adjacency.entry(module.clone()).or_default().insert(root_id.clone());
                adjacency.entry(root_id.clone()).or_default().insert(module.clone());
            }
        }
        for export in iface.exports.values() {
            if let crate::interface::UnlinkedExportTarget::CanonicalDeclaration { module: target, .. } = &export.target {
                if interfaces.contains_key(target) {
                    adjacency.entry(module.clone()).or_default().insert(target.clone());
                    adjacency.entry(target.clone()).or_default().insert(module.clone());
                }
            }
        }
    }

    let mut visited = BTreeSet::new();
    let mut component_members = BTreeMap::new();
    let mut module_to_component = BTreeMap::new();

    for module in interfaces.keys() {
        if visited.contains(module) {
            continue;
        }
        let mut members = BTreeSet::new();
        let mut queue = vec![module.clone()];
        visited.insert(module.clone());

        while let Some(curr) = queue.pop() {
            members.insert(curr.clone());
            if let Some(neighbors) = adjacency.get(&curr) {
                for neighbor in neighbors {
                    if visited.insert(neighbor.clone()) {
                        queue.push(neighbor.clone());
                    }
                }
            }
        }

        let comp_id = ComponentId(members.first().expect("non-empty component").clone());
        for member in &members {
            module_to_component.insert(member.clone(), comp_id.clone());
        }
        component_members.insert(comp_id, members);
    }

    (component_members, module_to_component)
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
    import_products: BTreeMap<ImportSiteId, Arc<crate::resolver::ImportResolutionProduct>>,
    reverse_importers: Arc<BTreeMap<ModuleId, BTreeSet<ModuleId>>>,
    sites_by_importer: Arc<BTreeMap<ModuleId, BTreeSet<ImportSiteId>>>,
    reverse_site_importers: Arc<BTreeMap<ModuleId, BTreeSet<ImportSiteId>>>,
    linked_modules: BTreeMap<ModuleId, (crate::linker::LinkedModule, crate::fingerprint::LinkedInterfaceFingerprint)>,
    linked_dependency_fingerprints: BTreeMap<ModuleId, crate::fingerprint::LinkedDependencyFingerprint>,
    retained_components: BTreeMap<ComponentId, Arc<ComponentLinkedProduct>>,
    module_components: BTreeMap<ModuleId, ComponentId>,
    linked: Option<Arc<LinkedProgram>>,
    resolved_imports: BTreeMap<(ModuleId, String), ModuleId>,
    diagnostics: BTreeMap<ModuleId, Vec<ModuleDiagnostic>>,
    blocked_modules: BTreeSet<ModuleId>,
    generation: u64,
    topology: Arc<ModuleTopology>,
    #[doc(hidden)]
    pub late_failure_injected: bool,
}

impl Default for WorkspaceModuleSession {
    fn default() -> Self {
        Self::new()
    }
}

impl WorkspaceModuleSession {
    pub fn new() -> Self {
        let universe = ProjectUniverse::new();
        let topology = Arc::new(ModuleTopology::from_parts(
            ResolverGeneration(0),
            &universe,
            &BTreeMap::new(),
            &BTreeMap::new(),
        ));
        Self {
            universe,
            provider: OverlaySourceProvider::new(FilesystemSourceProvider::new()),
            project_roots: BTreeMap::new(),
            modules_by_source: BTreeMap::new(),
            sources_by_module: BTreeMap::new(),
            standalone_projects: BTreeMap::new(),
            synthetic_ids: SyntheticProjectIdAllocator,
            interfaces: BTreeMap::new(),
            import_products: BTreeMap::new(),
            reverse_importers: Arc::new(BTreeMap::new()),
            sites_by_importer: Arc::new(BTreeMap::new()),
            reverse_site_importers: Arc::new(BTreeMap::new()),
            linked_modules: BTreeMap::new(),
            linked_dependency_fingerprints: BTreeMap::new(),
            retained_components: BTreeMap::new(),
            module_components: BTreeMap::new(),
            linked: None,
            resolved_imports: BTreeMap::new(),
            diagnostics: BTreeMap::new(),
            blocked_modules: BTreeSet::new(),
            generation: 0,
            topology,
            late_failure_injected: false,
        }
    }

    #[doc(hidden)]
    pub fn inject_late_rebuild_failure(&mut self, inject: bool) {
        self.late_failure_injected = inject;
    }

    pub fn universe(&self) -> &ProjectUniverse {
        &self.universe
    }

    pub fn provider(&self) -> &OverlaySourceProvider<FilesystemSourceProvider> {
        &self.provider
    }

    pub fn resolution_metrics(&self) -> (u64, u64) {
        self.provider.base().resolution_metrics()
    }

    pub fn generation(&self) -> u64 {
        self.generation
    }

    pub fn topology(&self) -> &Arc<ModuleTopology> {
        &self.topology
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

    pub fn import_products(&self) -> &BTreeMap<ImportSiteId, Arc<crate::resolver::ImportResolutionProduct>> {
        &self.import_products
    }

    pub fn import_product(&self, site: &ImportSiteId) -> Option<&Arc<crate::resolver::ImportResolutionProduct>> {
        self.import_products.get(site)
    }

    pub fn reverse_importers(&self) -> &BTreeMap<ModuleId, BTreeSet<ModuleId>> {
        &self.reverse_importers
    }

    pub fn reverse_importers_arc(&self) -> &Arc<BTreeMap<ModuleId, BTreeSet<ModuleId>>> {
        &self.reverse_importers
    }

    pub fn sites_by_importer(&self) -> &BTreeMap<ModuleId, BTreeSet<ImportSiteId>> {
        &self.sites_by_importer
    }

    pub fn sites_by_importer_arc(&self) -> &Arc<BTreeMap<ModuleId, BTreeSet<ImportSiteId>>> {
        &self.sites_by_importer
    }

    pub fn reverse_site_importers(&self) -> &BTreeMap<ModuleId, BTreeSet<ImportSiteId>> {
        &self.reverse_site_importers
    }

    pub fn reverse_site_importers_arc(&self) -> &Arc<BTreeMap<ModuleId, BTreeSet<ImportSiteId>>> {
        &self.reverse_site_importers
    }

    pub fn linked_modules(&self) -> &BTreeMap<ModuleId, (crate::linker::LinkedModule, crate::fingerprint::LinkedInterfaceFingerprint)> {
        &self.linked_modules
    }

    pub fn linked_dependency_fingerprints(&self) -> &BTreeMap<ModuleId, crate::fingerprint::LinkedDependencyFingerprint> {
        &self.linked_dependency_fingerprints
    }

    pub fn retained_components(&self) -> &BTreeMap<ComponentId, Arc<ComponentLinkedProduct>> {
        &self.retained_components
    }

    pub fn module_components(&self) -> &BTreeMap<ModuleId, ComponentId> {
        &self.module_components
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
        let previous_states = self.sources_by_module.clone();
        let removed_modules = previous_states.keys().cloned().collect::<BTreeSet<_>>();

        let mut target_universe = ProjectUniverse::new();
        let mut target_project_roots = BTreeMap::new();
        for root in roots {
            let canonical = root.canonicalize().unwrap_or_else(|_| root.clone());
            let id = target_universe.load_root_with_provider(canonical.join("project.toml"), dependency_provider)?;
            target_project_roots.insert(ProjectSourceIdentity::from_path(canonical), id);
        }

        let mut target_sources_by_module = BTreeMap::new();
        let mut target_modules_by_source = BTreeMap::new();
        let mut changed = BTreeSet::new();

        for state in previous_states.values() {
            let path = crate::source::canonicalize_path(&state.location.display_path);
            let ownership = classify_entry_ownership(&path, &mut target_universe)?;
            let module = match ownership {
                EntryOwnership::ProjectOwned { project } => {
                    let project_ref = target_universe.get_project(project).expect("loaded project is present");
                    target_project_roots.insert(ProjectSourceIdentity::from_path(&project_ref.root_dir), project);
                    let unit = crate::source::resolve_source_path(project_ref, &path).map_err(WorkspaceModuleSessionError::from)?;
                    unit.id
                }
                EntryOwnership::StandalonePackageOwned { package_root } => {
                    let project_id = target_universe.load_standalone_package(&package_root, None)?;
                    target_project_roots.insert(ProjectSourceIdentity::from_path(&package_root), project_id);
                    let project_ref = target_universe.get_project(project_id).expect("loaded package is present");
                    let unit = crate::source::resolve_source_path(project_ref, &path).map_err(WorkspaceModuleSessionError::from)?;
                    unit.id
                }
                EntryOwnership::StandaloneModule { file: _ } => {
                    let synthetic = *self.standalone_projects.entry(state.location.source_id.clone()).or_insert_with(|| self.synthetic_ids.allocate());
                    ModuleId::synthetic(synthetic, ModulePath::root())
                }
                EntryOwnership::Inline { synthetic } => {
                    ModuleId::synthetic(synthetic, ModulePath::root())
                }
            };

            let parsed = parse_source(module.clone(), state.kind, state.location.clone(), state.text.clone())?;
            let updated = WorkspaceSourceState {
                module: module.clone(),
                kind: state.kind,
                location: state.location.clone(),
                revision: state.revision,
                text: parsed.text.clone(),
                parsed,
                open_overlay: state.open_overlay,
            };
            target_modules_by_source.insert(state.location.source_id.clone(), module.clone());
            target_sources_by_module.insert(module.clone(), updated);
            changed.insert(module);
        }

        validate_cross_index_consistency(&target_modules_by_source, &target_sources_by_module)?;

        let stats = WorkspaceModuleStats {
            topology_invalidations: 1,
            ..WorkspaceModuleStats::default()
        };

        let target_generation = ResolverGeneration(self.generation.saturating_add(1));
        let rebuild_output = self.derive_rebuild(
            &target_universe,
            &self.provider,
            &target_sources_by_module,
            changed.clone(),
            removed_modules.clone(),
            removed_modules.clone(),
            stats,
            target_generation,
        )?;

        // --- COMMIT BARRIER ---
        self.universe = target_universe;
        self.project_roots = target_project_roots;
        self.modules_by_source = target_modules_by_source;
        self.sources_by_module = target_sources_by_module;
        self.provider.base().clear_cache();
        self.interfaces = rebuild_output.interfaces;
        self.import_products = rebuild_output.import_products;
        self.resolved_imports = rebuild_output.resolved_imports;
        self.reverse_importers = rebuild_output.reverse_importers.clone();
        self.sites_by_importer = rebuild_output.sites_by_importer.clone();
        self.reverse_site_importers = rebuild_output.reverse_site_importers.clone();
        self.topology = rebuild_output.topology.clone();
        self.linked_modules = rebuild_output.linked_modules;
        self.linked_dependency_fingerprints = rebuild_output.linked_dependency_fingerprints;
        self.retained_components = rebuild_output.retained_components;
        self.module_components = rebuild_output.module_components;
        self.linked = Some(rebuild_output.linked.clone());
        self.diagnostics = rebuild_output.diagnostics.clone();
        self.blocked_modules = rebuild_output.blocked_modules.clone();
        self.generation = self.generation.saturating_add(1);

        let interfaces = self
            .interfaces
            .iter()
            .map(|(id, (iface, _))| (id.clone(), iface.clone()))
            .collect();

        Ok(WorkspaceModuleUpdate {
            linked: rebuild_output.linked,
            sources: rebuild_output.parsed_sources,
            interfaces,
            import_products: self.import_products.clone(),
            diagnostics: self.diagnostics.clone(),
            blocked_modules: self.blocked_modules.clone(),
            changed_modules: changed,
            removed_modules,
            identity_changes: BTreeSet::new(),
            stats: rebuild_output.stats,
            topology: rebuild_output.topology,
            reverse_importers: rebuild_output.reverse_importers,
            sites_by_importer: rebuild_output.sites_by_importer,
            reverse_site_importers: rebuild_output.reverse_site_importers,
        })
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
        let mut ownership_reclassifications = false;
        let mut purged_identities = BTreeSet::new();
        let mut stats = WorkspaceModuleStats::default();

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
                    ownership_reclassifications |= source.display_path.file_name().is_some_and(|name| name == "package.ph" || name == "project.toml");
                    stats.ownership_lookups += 1;
                    if self.modules_by_source.contains_key(&source.source_id) || mutated_modules_by_source.contains_key(&source.source_id) {
                        stats.ownership_cache_hits += 1;
                    }
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
                    ownership_reclassifications |= source.display_path.file_name().is_some_and(|name| name == "package.ph" || name == "project.toml");
                    stats.ownership_lookups += 1;
                    if self.modules_by_source.contains_key(&source.source_id) || mutated_modules_by_source.contains_key(&source.source_id) {
                        stats.ownership_cache_hits += 1;
                    }
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
                    ownership_reclassifications |= state
                        .location
                        .display_path
                        .file_name()
                        .is_some_and(|name| name == "package.ph" || name == "project.toml");
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
                    ownership_reclassifications |= source.display_path.file_name().is_some_and(|name| name == "package.ph" || name == "project.toml");
                    stats.ownership_lookups += 1;
                    if self.modules_by_source.contains_key(&source.source_id) || mutated_modules_by_source.contains_key(&source.source_id) {
                        stats.ownership_cache_hits += 1;
                    }
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
                    ownership_reclassifications |= self
                        .sources_by_module
                        .get(&module)
                        .and_then(|state| state.location.display_path.file_name())
                        .is_some_and(|name| name == "package.ph" || name == "project.toml");
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
            stats.topology_invalidations += 1;
        }

        // Reclassification if project structure markers changed
        if ownership_reclassifications {
            let (reclassified, reclassified_removed, reclassified_identities) = self.reclassify_tracked_sources(&removed_modules_by_source)?;
            changed_modules.extend(reclassified);
            removed_sources.extend(reclassified_removed);
            identity_changes.extend(reclassified_identities);
        }

        let removed_modules = removed_sources;

        // Build target effective views for validation and derive_rebuild
        let mut target_modules_by_source = self.modules_by_source.clone();
        for id in &removed_modules_by_source {
            target_modules_by_source.remove(id);
        }
        target_modules_by_source.extend(mutated_modules_by_source.clone());

        let mut target_sources_by_module = self.sources_by_module.clone();
        for id in &removed_modules {
            target_sources_by_module.remove(id);
        }
        target_sources_by_module.extend(mutated_sources.clone());

        // Cross-index consistency validation before derivation
        validate_cross_index_consistency(&target_modules_by_source, &target_sources_by_module)?;

        // Prepare staged overlays for derivation
        let mut staged_overlays = BTreeMap::new();
        let mut removed_overlays = BTreeSet::new();
        let mut staged_by_source = BTreeMap::new();
        let mut removed_by_source = BTreeSet::new();

        for op in &overlay_ops {
            match op {
                OverlayOp::Set(overlay) => {
                    staged_by_source.insert(overlay.source.source_id.clone(), overlay.id.clone());
                    removed_by_source.remove(&overlay.source.source_id);
                    staged_overlays.insert(overlay.id.clone(), overlay.clone());
                    removed_overlays.remove(&overlay.id);
                }
                OverlayOp::Remove(module) => {
                    removed_overlays.insert(module.clone());
                    staged_overlays.remove(module);
                    if let Some(src_state) = target_sources_by_module.get(module) {
                        removed_by_source.insert(src_state.location.source_id.clone());
                        staged_by_source.remove(&src_state.location.source_id);
                    }
                }
            }
        }

        let staged_provider = StagedOverlayProvider {
            base: &self.provider,
            staged_overlays: &staged_overlays,
            removed_overlays: &removed_overlays,
            staged_by_source: &staged_by_source,
            removed_by_source: &removed_by_source,
        };

        let effective_universe = universe_override.as_ref().unwrap_or(&self.universe);

        let target_generation = ResolverGeneration(self.generation.saturating_add(1));
        // Derive new state privately without mutating committed state
        let rebuild_output = self.derive_rebuild(
            effective_universe,
            &staged_provider,
            &target_sources_by_module,
            changed_modules.clone(),
            removed_modules.clone(),
            identity_changes.clone(),
            stats.clone(),
            target_generation,
        )?;

        // Test-only late failure injection seam
        if self.late_failure_injected {
            return Err(WorkspaceModuleSessionError::Link(LinkError::MissingModule {
                module: ModuleId::universe_root(),
            }));
        }

        // --- COMMIT BARRIER ---
        // 1. Commit overlays to shared provider
        for op in overlay_ops {
            match op {
                OverlayOp::Set(overlay) => self.provider.set_overlay(overlay),
                OverlayOp::Remove(module) => self.provider.remove_overlay(&module),
            }
        }

        // 2. Commit tracking maps
        self.modules_by_source = target_modules_by_source;
        self.sources_by_module = target_sources_by_module;
        self.project_roots.extend(mutated_project_roots);
        self.standalone_projects.extend(mutated_standalone_projects);
        self.synthetic_ids = synthetic_ids;
        if let Some(u) = universe_override {
            self.universe = u;
        }

        // 3. Commit newly discovered transitive sources
        for discovered in rebuild_output.new_discovered_sources {
            self.modules_by_source.insert(discovered.location.source_id.clone(), discovered.module.clone());
            self.sources_by_module.insert(discovered.module.clone(), discovered);
        }

        // 4. Commit rebuild products
        self.interfaces = rebuild_output.interfaces;
        self.import_products = rebuild_output.import_products;
        self.resolved_imports = rebuild_output.resolved_imports;
        self.reverse_importers = rebuild_output.reverse_importers.clone();
        self.sites_by_importer = rebuild_output.sites_by_importer.clone();
        self.reverse_site_importers = rebuild_output.reverse_site_importers.clone();
        self.topology = rebuild_output.topology.clone();
        self.linked_modules = rebuild_output.linked_modules;
        self.linked_dependency_fingerprints = rebuild_output.linked_dependency_fingerprints;
        self.retained_components = rebuild_output.retained_components;
        self.module_components = rebuild_output.module_components;
        self.linked = Some(rebuild_output.linked.clone());
        self.diagnostics = rebuild_output.diagnostics.clone();
        self.blocked_modules = rebuild_output.blocked_modules.clone();

        // 5. Advance generation once at successful commit
        self.generation = self.generation.saturating_add(1);

        let interfaces = self
            .interfaces
            .iter()
            .map(|(id, (iface, _))| (id.clone(), iface.clone()))
            .collect();

        Ok(WorkspaceModuleUpdate {
            linked: rebuild_output.linked,
            sources: rebuild_output.parsed_sources,
            interfaces,
            import_products: self.import_products.clone(),
            diagnostics: self.diagnostics.clone(),
            blocked_modules: self.blocked_modules.clone(),
            changed_modules,
            removed_modules,
            identity_changes,
            stats: rebuild_output.stats,
            topology: rebuild_output.topology,
            reverse_importers: rebuild_output.reverse_importers,
            sites_by_importer: rebuild_output.sites_by_importer,
            reverse_site_importers: rebuild_output.reverse_site_importers,
        })
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

    fn reclassify_tracked_sources(
        &mut self,
        removed_sources_by_id: &BTreeSet<SourceId>,
    ) -> Result<(BTreeSet<ModuleId>, BTreeSet<ModuleId>, BTreeSet<ModuleId>), WorkspaceModuleSessionError> {
        let previous_sources = std::mem::take(&mut self.sources_by_module);
        let previous_modules: BTreeSet<ModuleId> = previous_sources.keys().cloned().collect();
        let mut mutated_modules = BTreeMap::new();
        let removed_modules = BTreeSet::new();
        let mut project_roots = BTreeMap::new();
        let mut standalone_projects = self.standalone_projects.clone();
        let mut synthetic_ids = self.synthetic_ids.clone();
        let mut universe_override = Some(self.universe.clone());
        let mut sources_by_module = BTreeMap::new();
        let mut modules_by_source = BTreeMap::new();
        let mut changed = BTreeSet::new();
        let mut identities = BTreeSet::new();

        for state in previous_sources.into_values() {
            if removed_sources_by_id.contains(&state.location.source_id) {
                continue;
            }
            let path = crate::source::canonicalize_path(&state.location.display_path);
            let effective_universe = universe_override.as_mut().unwrap();
            let ownership = classify_entry_ownership(&path, effective_universe)?;
            let (module, project_id) = match ownership {
                EntryOwnership::ProjectOwned { project } => {
                    let project_ref = effective_universe.get_project(project).expect("loaded project is present");
                    (
                        crate::source::resolve_source_path(project_ref, &path).map_err(WorkspaceModuleSessionError::from)?.id,
                        Some(project),
                    )
                }
                EntryOwnership::StandalonePackageOwned { package_root } => {
                    let project_id = effective_universe.load_standalone_package(&package_root, None)?;
                    project_roots.insert(ProjectSourceIdentity::from_path(&package_root), project_id);
                    let project_ref = effective_universe.get_project(project_id).expect("loaded package is present");
                    (
                        crate::source::resolve_source_path(project_ref, &path).map_err(WorkspaceModuleSessionError::from)?.id,
                        Some(project_id),
                    )
                }
                EntryOwnership::StandaloneModule { file: _ } => {
                    let synthetic = *standalone_projects
                        .entry(state.location.source_id.clone())
                        .or_insert_with(|| synthetic_ids.allocate());
                    (ModuleId::synthetic(synthetic, ModulePath::root()), None)
                }
                EntryOwnership::Inline { synthetic } => {
                    (ModuleId::synthetic(synthetic, ModulePath::root()), None)
                }
            };

            if let Some(project_id) = project_id {
                let project_ref = effective_universe.get_project(project_id).expect("loaded project is present");
                project_roots.insert(ProjectSourceIdentity::from_path(&project_ref.root_dir), project_id);
            }

            mutated_modules.insert(state.location.source_id.clone(), module.clone());
            if module != state.module {
                identities.insert(state.module.clone());
                changed.insert(module.clone());
            }

            let kind = Self::kind_for_source_delta(
                &module,
                &state.location,
                &sources_by_module,
                &removed_modules,
                &self.sources_by_module,
            );
            let updated = WorkspaceSourceState {
                module: module.clone(),
                kind,
                location: state.location.clone(),
                revision: state.revision,
                text: state.text.clone(),
                parsed: state.parsed.clone(),
                open_overlay: state.open_overlay,
            };
            modules_by_source.insert(state.location.source_id.clone(), module.clone());
            sources_by_module.insert(module, updated);
        }

        self.modules_by_source = modules_by_source;
        self.sources_by_module = sources_by_module;
        self.project_roots = project_roots;
        self.standalone_projects = standalone_projects;
        self.synthetic_ids = synthetic_ids;
        if let Some(universe) = universe_override {
            self.universe = universe;
        }

        let mut removed = BTreeSet::new();
        for old_module in previous_modules {
            if !self.sources_by_module.contains_key(&old_module) {
                removed.insert(old_module);
            }
        }
        Ok((changed, removed, identities))
    }

    #[allow(clippy::too_many_arguments)]
    fn derive_rebuild<P: SourceProvider>(
        &self,
        universe: &ProjectUniverse,
        provider: &P,
        sources_by_module: &BTreeMap<ModuleId, WorkspaceSourceState>,
        changed_modules: BTreeSet<ModuleId>,
        removed_modules: BTreeSet<ModuleId>,
        identity_changes: BTreeSet<ModuleId>,
        mut stats: WorkspaceModuleStats,
        target_generation: ResolverGeneration,
    ) -> Result<RebuildOutput, WorkspaceModuleSessionError> {
        stats.changed_sources = changed_modules.len();
        stats.identity_changes = identity_changes.len();
        let (resolution_hits_before, resolution_misses_before) = self.provider.base().resolution_metrics();

        let mut interfaces = self.interfaces.clone();
        let mut linked_modules = self.linked_modules.clone();
        let mut import_products = self.import_products.clone();
        let mut resolved_imports = self.resolved_imports.clone();
        let mut reverse_importers = (*self.reverse_importers).clone();
        let mut sites_by_importer = (*self.sites_by_importer).clone();
        let mut reverse_site_importers = (*self.reverse_site_importers).clone();
        let mut diagnostics: BTreeMap<ModuleId, Vec<ModuleDiagnostic>> = BTreeMap::new();
        let mut blocked_modules: BTreeSet<ModuleId> = BTreeSet::new();

        // 1. Invalidate removed modules from products
        for removed in &removed_modules {
            if interfaces.remove(removed).is_some() {
                stats.purged_products += 1;
            }
            if linked_modules.remove(removed).is_some() {
                stats.purged_products += 1;
            }
            if let Some(sites) = sites_by_importer.remove(removed) {
                for site in &sites {
                    if let Some(prod) = import_products.remove(site) {
                        stats.purged_products += 1;
                        if let Ok(target) = &prod.target {
                            if let Some(rev) = reverse_site_importers.get_mut(target) {
                                rev.remove(site);
                            }
                        }
                    }
                }
            }
            if let Some(rev_sites) = reverse_site_importers.remove(removed) {
                for site in &rev_sites {
                    import_products.remove(site);
                }
            }
            let resolved_imports_before = resolved_imports.len();
            resolved_imports.retain(|(importer, _), _| importer != removed);
            stats.purged_products += resolved_imports_before.saturating_sub(resolved_imports.len());
            if reverse_importers.remove(removed).is_some() {
                stats.purged_products += 1;
            }
            for importers in reverse_importers.values_mut() {
                importers.remove(removed);
            }
        }

        // 2. Build or check unlinked interfaces for changed/new modules
        let mut modules_with_changed_interface = BTreeSet::new();
        let mut changed_exposures = BTreeSet::new();
        for (module, state) in sources_by_module {
            if !changed_modules.contains(module) && interfaces.contains_key(module) {
                stats.interfaces_reused += 1;
                continue;
            }
            stats.interfaces_built += 1;
            match InterfaceBuilder::build(module.clone(), state.kind, &state.parsed.program) {
                Ok(interface) => {
                    let new_fp = interface.fingerprint();
                    let old_fp = interfaces.get(module).map(|(_, fp)| *fp);
                    let fp_changed = old_fp != Some(new_fp);
                    if fp_changed {
                        let old_exposures = interfaces.get(module).map(|(iface, _)| &iface.exposed_children);
                        if old_exposures != Some(&interface.exposed_children) {
                            changed_exposures.insert(module.clone());
                        }
                        modules_with_changed_interface.insert(module.clone());
                    }
                    interfaces.insert(module.clone(), (Arc::new(interface), new_fp));
                }
                Err(err) => {
                    if let Some((old_iface, _)) = interfaces.remove(module) {
                        stats.purged_products += 1;
                        if !old_iface.exposed_children.is_empty() {
                            changed_exposures.insert(module.clone());
                        }
                    }
                    modules_with_changed_interface.insert(module.clone());
                    let diag = ModuleDiagnostic::from_interface_error(module.clone(), err);
                    diagnostics.entry(module.clone()).or_default().push(diag);
                    blocked_modules.insert(module.clone());
                }
            }
        }

        let mut added_modules = BTreeSet::new();
        for id in sources_by_module.keys() {
            if !self.sources_by_module.contains_key(id) {
                added_modules.insert(id.clone());
            }
        }
        let delta = TopologyDelta {
            added_modules: added_modules.clone(),
            removed_modules: removed_modules.clone(),
            changed_interfaces: modules_with_changed_interface.clone(),
            changed_exposures,
            project_roots_changed: false,
        };

        // 3. Body-only edit short-circuit:
        if modules_with_changed_interface.is_empty()
            && removed_modules.is_empty()
            && identity_changes.is_empty()
            && self.linked.is_some()
        {
            stats.linked_modules_reused = linked_modules.len();
            stats.linked_components = 0;
            stats.linked_components_reused = self.retained_components.len();
            stats.linked_components_recomputed = 0;
            let (resolution_hits_after, resolution_misses_after) = self.provider.base().resolution_metrics();
            stats.filesystem_resolution_hits = resolution_hits_after.saturating_sub(resolution_hits_before) as usize;
            stats.filesystem_resolution_misses = resolution_misses_after.saturating_sub(resolution_misses_before) as usize;
            stats.affected_modules = changed_modules.len() + removed_modules.len();
            let total_sites = import_products.len();
            stats.import_sites_considered = total_sites;
            stats.import_sites_validated = total_sites;
            stats.import_sites_reused = total_sites;
            stats.import_resolutions_reused = total_sites;
            stats.negative_resolutions_reused = import_products.values().filter(|p| p.target.is_err()).count();
            let parsed_sources = sources_by_module
                .iter()
                .map(|(id, state)| (id.clone(), state.parsed.clone()))
                .collect();
            let mut topology = (*self.topology).clone();
            topology.generation = target_generation;
            let topology = Arc::new(topology);
            return Ok(RebuildOutput {
                linked: self.linked.clone().unwrap(),
                parsed_sources,
                new_discovered_sources: Vec::new(),
                interfaces,
                import_products,
                resolved_imports,
                reverse_importers: Arc::new(reverse_importers),
                sites_by_importer: self.sites_by_importer.clone(),
                reverse_site_importers: self.reverse_site_importers.clone(),
                linked_modules,
                linked_dependency_fingerprints: self.linked_dependency_fingerprints.clone(),
                retained_components: self.retained_components.clone(),
                module_components: self.module_components.clone(),
                diagnostics: self.diagnostics.clone(),
                blocked_modules: self.blocked_modules.clone(),
                stats,
                topology,
            });
        }

        // 4. Validate-before-resolve import loop
        let mut parsed_sources = sources_by_module
            .iter()
            .map(|(id, state)| (id.clone(), state.parsed.clone()))
            .collect::<BTreeMap<_, _>>();

        let mut resolver = ModuleResolver::new(universe, provider);
        let mut queue = VecDeque::from_iter(sources_by_module.keys().cloned());
        let mut new_discovered_sources = Vec::new();
        let mut visited_modules = BTreeSet::new();
        let mut recomputed_importers = BTreeSet::new();

        while let Some(module) = queue.pop_front() {
            if !visited_modules.insert(module.clone()) {
                continue;
            }
            let Some((interface, _)) = interfaces.get(&module).cloned() else {
                continue;
            };

            let mut module_new_site_products = BTreeMap::new();

            for (site, import_surface) in interface.import_sites() {
                stats.import_sites_considered += 1;
                let path_syntax = import_surface.path();
                let path_str = path_syntax.to_string();
                let import_range = import_surface.range();

                let product = if let Some(existing) = import_products.get(&site) {
                    stats.import_sites_validated += 1;
                    if existing.written_path.written == path_str && !delta.resolution_product_may_have_changed(existing) {
                        stats.import_sites_reused += 1;
                        stats.import_resolutions_reused += 1;
                        if existing.target.is_err() {
                            stats.negative_resolutions_reused += 1;
                        }
                        existing.clone()
                    } else {
                        recomputed_importers.insert(module.clone());
                        stats.imports_resolved += 1;
                        Arc::new(resolver.resolve_import_product_for_site(site.clone(), path_syntax))
                    }
                } else {
                    recomputed_importers.insert(module.clone());
                    stats.imports_resolved += 1;
                    Arc::new(resolver.resolve_import_product_for_site(site.clone(), path_syntax))
                };

                let target_res = product.target.clone();
                module_new_site_products.insert(site.clone(), product);

                let target_id = match target_res {
                    Ok(target) => target,
                    Err(error) => {
                        let diag = ModuleDiagnostic::from_resolution_error(module.clone(), error, import_range);
                        diagnostics.entry(module.clone()).or_default().push(diag);
                        blocked_modules.insert(module.clone());
                        continue;
                    }
                };

                // Transitive discovery
                if !parsed_sources.contains_key(&target_id) {
                    match resolver.load_parsed(&target_id) {
                        Ok(loaded) => {
                            match InterfaceBuilder::build(target_id.clone(), loaded.kind, &loaded.program) {
                                Ok(loaded_iface) => {
                                    let fp = loaded_iface.fingerprint();
                                    interfaces.insert(target_id.clone(), (Arc::new(loaded_iface), fp));
                                    parsed_sources.insert(target_id.clone(), loaded.clone());
                                    if !sources_by_module.contains_key(&target_id) {
                                        if let Some(loc) = loaded.source.clone() {
                                            new_discovered_sources.push(WorkspaceSourceState {
                                                module: target_id.clone(),
                                                kind: loaded.kind,
                                                location: loc,
                                                revision: SourceRevision::default(),
                                                text: loaded.text.clone(),
                                                parsed: loaded.clone(),
                                                open_overlay: false,
                                            });
                                        }
                                    }
                                    queue.push_back(target_id);
                                }
                                Err(iface_err) => {
                                    let diag = ModuleDiagnostic::from_interface_error(target_id.clone(), iface_err);
                                    diagnostics.entry(target_id.clone()).or_default().push(diag);
                                    blocked_modules.insert(target_id);
                                }
                            }
                        }
                        Err(err) => match err {
                            ModuleLoadError::Parse { module: m, error: parse_err, .. } => {
                                let diag = ModuleDiagnostic::from_syntax_error(m.clone(), parse_err);
                                diagnostics.entry(m.clone()).or_default().push(diag);
                                blocked_modules.insert(m);
                            }
                            ModuleLoadError::Interface { module: m, error: iface_err } => {
                                let diag = ModuleDiagnostic::from_interface_error(m.clone(), iface_err);
                                diagnostics.entry(m.clone()).or_default().push(diag);
                                blocked_modules.insert(m);
                            }
                            ModuleLoadError::Resolution(res_err) => {
                                let diag = ModuleDiagnostic::from_resolution_error(target_id.clone(), res_err, import_range);
                                diagnostics.entry(target_id.clone()).or_default().push(diag);
                                blocked_modules.insert(target_id);
                            }
                        },
                    }
                }
            }

            reconcile_site_dependencies(
                &module,
                &module_new_site_products,
                &mut import_products,
                &mut sites_by_importer,
                &mut reverse_site_importers,
                &mut resolved_imports,
                &mut reverse_importers,
            );
        }

        let mut affected_modules_set = modules_with_changed_interface.clone();
        affected_modules_set.extend(removed_modules.clone());
        affected_modules_set.extend(recomputed_importers.iter().cloned());
        stats.affected_modules = affected_modules_set.len();

        // 5. Link affected components in tolerant mode
        let (new_components, new_module_components) = compute_connected_components(&interfaces, &import_products);

        let mut affected_components = BTreeSet::new();
        for m in modules_with_changed_interface
            .iter()
            .chain(&added_modules)
            .chain(&identity_changes)
            .chain(&recomputed_importers)
            .chain(&delta.changed_exposures)
        {
            if let Some(comp_id) = new_module_components.get(m) {
                affected_components.insert(comp_id.clone());
            }
        }

        for removed in &removed_modules {
            if let Some(old_comp_id) = self.module_components.get(removed) {
                affected_components.insert(old_comp_id.clone());
            }
        }

        for (comp_id, members) in &new_components {
            if let Some(retained) = self.retained_components.get(comp_id) {
                if &retained.members != members {
                    affected_components.insert(comp_id.clone());
                }
            } else {
                affected_components.insert(comp_id.clone());
            }
        }

        let (linked, new_retained_components, linked_dependency_fingerprints) = if parsed_sources.is_empty() {
            (
                Arc::new(LinkedProgram {
                    universe: Arc::new(universe.clone()),
                    modules: BTreeMap::new(),
                    graphs: crate::graph::ModuleGraphs::default(),
                    entry: ModuleId::universe_root(),
                    initialization_order: Vec::new(),
                }),
                BTreeMap::new(),
                BTreeMap::new(),
            )
        } else {
            let mut new_retained_components = BTreeMap::new();
            let mut modules = BTreeMap::new();
            let mut graphs = crate::graph::ModuleGraphs::default();
            let mut linked_dep_fps = BTreeMap::new();

            for (comp_id, members) in &new_components {
                let comp_product = if affected_components.contains(comp_id) {
                    stats.linked_components_recomputed += 1;
                    let entry = comp_id.0.clone();
                    let comp_interfaces: BTreeMap<ModuleId, UnlinkedModuleInterface> = members
                        .iter()
                        .filter_map(|id| interfaces.get(id).map(|(iface, _)| (id.clone(), (**iface).clone())))
                        .collect();
                    let comp_linker = ModuleLinker::new(Arc::new(universe.clone()), comp_interfaces);
                    let component = comp_linker.link_component_interfaces_tolerant(entry.clone(), &resolved_imports);

                    let mut comp_modules = BTreeMap::new();
                    let mut comp_public_fps = BTreeMap::new();
                    let mut comp_private_fps = BTreeMap::new();
                    let mut comp_diags = Vec::new();

                    for err in component.diagnostics {
                        let err_module = err.module().cloned().unwrap_or_else(|| entry.clone());
                        let target_iface = interfaces.get(&err_module).map(|(iface, _)| &**iface);
                        let diag = ModuleDiagnostic::from_link_error(err, target_iface);
                        comp_diags.push(diag);
                    }

                    for (mod_id, linked_mod) in component.program.modules {
                        stats.linked_modules_recomputed += 1;
                        let public_fp = linked_mod.interface.fingerprint();
                        let private_fp = crate::fingerprint::linked_dependency_fingerprint(&linked_mod);
                        comp_public_fps.insert(mod_id.clone(), public_fp);
                        comp_private_fps.insert(mod_id.clone(), private_fp);
                        comp_modules.insert(mod_id, linked_mod);
                    }

                    Arc::new(ComponentLinkedProduct {
                        component_id: comp_id.clone(),
                        members: members.clone(),
                        modules: comp_modules,
                        graphs: component.program.graphs,
                        blocked_modules: component.blocked_modules,
                        diagnostics: comp_diags,
                        public_fingerprints: comp_public_fps,
                        private_fingerprints: comp_private_fps,
                    })
                } else {
                    stats.linked_components_reused += 1;
                    let retained = self
                        .retained_components
                        .get(comp_id)
                        .expect("unaffected component must be present in retained_components")
                        .clone();
                    stats.linked_modules_reused += retained.modules.len();
                    retained
                };

                for (mod_id, linked_mod) in &comp_product.modules {
                    let public_fp = comp_product.public_fingerprints.get(mod_id).copied().unwrap_or_else(|| linked_mod.interface.fingerprint());
                    let private_fp = comp_product.private_fingerprints.get(mod_id).copied().unwrap_or_else(|| crate::fingerprint::linked_dependency_fingerprint(linked_mod));
                    linked_modules.insert(mod_id.clone(), (linked_mod.clone(), public_fp));
                    linked_dep_fps.insert(mod_id.clone(), private_fp);
                    modules.insert(mod_id.clone(), linked_mod.clone());
                }

                for diag in &comp_product.diagnostics {
                    diagnostics.entry(diag.module.clone()).or_default().push(diag.clone());
                }
                blocked_modules.extend(comp_product.blocked_modules.clone());
                graphs.merge_from(&comp_product.graphs);

                new_retained_components.insert(comp_id.clone(), comp_product);
            }

            stats.linked_components = stats.linked_components_recomputed;

            let initialization_order = graphs.runtime.initialization_order().unwrap_or_default();
            let entry = parsed_sources.keys().next().cloned().unwrap_or_else(ModuleId::universe_root);
            (
                Arc::new(LinkedProgram {
                    universe: Arc::new(universe.clone()),
                    modules,
                    graphs,
                    entry,
                    initialization_order,
                }),
                new_retained_components,
                linked_dep_fps,
            )
        };

        let (resolution_hits_after, resolution_misses_after) = self.provider.base().resolution_metrics();
        stats.filesystem_resolution_hits = resolution_hits_after.saturating_sub(resolution_hits_before) as usize;
        stats.filesystem_resolution_misses = resolution_misses_after.saturating_sub(resolution_misses_before) as usize;

        let mut source_locations: BTreeMap<ModuleId, SourceLocation> = sources_by_module
            .iter()
            .map(|(id, state)| (id.clone(), state.location.clone()))
            .collect();
        for discovered in &new_discovered_sources {
            source_locations.insert(discovered.module.clone(), discovered.location.clone());
        }
        let unlinked: BTreeMap<ModuleId, UnlinkedModuleInterface> = interfaces
            .iter()
            .map(|(id, (iface, _))| (id.clone(), (**iface).clone()))
            .collect();

        let topology = Arc::new(ModuleTopology::from_parts(
            target_generation,
            universe,
            &unlinked,
            &source_locations,
        ));

        Ok(RebuildOutput {
            linked,
            parsed_sources,
            new_discovered_sources,
            interfaces,
            import_products,
            resolved_imports,
            reverse_importers: Arc::new(reverse_importers),
            sites_by_importer: Arc::new(sites_by_importer),
            reverse_site_importers: Arc::new(reverse_site_importers),
            linked_modules,
            linked_dependency_fingerprints,
            retained_components: new_retained_components,
            module_components: new_module_components,
            diagnostics,
            blocked_modules,
            stats,
            topology,
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
