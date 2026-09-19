//! Program-level compiler seam for closed linked module plans.

use super::artifact::ModuleMaterializationPlan;
use crate::typing::{MetadataPoolId, RuntimeCallEnvironmentRecipe, RuntimeTypeRecipe, RuntimeTypeRef};
use phalcom_common::range::SourceRange;
use phalcom_modules::{
    EntryOwnership, FilesystemSourceProvider, ImportSurface, InterfaceBuilder, InterfaceError, LinkError, LinkedModule, LinkedProgram, ModuleComponent,
    ModuleId, ModuleKind, ModuleLinker, ModulePath, ModuleResolutionError, ModuleResolver, ProjectError, ProjectUniverse, SourceError, SourceId,
    SourceLocation, SourceProvider, UniverseSourceProvider, classify_entry_ownership,
};
use phalcom_semantic::SemanticDiagnostic;
use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::path::{Component, Path, PathBuf};
use std::sync::Arc;
use thiserror::Error;

type RuntimeTraitEnvironmentRootSpec = (
    ModuleId,
    SourceRange,
    phalcom_semantic::identity::ImplId,
    phalcom_semantic::traits::TraitRequirementId,
    phalcom_semantic::types::id::TypeParameterId,
    phalcom_semantic::types::id::TypeId,
    String,
);

fn add_runtime_trait_environment_roots(
    roots: &mut Vec<RuntimeTraitEnvironmentRootSpec>,
    seen: &mut BTreeSet<(
        ModuleId,
        SourceRange,
        phalcom_semantic::identity::ImplId,
        phalcom_semantic::traits::TraitRequirementId,
        phalcom_semantic::types::id::TypeParameterId,
    )>,
    module: &ModuleId,
    range: SourceRange,
    source_impl: &phalcom_semantic::identity::ImplId,
    requirement: &phalcom_semantic::traits::TraitRequirementId,
    bindings: impl IntoIterator<Item = (phalcom_semantic::types::id::TypeParameterId, phalcom_semantic::types::id::TypeId)>,
) {
    for (parameter, ty) in bindings {
        let identity = (module.clone(), range, source_impl.clone(), requirement.clone(), parameter);
        if !seen.insert(identity) {
            continue;
        }
        let key = format!(
            "trait-environment-{}-{}-{}-{}-{}",
            range.start,
            range.end,
            source_impl.local.0,
            requirement.selector.encode(),
            parameter.index(),
        );
        roots.push((module.clone(), range, source_impl.clone(), requirement.clone(), parameter, ty, key));
    }
}

/// Entry selection for a program compilation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EntrySelection {
    /// Compile an already linked module ID.
    ModuleId(ModuleId),
    /// Compile a project rooted at directory with `project.toml`.
    Project(PathBuf),
    /// Compile a package directory containing `package.ph` and `main.ph`.
    Package(PathBuf),
    /// Compile a single module file path.
    Module(PathBuf),
    /// Compile standalone inline source text.
    Inline(Arc<str>),
}

/// One VM-independent compiled module.
#[derive(Clone, Debug, PartialEq)]
pub struct CompiledModule {
    /// Module identity.
    pub id: ModuleId,
    /// Source kind from the linked interface.
    pub kind: ModuleKind,
    /// Source location is attached by a source-backed compiler facade when
    /// available; the VM-independent linker itself only knows logical identity.
    pub source: Option<SourceLocation>,
    /// Source text retained for compilation.
    pub source_text: Option<Arc<str>>,
    /// Linked source interface.
    pub interface: Arc<phalcom_modules::LinkedModuleInterface>,
    /// Materialization/initializer artifact.
    pub plan: ModuleMaterializationPlan,
    /// Symbolic reads retained for compiler/runtime lowering.
    pub linked_reads: Vec<phalcom_modules::LinkedReadSpec>,
    /// Lowering semantics projected from formal analysis.
    pub lowering: Arc<super::semantic_lowering::ModuleLoweringSemantics>,
}

/// Closed compiled program passed to runtime materialization.
#[derive(Clone, Debug)]
pub struct CompiledProgram {
    /// Resolved project universe shared by every linked module.
    pub project_universe: Arc<ProjectUniverse>,
    /// Original linked program plan.
    pub linked: Arc<LinkedProgram>,
    /// Compiled modules keyed by semantic identity.
    pub modules: BTreeMap<ModuleId, CompiledModule>,
    /// Program entry module.
    pub entry: ModuleId,
    /// Precomputed initialization order.
    pub initialization_order: Vec<ModuleId>,
    /// Immutable semantic metadata bundle, if retained by build profile.
    pub semantic_metadata: Option<Arc<phalcom_type_meta::SemanticMetadataBundle>>,
}

/// Semantic diagnostics grouped by module.
#[derive(Clone, Default)]
pub struct ProgramSemanticDiagnostics {
    pub by_module: BTreeMap<ModuleId, Vec<SemanticDiagnostic>>,
    pub snapshot: Option<Arc<phalcom_semantic::SemanticSnapshot>>,
}

impl std::fmt::Display for ProgramSemanticDiagnostics {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut first = true;
        for (module, diags) in &self.by_module {
            for diag in diags {
                if !first {
                    writeln!(f)?;
                }
                first = false;
                writeln!(
                    f,
                    "{}: {:?} {:?}: {} at {:?}",
                    module, diag.severity, diag.code, diag.message, diag.primary_range
                )?;
                for label in &diag.labels {
                    writeln!(f, "  --> label: {} at {:?}", label.message, label.span.range)?;
                }
            }
        }
        Ok(())
    }
}

impl std::fmt::Debug for ProgramSemanticDiagnostics {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ProgramSemanticDiagnostics(\n{}\n)", self)
    }
}

impl ProgramSemanticDiagnostics {
    pub fn from_snapshot(snapshot: &phalcom_semantic::SemanticSnapshot) -> Self {
        let by_module = snapshot
            .diagnostics
            .iter()
            .map(|(module, diagnostics)| (module.clone(), diagnostics.to_vec()))
            .collect();
        Self {
            by_module,
            snapshot: Some(Arc::new(snapshot.clone())),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.by_module.values().all(|v| v.is_empty())
    }

    pub fn has_errors(&self) -> bool {
        self.by_module
            .values()
            .any(|v| v.iter().any(|d| d.severity == phalcom_semantic::DiagnosticSeverity::Error))
    }

    pub fn for_module(&self, module: &ModuleId) -> &[SemanticDiagnostic] {
        self.by_module.get(module).map(Vec::as_slice).unwrap_or(&[])
    }

    pub fn iter(&self) -> impl Iterator<Item = (&ModuleId, &[SemanticDiagnostic])> {
        self.by_module.iter().map(|(m, v)| (m, v.as_slice()))
    }
}

/// Program-level compile errors remain structured by phase.
#[derive(Debug, Error, Clone)]
pub enum ProgramCompileError {
    /// Static linking failed.
    #[error(transparent)]
    Link(#[from] LinkError),
    /// Parser error.
    #[error(transparent)]
    Parse(#[from] phalcom_ast::error::SyntaxError),
    /// Interface error.
    #[error(transparent)]
    Interface(#[from] InterfaceError),
    /// Requested entry is not part of the linked plan.
    #[error("entry module {0} is not present in linked program")]
    MissingEntry(ModuleId),
    /// Project is not executable.
    #[error("project manifest at '{0}' is not executable (no entry declared)")]
    ProjectNotExecutable(String),
    /// Package is not executable.
    #[error("package at '{0}' is not executable (missing main.ph)")]
    PackageNotExecutable(String),
    /// Project resolution failure.
    #[error("project error: {0}")]
    Project(#[from] ProjectError),
    /// Module resolution failure.
    #[error("resolution error: {0}")]
    Resolution(#[from] ModuleResolutionError),
    /// Module interface/load failure.
    #[error("module load error: {0}")]
    ModuleLoad(#[from] phalcom_modules::ModuleLoadError),
    /// Source I/O failure.
    #[error("source error: {0}")]
    Source(#[from] SourceError),
    /// Standalone module cannot import arbitrary sibling or project modules.
    #[error(
        "standalone module execution cannot resolve import '{import_name}' (standalone modules only support builtin 'universe' roots; sibling modules require a Project or Package)"
    )]
    StandaloneImportRequiresPackageContext { import_name: Box<str> },
    /// Context-free REPL cannot import modules without project context.
    #[error("context-free inline/REPL execution does not support module import '{import_name}' (requires project context)")]
    ReplImportRequiresProjectContext { import_name: Box<str> },
    /// Semantic type checking errors.
    #[error("semantic error:\n{0}")]
    Semantic(ProgramSemanticDiagnostics),
    /// Generic I/O error.
    #[error("io error: {0}")]
    Io(String),
    /// Canonical Universe semantic diagnostics differ from their reviewed baseline.
    #[error("canonical Universe semantic baseline rejected: {0}")]
    CanonicalSemanticBaseline(String),
    /// Canonical generic evidence could not be represented for lowering.
    #[error("generic evidence layout error: {0}")]
    GenericEvidenceLayout(String),
}

/// Fully analyzed program snapshot ready for code generation.
#[derive(Clone, Debug)]
pub struct AnalyzedProgram {
    pub project_universe: Arc<ProjectUniverse>,
    pub linked: Arc<LinkedProgram>,
    pub semantic: Arc<phalcom_semantic::SemanticSnapshot>,
    pub sources: BTreeMap<ModuleId, Arc<phalcom_modules::source::ParsedModuleUnit>>,
    /// Modules that this program owns at runtime. Semantic support modules may
    /// be present in `linked` and `sources` without being rematerialized into
    /// an already-bootstrapped Universe VM.
    pub runtime_modules: BTreeSet<ModuleId>,
    pub entry: ModuleId,
}

/// Analyzer that coordinates discovery, linking, and whole-workspace semantic analysis.
pub struct ProgramAnalyzer;

struct DiscoveredProgramClosure {
    interfaces: BTreeMap<ModuleId, phalcom_modules::UnlinkedModuleInterface>,
    sources: BTreeMap<ModuleId, Arc<phalcom_modules::source::ParsedModuleUnit>>,
    resolved: BTreeMap<(ModuleId, String), ModuleId>,
}

fn discover_reachable_dependencies<P: SourceProvider>(
    resolver: &mut ModuleResolver<'_, P>,
    universe: &ProjectUniverse,
    mut interfaces: BTreeMap<ModuleId, phalcom_modules::UnlinkedModuleInterface>,
    mut sources: BTreeMap<ModuleId, Arc<phalcom_modules::source::ParsedModuleUnit>>,
    mut resolved: BTreeMap<(ModuleId, String), ModuleId>,
    pending: impl IntoIterator<Item = ModuleId>,
    standalone_entry: Option<&ModuleId>,
) -> Result<DiscoveredProgramClosure, ProgramCompileError> {
    let mut visited = interfaces.keys().cloned().collect::<HashSet<_>>();
    let mut pending = pending.into_iter().collect::<Vec<_>>();
    for module in &pending {
        visited.insert(module.clone());
    }

    while let Some(current_id) = pending.pop() {
        let interface = if let Some(interface) = interfaces.get(&current_id) {
            interface.clone()
        } else {
            let parsed = resolver.load_parsed(&current_id)?;
            let interface = resolver.load_interface(&current_id)?;
            sources.insert(current_id.clone(), parsed);
            interfaces.insert(current_id.clone(), interface.clone());
            interface
        };

        if let Some(project_id) = current_id.project.as_resolved() {
            if universe.get_project(project_id).is_some() {
                let mut curr_parent = current_id.path.parent();
                while let Some(parent) = curr_parent {
                    let pkg_id = ModuleId::resolved(project_id, parent.clone());
                    if visited.insert(pkg_id.clone()) {
                        pending.push(pkg_id);
                    }
                    curr_parent = parent.parent();
                }
            }
        }

        for import_surface in &interface.imports {
            let import_path = match import_surface {
                phalcom_modules::ImportSurface::Module(m) => &m.path,
                phalcom_modules::ImportSurface::Selective(s) => &s.path,
                phalcom_modules::ImportSurface::ReExport(r) => &r.path,
            };

            if standalone_entry.is_some_and(|entry| entry == &current_id) {
                match &import_path.root {
                    phalcom_ast::ast::ImportRoot::Absolute(root) if root.name == "universe" => {}
                    _ => {
                        return Err(ProgramCompileError::StandaloneImportRequiresPackageContext {
                            import_name: import_path.to_string().into(),
                        });
                    }
                }
            }

            let target_unit = resolver.resolve_import(&current_id, import_path)?;
            resolved.insert((current_id.clone(), import_path.to_string()), target_unit.id.clone());
            if visited.insert(target_unit.id.clone()) {
                pending.push(target_unit.id);
            }
        }
    }

    Ok(DiscoveredProgramClosure { interfaces, sources, resolved })
}

impl ProgramAnalyzer {
    /// Analyzes an entry selection into an [`AnalyzedProgram`].
    pub fn analyze_entry_selection(entry: EntrySelection) -> Result<AnalyzedProgram, ProgramCompileError> {
        match entry {
            EntrySelection::ModuleId(entry_id) => Err(ProgramCompileError::Io(format!(
                "EntrySelection::ModuleId({entry_id}) cannot be analyzed without an existing linked project universe"
            ))),
            EntrySelection::Project(root_dir) => {
                let manifest_path = if root_dir.ends_with("project.toml") {
                    root_dir
                } else {
                    root_dir.join("project.toml")
                };
                if !manifest_path.exists() {
                    return Err(ProgramCompileError::Io(format!("project manifest not found at {}", manifest_path.display())));
                }
                let mut universe = ProjectUniverse::new();
                let root_id = universe.load_root(&manifest_path)?;
                let project = universe
                    .get_project(root_id)
                    .ok_or_else(|| ProgramCompileError::Io(format!("project {root_id:?} not found")))?;
                let entry_path = project
                    .entry
                    .clone()
                    .ok_or_else(|| ProgramCompileError::ProjectNotExecutable(manifest_path.display().to_string()))?;
                let entry_id = ModuleId {
                    project: root_id.into(),
                    path: entry_path,
                };
                let provider = FilesystemSourceProvider::new();
                Self::discover_and_analyze(Arc::new(universe), provider, entry_id)
            }
            EntrySelection::Package(pkg_dir) => {
                let canonical_pkg = pkg_dir
                    .canonicalize()
                    .map_err(|e| ProgramCompileError::Io(format!("{}: {e}", pkg_dir.display())))?;
                let pkg_file = canonical_pkg.join("package.ph");
                if !pkg_file.is_file() {
                    return Err(ProgramCompileError::Project(ProjectError::MissingRootPackage(canonical_pkg)));
                }
                let main_file = canonical_pkg.join("main.ph");
                if !main_file.is_file() {
                    return Err(ProgramCompileError::PackageNotExecutable(canonical_pkg.display().to_string()));
                }
                let mut universe = ProjectUniverse::new();
                let root_id = universe.load_standalone_package(&canonical_pkg, Some("main"))?;
                let entry_id = ModuleId {
                    project: root_id.into(),
                    path: ModulePath::from_components(vec![
                        ModuleComponent::from_identifier("main").map_err(|e| ProgramCompileError::Io(e.to_string()))?,
                    ]),
                };
                let provider = FilesystemSourceProvider::new();
                Self::discover_and_analyze(Arc::new(universe), provider, entry_id)
            }
            EntrySelection::Module(file_path) => {
                let canonical = file_path
                    .canonicalize()
                    .map_err(|e| ProgramCompileError::Io(format!("{}: {}", file_path.display(), e)))?;
                let mut universe = ProjectUniverse::new();
                let ownership = classify_entry_ownership(&canonical, &mut universe)?;
                match ownership {
                    EntryOwnership::ProjectOwned { project: root_id } => {
                        let project = universe.get_project(root_id).unwrap();
                        let rel_path = canonical.strip_prefix(&project.source_root).map_err(|_| {
                            ProgramCompileError::Io(format!("file {} not under source root {}", file_path.display(), project.source_root.display()))
                        })?;
                        let module_path = relative_path_to_module_path(rel_path)?;
                        let entry_id = ModuleId {
                            project: root_id.into(),
                            path: module_path,
                        };
                        let provider = FilesystemSourceProvider::new();
                        Self::discover_and_analyze(Arc::new(universe), provider, entry_id)
                    }
                    EntryOwnership::StandalonePackageOwned { package_root } => {
                        let root_id = universe.load_standalone_package(&package_root, None)?;
                        let project = universe.get_project(root_id).unwrap();
                        let rel_path = canonical.strip_prefix(&project.source_root).map_err(|_| {
                            ProgramCompileError::Io(format!("file {} not under package root {}", file_path.display(), project.source_root.display()))
                        })?;
                        let module_path = relative_path_to_module_path(rel_path)?;
                        let entry_id = ModuleId {
                            project: root_id.into(),
                            path: module_path,
                        };
                        let provider = FilesystemSourceProvider::new();
                        Self::discover_and_analyze(Arc::new(universe), provider, entry_id)
                    }
                    EntryOwnership::StandaloneModule { file } => Self::analyze_standalone_module(file),
                    EntryOwnership::Inline { .. } => Self::analyze_standalone_module(file_path),
                }
            }
            EntrySelection::Inline(source_text) => {
                let parsed = phalcom_ast::parse(&source_text, 0);
                if let Some(error) = parsed.errors.first() {
                    return Err(ProgramCompileError::Parse(error.clone()));
                }
                if !parsed.program.preamble.dependencies.is_empty() {
                    let dep = &parsed.program.preamble.dependencies[0];
                    let import_name = match dep {
                        phalcom_ast::ast::DependencyDecl::Import(phalcom_ast::ast::ImportDecl::Module(m)) => m.path.to_string(),
                        phalcom_ast::ast::DependencyDecl::Import(phalcom_ast::ast::ImportDecl::Selective(s)) => s.path.to_string(),
                        phalcom_ast::ast::DependencyDecl::ReExport(r) => r.path.to_string(),
                        phalcom_ast::ast::DependencyDecl::Expose(e) => e.child.name.clone(),
                    };
                    return Err(ProgramCompileError::ReplImportRequiresProjectContext {
                        import_name: import_name.into(),
                    });
                }
                let mut ids = phalcom_modules::SyntheticProjectIdAllocator;
                let entry_id = ModuleId::synthetic(ids.allocate(), ModulePath::root());
                let universe = Arc::new(ProjectUniverse::new());
                let interface = InterfaceBuilder::build(entry_id.clone(), ModuleKind::Module, &parsed.program)?;
                let mut interfaces = BTreeMap::new();
                interfaces.insert(entry_id.clone(), interface);

                let parsed_unit = Arc::new(phalcom_modules::source::ParsedModuleUnit::new(
                    entry_id.clone(),
                    ModuleKind::Module,
                    None,
                    source_text,
                    Arc::new(parsed.program),
                ));
                let mut sources = BTreeMap::new();
                sources.insert(entry_id.clone(), parsed_unit);

                // Inline evaluation has an implicit Universe runtime. Its semantic
                // workspace must therefore include the same source-authored
                // conformance declarations, rather than treating native member
                // surfaces as a substitute for canonical trait evidence.
                let provider = UniverseSourceProvider::new();
                let filesystem = FilesystemSourceProvider::new();
                let mut resolver = ModuleResolver::new(&universe, &filesystem);
                for node in provider
                    .nodes()
                    .iter()
                    .filter(|node| matches!(node.path, ["collections", ..] | ["scalar", "string"]))
                {
                    let path = ModulePath::from_components(
                        node.path
                            .iter()
                            .map(|component| ModuleComponent::from_identifier(component).expect("canonical Universe component"))
                            .collect::<Vec<_>>(),
                    );
                    let module = ModuleId::universe(path);
                    interfaces.insert(module.clone(), provider.load_interface(&module)?);
                    sources.insert(module.clone(), provider.load_parsed(&module)?);
                }
                let mut resolved = BTreeMap::new();
                for (module, interface) in &interfaces {
                    if !module.project.is_universe() {
                        continue;
                    }
                    for import in &interface.imports {
                        let path = match import {
                            ImportSurface::Module(decl) => &decl.path,
                            ImportSurface::Selective(decl) => &decl.path,
                            ImportSurface::ReExport(decl) => &decl.path,
                        };
                        let target = resolver.resolve_import(module, path)?;
                        resolved.insert((module.clone(), path.to_string()), target.id);
                    }
                }
                let linked = Arc::new(ModuleLinker::new(universe.clone(), interfaces).link_all(entry_id.clone(), &resolved)?);

                let analysis = phalcom_semantic::analyze_workspace(phalcom_semantic::SemanticWorkspaceInput::new(linked.clone(), sources.clone(), 0));

                Ok(AnalyzedProgram {
                    project_universe: universe,
                    linked,
                    semantic: analysis.snapshot,
                    sources,
                    runtime_modules: BTreeSet::from([entry_id.clone()]),
                    entry: entry_id,
                })
            }
        }
    }

    fn analyze_standalone_module(file_path: PathBuf) -> Result<AnalyzedProgram, ProgramCompileError> {
        let canonical = file_path
            .canonicalize()
            .map_err(|e| ProgramCompileError::Io(format!("{}: {e}", file_path.display())))?;
        if !canonical.is_file() {
            return Err(ProgramCompileError::Io(format!("{} is not a module file", canonical.display())));
        }
        let source_text: Arc<str> =
            Arc::from(std::fs::read_to_string(&canonical).map_err(|e| ProgramCompileError::Io(format!("{}: {e}", canonical.display())))?);
        let parsed = phalcom_ast::parse(&source_text, 0);
        if let Some(error) = parsed.errors.first() {
            return Err(ProgramCompileError::Parse(error.clone()));
        }
        let mut ids = phalcom_modules::SyntheticProjectIdAllocator;
        let entry_id = ModuleId::synthetic(ids.allocate(), ModulePath::root());
        let universe = Arc::new(ProjectUniverse::new());
        let interface = InterfaceBuilder::build(entry_id.clone(), ModuleKind::Module, &parsed.program)?;
        let interfaces = BTreeMap::from([(entry_id.clone(), interface)]);

        let mut sources = BTreeMap::new();
        let parsed_unit = Arc::new(phalcom_modules::source::ParsedModuleUnit::new(
            entry_id.clone(),
            ModuleKind::Module,
            Some(SourceLocation {
                source_id: SourceId(canonical.to_string_lossy().into()),
                display_path: canonical.clone(),
            }),
            source_text,
            Arc::new(parsed.program.clone()),
        ));
        sources.insert(entry_id.clone(), parsed_unit);

        for dependency in &parsed.program.preamble.dependencies {
            let path = match dependency {
                phalcom_ast::ast::DependencyDecl::Import(phalcom_ast::ast::ImportDecl::Module(decl)) => &decl.path,
                phalcom_ast::ast::DependencyDecl::Import(phalcom_ast::ast::ImportDecl::Selective(decl)) => &decl.path,
                phalcom_ast::ast::DependencyDecl::ReExport(decl) => &decl.path,
                phalcom_ast::ast::DependencyDecl::Expose(decl) => {
                    return Err(ProgramCompileError::StandaloneImportRequiresPackageContext {
                        import_name: decl.child.name.clone().into(),
                    });
                }
            };

            if !matches!(&path.root, phalcom_ast::ast::ImportRoot::Absolute(root) if root.name == "universe") {
                return Err(ProgramCompileError::StandaloneImportRequiresPackageContext {
                    import_name: path.to_string().into(),
                });
            }
        }

        let provider = FilesystemSourceProvider::new();
        let mut resolver = ModuleResolver::new(&universe, &provider);
        let discovered = discover_reachable_dependencies(
            &mut resolver,
            &universe,
            interfaces,
            sources,
            BTreeMap::new(),
            [entry_id.clone()],
            Some(&entry_id),
        )?;

        let linked = Arc::new(ModuleLinker::new(universe.clone(), discovered.interfaces).link(entry_id.clone(), &discovered.resolved)?);

        let analysis = phalcom_semantic::analyze_workspace(phalcom_semantic::SemanticWorkspaceInput::new(linked.clone(), discovered.sources.clone(), 0));

        Ok(AnalyzedProgram {
            project_universe: universe,
            linked,
            semantic: analysis.snapshot,
            sources: discovered.sources,
            runtime_modules: BTreeSet::from([entry_id.clone()]),
            entry: entry_id,
        })
    }

    fn discover_and_analyze(
        universe: Arc<ProjectUniverse>,
        source_provider: FilesystemSourceProvider,
        entry: ModuleId,
    ) -> Result<AnalyzedProgram, ProgramCompileError> {
        let mut resolver = ModuleResolver::new(&universe, &source_provider);
        let interfaces = BTreeMap::new();
        let sources = BTreeMap::new();
        let resolved = BTreeMap::new();
        let mut pending = vec![entry.clone()];
        if let Some(project_id) = entry.project.as_resolved() {
            let root_id = ModuleId::resolved(project_id, ModulePath::root());
            pending.push(root_id);
        }

        let discovered = discover_reachable_dependencies(&mut resolver, &universe, interfaces, sources, resolved, pending, None)?;

        let linker = ModuleLinker::new(universe.clone(), discovered.interfaces);
        let linked = Arc::new(linker.link(entry.clone(), &discovered.resolved)?);

        let analysis = phalcom_semantic::analyze_workspace(phalcom_semantic::SemanticWorkspaceInput::new(linked.clone(), discovered.sources.clone(), 0));

        Ok(AnalyzedProgram {
            project_universe: universe,
            // Universe support modules are already canonical, bootstrapped
            // runtime state. Keep them in the linked/semantic closure, but do
            // not rematerialize their project-specific artifacts over that
            // state when an ordinary package is run in a full VM.
            runtime_modules: linked.modules.keys().filter(|id| !id.project.is_universe()).cloned().collect(),
            linked,
            semantic: analysis.snapshot,
            sources: discovered.sources,
            entry,
        })
    }
}

/// Compiler facade over an analyzed semantic program.
pub struct ProgramCompiler;

impl ProgramCompiler {
    /// Compiles an analyzed program into a fully linked `CompiledProgram`.
    pub fn compile_analyzed(analyzed: &AnalyzedProgram) -> Result<CompiledProgram, ProgramCompileError> {
        if analyzed.semantic.has_errors() {
            return Err(ProgramCompileError::Semantic(ProgramSemanticDiagnostics::from_snapshot(&analyzed.semantic)));
        }
        Self::project_analyzed(analyzed)
    }

    /// Compiles the canonical source-complete Universe projection while
    /// preserving only the reviewed canonical diagnostic baseline. This remains
    /// crate-private so ordinary user/project compilation continues to reject
    /// semantic errors.
    pub(crate) fn compile_analyzed_for_canonical_bootstrap(analyzed: &AnalyzedProgram) -> Result<CompiledProgram, ProgramCompileError> {
        super::canonical_semantics::validate_canonical_error_baseline(&analyzed.semantic)
            .map_err(|error| ProgramCompileError::CanonicalSemanticBaseline(error.to_string()))?;
        Self::project_analyzed(analyzed)
    }

    fn project_analyzed(analyzed: &AnalyzedProgram) -> Result<CompiledProgram, ProgramCompileError> {
        let mut runtime_root_specs = Vec::<(ModuleId, SourceRange, phalcom_semantic::types::id::TypeId, String)>::new();
        let mut runtime_call_root_specs = Vec::<(
            ModuleId,
            SourceRange,
            phalcom_semantic::identity::CallableId,
            phalcom_semantic::types::id::TypeParameterId,
            phalcom_semantic::types::id::TypeId,
            String,
        )>::new();
        let mut runtime_trait_root_specs = Vec::<RuntimeTraitEnvironmentRootSpec>::new();
        let mut runtime_trait_root_seen = BTreeSet::new();
        for id in analyzed.linked.modules.keys() {
            if !analyzed.runtime_modules.contains(id) {
                continue;
            }
            if let Some(source) = analyzed.sources.get(id) {
                for (range, ty) in super::semantic_lowering::anonymous_product_type_roots(id, &analyzed.semantic, &source.program) {
                    runtime_root_specs.push((id.clone(), range, ty, format!("anonymous-product-{}-{}", range.start, range.end)));
                }
            }
            for (callable, analysis) in analyzed.semantic.callable_analyses.iter().filter(|(callable, _)| callable.owner.module() == id) {
                for expression in analysis.expressions.values() {
                    let Some(specialization) = expression.call_specialization.as_ref() else {
                        continue;
                    };
                    for &(parameter, ty) in specialization.bindings.iter() {
                        runtime_call_root_specs.push((
                            id.clone(),
                            expression.range,
                            specialization.callable.clone(),
                            parameter,
                            ty,
                            format!(
                                "call-environment-{}-{}-{}-{}",
                                expression.range.start,
                                expression.range.end,
                                specialization.callable.selector,
                                parameter.index()
                            ),
                        ));
                    }
                    for conformance in specialization.conformances.iter() {
                        if let phalcom_semantic::checker::analysis::GenericCallConformanceSource::Derived { evidence } = &conformance.source {
                            for (&parameter, &ty) in evidence.impl_bindings.bindings().iter() {
                                runtime_call_root_specs.push((
                                    id.clone(),
                                    expression.range,
                                    specialization.callable.clone(),
                                    parameter,
                                    ty,
                                    format!(
                                        "call-derived-{}-{}-{}-{}",
                                        expression.range.start,
                                        expression.range.end,
                                        specialization.callable.selector,
                                        parameter.index()
                                    ),
                                ));
                            }
                        }
                    }
                    if let Some(phalcom_semantic::trait_dispatch::TraitDispatchSite::Evidenced(selection)) = expression.trait_dispatch.as_ref()
                        && let Some(evidence) = analyzed.semantic.conformance_evidence_for(selection.exact_target, &selection.exact_trait_ref)
                    {
                        add_runtime_trait_environment_roots(
                            &mut runtime_trait_root_specs,
                            &mut runtime_trait_root_seen,
                            id,
                            expression.range,
                            &selection.source_impl,
                            &selection.requirement,
                            evidence.impl_environment.bindings.iter().map(|(&parameter, &ty)| (parameter, ty)),
                        );
                    }
                    let _ = callable;
                }
                for lane in analysis.for_iteration_lanes.values() {
                    for requirement_resolution in [&lane.iterator, &lane.next] {
                        let phalcom_semantic::checker::ForIterationRequirementResolution::Evidence { evidence, requirement, .. } = requirement_resolution
                        else {
                            continue;
                        };
                        match evidence {
                            phalcom_semantic::generic_evidence::GenericConformanceEvidence::Exact(exact) => {
                                add_runtime_trait_environment_roots(
                                    &mut runtime_trait_root_specs,
                                    &mut runtime_trait_root_seen,
                                    id,
                                    lane.source_range,
                                    &exact.source_impl,
                                    requirement,
                                    exact.impl_environment.bindings.iter().map(|(&parameter, &ty)| (parameter, ty)),
                                );
                            }
                            phalcom_semantic::generic_evidence::GenericConformanceEvidence::Derived(derived) => match &derived.origin {
                                phalcom_semantic::generic_evidence::DerivedConformanceOrigin::ConditionalSource { exact_source } => {
                                    add_runtime_trait_environment_roots(
                                        &mut runtime_trait_root_specs,
                                        &mut runtime_trait_root_seen,
                                        id,
                                        lane.source_range,
                                        &exact_source.source_impl,
                                        requirement,
                                        exact_source.impl_environment.bindings.iter().map(|(&parameter, &ty)| (parameter, ty)),
                                    );
                                }
                                phalcom_semantic::generic_evidence::DerivedConformanceOrigin::GuardedRule { rule, .. } => {
                                    add_runtime_trait_environment_roots(
                                        &mut runtime_trait_root_specs,
                                        &mut runtime_trait_root_seen,
                                        id,
                                        lane.source_range,
                                        &rule.impl_id,
                                        requirement,
                                        derived.impl_bindings.bindings().iter().map(|(&parameter, &ty)| (parameter, ty)),
                                    );
                                }
                            },
                            phalcom_semantic::generic_evidence::GenericConformanceEvidence::Assumed(_) => {}
                        }
                    }
                }
            }
        }
        let runtime_root_refs = runtime_root_specs
            .iter()
            .map(|(module, _, ty, key)| (module, key.as_str(), *ty))
            .chain(runtime_call_root_specs.iter().map(|(module, _, _, _, ty, key)| (module, key.as_str(), *ty)))
            .chain(runtime_trait_root_specs.iter().map(|(module, _, _, _, _, ty, key)| (module, key.as_str(), *ty)))
            .collect::<Vec<_>>();
        let exporter = phalcom_semantic::metadata::MetadataExporter::new(
            analyzed.semantic.store(),
            Some(analyzed.semantic.declarations()),
            None,
            None,
            phalcom_type_meta::header::MetadataProfile::RuntimePublic,
        )
        .with_project_universe(&analyzed.project_universe)
        .with_aliases(analyzed.semantic.type_aliases.as_ref());
        let mut stable_call_root_specs = Vec::with_capacity(runtime_call_root_specs.len());
        let mut exporter = exporter;
        for (module, range, callable, parameter, ty, key) in &runtime_call_root_specs {
            stable_call_root_specs.push((
                module.clone(),
                *range,
                callable.clone(),
                phalcom_semantic::metadata::stable_identity::to_stable_callable_with_context(
                    callable,
                    &phalcom_semantic::metadata::stable_identity::StableIdentityContext::new(&analyzed.project_universe),
                ),
                *parameter,
                exporter.export_type_parameter(*parameter),
                *ty,
                key.clone(),
            ));
        }
        let mut stable_trait_root_specs = Vec::with_capacity(runtime_trait_root_specs.len());
        for (module, range, source_impl, requirement, parameter, ty, key) in &runtime_trait_root_specs {
            stable_trait_root_specs.push((
                module.clone(),
                *range,
                source_impl.clone(),
                requirement.clone(),
                exporter.export_type_parameter(*parameter),
                *ty,
                key.clone(),
            ));
        }
        let metadata_bundle = Arc::new(
            exporter
                .build_bundle(&runtime_root_refs)
                .map_err(|e| ProgramCompileError::Io(format!("metadata export error: {e}")))?,
        );
        let mut runtime_type_roots = BTreeMap::<ModuleId, BTreeMap<SourceRange, RuntimeTypeRef>>::new();
        for ((module, range, _, _), root) in runtime_root_specs.iter().zip(metadata_bundle.runtime_roots.iter()) {
            runtime_type_roots.entry(module.clone()).or_default().insert(
                *range,
                RuntimeTypeRef::Base {
                    pool: MetadataPoolId(0),
                    node: root.form,
                },
            );
        }
        let mut runtime_call_environment_recipes = BTreeMap::<ModuleId, BTreeMap<SourceRange, Arc<RuntimeCallEnvironmentRecipe>>>::new();
        let mut call_bindings = BTreeMap::<
            (ModuleId, SourceRange, phalcom_semantic::identity::CallableId),
            Vec<(phalcom_type_meta::StableTypeParameterRef, RuntimeTypeRecipe)>,
        >::new();
        let mut call_type_recipes = BTreeMap::<
            (
                ModuleId,
                SourceRange,
                phalcom_semantic::identity::CallableId,
                phalcom_semantic::types::id::TypeParameterId,
            ),
            (phalcom_type_meta::StableTypeParameterRef, RuntimeTypeRecipe),
        >::new();
        for ((module, range, callable, stable_callable, parameter, stable_parameter, ty, key), root) in stable_call_root_specs
            .iter()
            .zip(metadata_bundle.runtime_roots.iter().skip(runtime_root_specs.len()))
        {
            let recipe = if phalcom_semantic::checker::associated::contains_any_type_parameter(&analyzed.semantic.store, *ty) {
                RuntimeTypeRecipe::Template(RuntimeTypeRef::Base {
                    pool: MetadataPoolId(0),
                    node: root.form,
                })
            } else {
                RuntimeTypeRecipe::Closed(RuntimeTypeRef::Base {
                    pool: MetadataPoolId(0),
                    node: root.form,
                })
            };
            call_type_recipes.insert((module.clone(), *range, callable.clone(), *parameter), (stable_parameter.clone(), recipe));
            if key.starts_with("call-environment-") {
                call_bindings
                    .entry((module.clone(), *range, callable.clone()))
                    .or_default()
                    .push((stable_parameter.clone(), recipe));
            }
            let _ = stable_callable;
        }
        let mut runtime_trait_environment_bindings = BTreeMap::<ModuleId, super::semantic_lowering::RuntimeTraitEnvironmentBindings>::new();
        for ((module, range, source_impl, requirement, stable_parameter, ty, _key), root) in stable_trait_root_specs.iter().zip(
            metadata_bundle
                .runtime_roots
                .iter()
                .skip(runtime_root_specs.len() + runtime_call_root_specs.len()),
        ) {
            let recipe = if phalcom_semantic::checker::associated::contains_any_type_parameter(&analyzed.semantic.store, *ty) {
                RuntimeTypeRecipe::Template(RuntimeTypeRef::Base {
                    pool: MetadataPoolId(0),
                    node: root.form,
                })
            } else {
                RuntimeTypeRecipe::Closed(RuntimeTypeRef::Base {
                    pool: MetadataPoolId(0),
                    node: root.form,
                })
            };
            let bindings = runtime_trait_environment_bindings
                .entry(module.clone())
                .or_default()
                .entry((*range, source_impl.clone(), requirement.clone()))
                .or_insert_with(|| Box::new([]));
            let mut updated = bindings.to_vec();
            updated.push((stable_parameter.clone(), recipe));
            updated.sort_by(|(left, _), (right, _)| left.cmp(right));
            *bindings = updated.into_boxed_slice();
        }
        let mut call_conformance_slots =
            BTreeMap::<(ModuleId, SourceRange, phalcom_semantic::identity::CallableId), Vec<crate::typing::RuntimeConformanceSlotRecipe>>::new();

        for (callable, analysis) in analyzed.semantic.callable_analyses.iter() {
            if !analyzed.runtime_modules.contains(callable.owner.module()) {
                continue;
            }
            let caller_owner_sig = match &callable.owner {
                phalcom_semantic::identity::CallableOwnerId::Declaration(decl) => {
                    if callable.side == phalcom_semantic::identity::DispatchSide::Instance {
                        analyzed
                            .semantic
                            .declarations
                            .generic_signature(decl)
                            .or_else(|| analyzed.semantic.trait_headers.get(decl).and_then(|h| h.generic_signature.as_ref()))
                    } else {
                        None
                    }
                }
                phalcom_semantic::identity::CallableOwnerId::Variant(var) => {
                    if callable.side == phalcom_semantic::identity::DispatchSide::Instance {
                        analyzed.semantic.declarations.generic_signature(&var.owner)
                    } else {
                        None
                    }
                }
                phalcom_semantic::identity::CallableOwnerId::Conformance(impl_id) => analyzed
                    .semantic
                    .conformance_witness_plans
                    .get(impl_id)
                    .and_then(|plan| plan.generic_signature.as_ref())
                    .or_else(|| {
                        analyzed
                            .semantic
                            .derived_conformance_rules
                            .iter()
                            .find(|r| &r.impl_id == impl_id)
                            .map(|r| &r.generic_signature)
                    }),
            };
            let mut caller_evidence = phalcom_semantic::generic_evidence::GenericEvidenceEnvironment::empty();
            if let Some(owner_sig) = caller_owner_sig {
                let owner_env = phalcom_semantic::generic_evidence::GenericEvidenceEnvironment::try_from_constraints(
                    &owner_sig.constraints,
                    owner_sig.owner.clone(),
                    &analyzed.semantic.trait_surfaces,
                )
                .map_err(|error| ProgramCompileError::GenericEvidenceLayout(format!("{error:?}")))?;
                caller_evidence = caller_evidence
                    .try_compose(&owner_env, &analyzed.semantic.trait_surfaces)
                    .map_err(|error| ProgramCompileError::GenericEvidenceLayout(format!("{error:?}")))?;
            }
            if let Some(sig) = analyzed.semantic.callable_signatures.get(callable) {
                if let Some(generics) = &sig.generics {
                    let callable_env = phalcom_semantic::generic_evidence::GenericEvidenceEnvironment::try_from_constraints(
                        &generics.constraints,
                        generics.owner.clone(),
                        &analyzed.semantic.trait_surfaces,
                    )
                    .map_err(|error| ProgramCompileError::GenericEvidenceLayout(format!("{error:?}")))?;
                    caller_evidence = caller_evidence
                        .try_compose(&callable_env, &analyzed.semantic.trait_surfaces)
                        .map_err(|error| ProgramCompileError::GenericEvidenceLayout(format!("{error:?}")))?;
                }
            }

            for expression in analysis.expressions.values() {
                let Some(specialization) = expression.call_specialization.as_ref() else {
                    continue;
                };

                let callee = &specialization.callable;
                let callee_owner_sig = match &callee.owner {
                    phalcom_semantic::identity::CallableOwnerId::Declaration(decl) => {
                        if callee.side == phalcom_semantic::identity::DispatchSide::Instance {
                            analyzed
                                .semantic
                                .declarations
                                .generic_signature(decl)
                                .or_else(|| analyzed.semantic.trait_headers.get(decl).and_then(|h| h.generic_signature.as_ref()))
                        } else {
                            None
                        }
                    }
                    phalcom_semantic::identity::CallableOwnerId::Variant(var) => {
                        if callee.side == phalcom_semantic::identity::DispatchSide::Instance {
                            analyzed.semantic.declarations.generic_signature(&var.owner)
                        } else {
                            None
                        }
                    }
                    phalcom_semantic::identity::CallableOwnerId::Conformance(impl_id) => analyzed
                        .semantic
                        .conformance_witness_plans
                        .get(impl_id)
                        .and_then(|plan| plan.generic_signature.as_ref())
                        .or_else(|| {
                            analyzed
                                .semantic
                                .derived_conformance_rules
                                .iter()
                                .find(|r| &r.impl_id == impl_id)
                                .map(|r| &r.generic_signature)
                        }),
                };
                let mut callee_evidence = phalcom_semantic::generic_evidence::GenericEvidenceEnvironment::empty();
                if let Some(owner_sig) = callee_owner_sig {
                    let owner_env = phalcom_semantic::generic_evidence::GenericEvidenceEnvironment::try_from_constraints(
                        &owner_sig.constraints,
                        owner_sig.owner.clone(),
                        &analyzed.semantic.trait_surfaces,
                    )
                    .map_err(|error| ProgramCompileError::GenericEvidenceLayout(format!("{error:?}")))?;
                    callee_evidence = callee_evidence
                        .try_compose(&owner_env, &analyzed.semantic.trait_surfaces)
                        .map_err(|error| ProgramCompileError::GenericEvidenceLayout(format!("{error:?}")))?;
                }
                if let Some(sig) = analyzed.semantic.callable_signatures.get(callee) {
                    if let Some(generics) = &sig.generics {
                        let callable_env = phalcom_semantic::generic_evidence::GenericEvidenceEnvironment::try_from_constraints(
                            &generics.constraints,
                            generics.owner.clone(),
                            &analyzed.semantic.trait_surfaces,
                        )
                        .map_err(|error| ProgramCompileError::GenericEvidenceLayout(format!("{error:?}")))?;
                        callee_evidence = callee_evidence
                            .try_compose(&callable_env, &analyzed.semantic.trait_surfaces)
                            .map_err(|error| ProgramCompileError::GenericEvidenceLayout(format!("{error:?}")))?;
                    }
                }

                let layout_len = callee_evidence.layout().len();
                let mut slots: Vec<Option<crate::typing::RuntimeConformanceSlotRecipe>> = vec![None; layout_len];
                let mut callee_instantiation = phalcom_semantic::types::instantiation::GenericInstantiation::default();
                for &(parameter, ty) in specialization.bindings.iter() {
                    callee_instantiation.bind_type(parameter, ty);
                }
                let mut materialization_store = (*analyzed.semantic.store).clone();
                for (slot_key, slot_idx) in callee_evidence.layout().slots.iter() {
                    let idx = *slot_idx as usize;
                    let instantiated_subject = match &slot_key.conformance.subject {
                        phalcom_semantic::types::parameter::TypeTerm::Canonical(ty) => phalcom_semantic::types::instantiation::materialize_type(
                            &mut materialization_store,
                            *ty,
                            &callee_instantiation,
                            phalcom_semantic::types::instantiation::RowMaterializationMode::PreserveUnboundStableTail,
                        )
                        .map(phalcom_semantic::types::parameter::TypeTerm::Canonical)
                        .map_err(|error| ProgramCompileError::GenericEvidenceLayout(format!("conformance subject instantiation failed: {error:?}")))?,
                        _ => slot_key.conformance.subject.clone(),
                    };
                    let instantiated_trait_ref = phalcom_semantic::traits::TraitRef::new(
                        slot_key.conformance.trait_ref.declaration.clone(),
                        slot_key
                            .conformance
                            .trait_ref
                            .arguments
                            .iter()
                            .map(|&arg| {
                                phalcom_semantic::types::instantiation::materialize_type(
                                    &mut materialization_store,
                                    arg,
                                    &callee_instantiation,
                                    phalcom_semantic::types::instantiation::RowMaterializationMode::PreserveUnboundStableTail,
                                )
                                .map_err(|error| {
                                    ProgramCompileError::GenericEvidenceLayout(format!("conformance trait argument instantiation failed: {error:?}"))
                                })
                            })
                            .collect::<Result<Vec<_>, _>>()?,
                    );
                    let instantiated_conformance = phalcom_semantic::generic_evidence::GenericConformanceKey::new(instantiated_subject, instantiated_trait_ref);
                    let binding = specialization.conformances.iter().find(|b| b.callee_conformance == instantiated_conformance);
                    if let Some(binding) = binding {
                        match &binding.source {
                            phalcom_semantic::checker::analysis::GenericCallConformanceSource::Exact { evidence } => {
                                if let Some(selection) = evidence.requirements.get(&slot_key.requirement) {
                                    let selector = analyzed
                                        .semantic
                                        .trait_surfaces
                                        .get(&slot_key.conformance.trait_ref.declaration)
                                        .and_then(|s| s.get(&slot_key.requirement))
                                        .map(|m| m.requirement.selector.clone())
                                        .unwrap_or_else(|| slot_key.requirement.selector.clone());
                                    let callable = match selection {
                                        phalcom_semantic::impls::RequirementSelectionTemplate::ConformanceCallable { callable }
                                        | phalcom_semantic::impls::RequirementSelectionTemplate::TraitDefault { callable }
                                        | phalcom_semantic::impls::RequirementSelectionTemplate::InherentCallable { callable, .. } => Some(callable.clone()),
                                        phalcom_semantic::impls::RequirementSelectionTemplate::ConditionalInherent { candidate, .. } => {
                                            Some(candidate.callable.clone())
                                        }
                                        phalcom_semantic::impls::RequirementSelectionTemplate::DataComponent { .. } => None,
                                    };
                                    if let Some(callable) = callable {
                                        if idx < slots.len() {
                                            slots[idx] = Some(crate::typing::RuntimeConformanceSlotRecipe::Selection { callable, selector });
                                        }
                                    }
                                }
                            }
                            phalcom_semantic::checker::analysis::GenericCallConformanceSource::Forwarded { caller_conformance } => {
                                let caller_slot_key =
                                    phalcom_semantic::generic_evidence::GenericEvidenceSlotKey::new(caller_conformance.clone(), slot_key.requirement.clone());
                                if let Some(caller_slot) = caller_evidence.slot_for(&caller_slot_key) {
                                    if idx < slots.len() {
                                        slots[idx] = Some(crate::typing::RuntimeConformanceSlotRecipe::Forwarded(caller_slot));
                                    }
                                }
                            }
                            phalcom_semantic::checker::analysis::GenericCallConformanceSource::Derived { evidence } => {
                                if let Some(selection) = evidence.requirements().get(&slot_key.requirement) {
                                    let selector = analyzed
                                        .semantic
                                        .trait_surfaces
                                        .get(&slot_key.conformance.trait_ref.declaration)
                                        .and_then(|s| s.get(&slot_key.requirement))
                                        .map(|m| m.requirement.selector.clone())
                                        .unwrap_or_else(|| slot_key.requirement.selector.clone());
                                    let (callable, is_trait_default) = match selection {
                                        phalcom_semantic::impls::RequirementSelectionTemplate::ConformanceCallable { callable } => {
                                            (Some(callable.clone()), false)
                                        }
                                        phalcom_semantic::impls::RequirementSelectionTemplate::TraitDefault { callable } => (Some(callable.clone()), true),
                                        phalcom_semantic::impls::RequirementSelectionTemplate::InherentCallable { callable, .. } => {
                                            (Some(callable.clone()), false)
                                        }
                                        phalcom_semantic::impls::RequirementSelectionTemplate::ConditionalInherent { candidate, .. } => {
                                            (Some(candidate.callable.clone()), false)
                                        }
                                        phalcom_semantic::impls::RequirementSelectionTemplate::DataComponent { .. } => (None, false),
                                    };
                                    if let Some(callable) = callable {
                                        let mut type_bindings = Vec::new();
                                        for &param in evidence.impl_bindings.bindings().keys() {
                                            if let Some((stable_param, recipe)) =
                                                call_type_recipes.get(&(callable.owner.module().clone(), expression.range, callee.clone(), param))
                                            {
                                                type_bindings.push((stable_param.clone(), *recipe));
                                            }
                                        }
                                        type_bindings.sort_by(|(left, _), (right, _)| left.cmp(right));

                                        let mut nested_slots = Vec::new();
                                        if !is_trait_default {
                                            for nested_item in evidence.nested.iter() {
                                                if let phalcom_semantic::generic_evidence::NestedGenericEvidence::Conformance(nested_conformance) = nested_item
                                                {
                                                    match nested_conformance {
                                                        phalcom_semantic::generic_evidence::GenericConformanceEvidence::Exact(exact_ev) => {
                                                            if let Some((req, nested_selection)) = exact_ev.requirements.iter().next() {
                                                                let nested_callable = match nested_selection {
                                                                    phalcom_semantic::impls::RequirementSelectionTemplate::ConformanceCallable { callable }
                                                                    | phalcom_semantic::impls::RequirementSelectionTemplate::TraitDefault { callable }
                                                                    | phalcom_semantic::impls::RequirementSelectionTemplate::InherentCallable {
                                                                        callable,
                                                                        ..
                                                                    } => Some(callable.clone()),
                                                                    phalcom_semantic::impls::RequirementSelectionTemplate::ConditionalInherent {
                                                                        candidate,
                                                                        ..
                                                                    } => Some(candidate.callable.clone()),
                                                                    phalcom_semantic::impls::RequirementSelectionTemplate::DataComponent { .. } => None,
                                                                };
                                                                if let Some(nested_callable) = nested_callable {
                                                                    let nested_selector = analyzed
                                                                        .semantic
                                                                        .trait_surfaces
                                                                        .get(&exact_ev.exact_trait_ref.declaration)
                                                                        .and_then(|s| s.get(req))
                                                                        .map(|m| m.requirement.selector.clone())
                                                                        .unwrap_or_else(|| nested_callable.selector.clone());
                                                                    nested_slots.push(crate::typing::RuntimeConformanceSlotRecipe::Selection {
                                                                        callable: nested_callable,
                                                                        selector: nested_selector,
                                                                    });
                                                                }
                                                            }
                                                        }
                                                        phalcom_semantic::generic_evidence::GenericConformanceEvidence::Assumed(assumed) => {
                                                            let trait_decl = &assumed.key.trait_ref.declaration;
                                                            let req = analyzed
                                                                .semantic
                                                                .trait_surfaces
                                                                .get(trait_decl)
                                                                .and_then(|s| s.iter().next().map(|(r, _)| r.clone()));
                                                            if let Some(req) = req {
                                                                let caller_slot_key =
                                                                    phalcom_semantic::generic_evidence::GenericEvidenceSlotKey::new(assumed.key.clone(), req);
                                                                if let Some(caller_slot) = caller_evidence.slot_for(&caller_slot_key) {
                                                                    nested_slots.push(crate::typing::RuntimeConformanceSlotRecipe::Forwarded(caller_slot));
                                                                }
                                                            }
                                                        }
                                                        phalcom_semantic::generic_evidence::GenericConformanceEvidence::Derived(nested_derived) => {
                                                            if let Some((req, nested_selection)) = nested_derived.requirements().iter().next() {
                                                                let nested_callable = match nested_selection {
                                                                    phalcom_semantic::impls::RequirementSelectionTemplate::ConformanceCallable { callable }
                                                                    | phalcom_semantic::impls::RequirementSelectionTemplate::TraitDefault { callable }
                                                                    | phalcom_semantic::impls::RequirementSelectionTemplate::InherentCallable {
                                                                        callable,
                                                                        ..
                                                                    } => Some(callable.clone()),
                                                                    phalcom_semantic::impls::RequirementSelectionTemplate::ConditionalInherent {
                                                                        candidate,
                                                                        ..
                                                                    } => Some(candidate.callable.clone()),
                                                                    phalcom_semantic::impls::RequirementSelectionTemplate::DataComponent { .. } => None,
                                                                };
                                                                if let Some(nested_callable) = nested_callable {
                                                                    let trait_decl = match &nested_derived.origin {
                                                                        phalcom_semantic::generic_evidence::DerivedConformanceOrigin::ConditionalSource {
                                                                            exact_source,
                                                                        } => &exact_source.exact_trait_ref.declaration,
                                                                        phalcom_semantic::generic_evidence::DerivedConformanceOrigin::GuardedRule {
                                                                            exact_trait_ref,
                                                                            ..
                                                                        } => &exact_trait_ref.declaration,
                                                                    };
                                                                    let nested_selector = analyzed
                                                                        .semantic
                                                                        .trait_surfaces
                                                                        .get(trait_decl)
                                                                        .and_then(|s| s.get(req))
                                                                        .map(|m| m.requirement.selector.clone())
                                                                        .unwrap_or_else(|| nested_callable.selector.clone());
                                                                    nested_slots.push(crate::typing::RuntimeConformanceSlotRecipe::Selection {
                                                                        callable: nested_callable,
                                                                        selector: nested_selector,
                                                                    });
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }

                                        if idx < slots.len() {
                                            slots[idx] = Some(crate::typing::RuntimeConformanceSlotRecipe::DerivedConditional {
                                                callable,
                                                selector,
                                                type_bindings: type_bindings.into_boxed_slice(),
                                                nested_slots: nested_slots.into_boxed_slice(),
                                            });
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                let resolved_slots = slots
                    .into_iter()
                    .enumerate()
                    .map(|(slot, recipe)| {
                        recipe.ok_or_else(|| {
                            ProgramCompileError::GenericEvidenceLayout(format!(
                                "missing runtime conformance recipe for semantic slot {slot} at call to {callee:?}"
                            ))
                        })
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                call_conformance_slots.insert((callable.owner.module().clone(), expression.range, callee.clone()), resolved_slots);
            }
        }

        let mut all_call_keys = std::collections::BTreeSet::new();
        for key in call_bindings.keys() {
            all_call_keys.insert(key.clone());
        }
        for key in call_conformance_slots.keys() {
            all_call_keys.insert(key.clone());
        }

        for (module, range, callee_id) in all_call_keys {
            let mut bindings = call_bindings.remove(&(module.clone(), range, callee_id.clone())).unwrap_or_default();
            bindings.sort_by(|(left, _), (right, _)| left.cmp(right));
            let stable_callable = phalcom_semantic::metadata::stable_identity::to_stable_callable_with_context(
                &callee_id,
                &phalcom_semantic::metadata::stable_identity::StableIdentityContext::new(&analyzed.project_universe),
            );
            let conformance_slots = call_conformance_slots
                .remove(&(module.clone(), range, callee_id))
                .unwrap_or_default()
                .into_boxed_slice();
            runtime_call_environment_recipes.entry(module).or_default().insert(
                range,
                Arc::new(RuntimeCallEnvironmentRecipe {
                    callable: stable_callable,
                    bindings: bindings.into_boxed_slice(),
                    conformance_slots,
                }),
            );
        }
        let mut modules = BTreeMap::new();
        for (id, linked_module) in &analyzed.linked.modules {
            if !analyzed.runtime_modules.contains(id) {
                continue;
            }
            let (source, source_text) = if let Some(parsed_unit) = analyzed.sources.get(id) {
                (parsed_unit.source.clone(), Some(parsed_unit.text.clone()))
            } else {
                (None, None)
            };
            let mut lowering = super::semantic_lowering::build_module_lowering_semantics_with_runtime_types_and_calls(
                id,
                &analyzed.semantic,
                &analyzed.project_universe,
                runtime_type_roots.get(id).unwrap_or(&BTreeMap::new()),
                runtime_call_environment_recipes.get(id).unwrap_or(&BTreeMap::new()),
            )
            .map_err(|e| ProgramCompileError::Io(format!("lowering projection error in {id}: {e}")))?;
            if let Some(environments) = runtime_trait_environment_bindings.get(id) {
                super::semantic_lowering::attach_runtime_trait_environments(&mut lowering, environments);
            }
            modules.insert(id.clone(), compile_module(id.clone(), linked_module, source, source_text, Arc::new(lowering)));
        }

        Ok(CompiledProgram {
            project_universe: analyzed.project_universe.clone(),
            linked: analyzed.linked.clone(),
            modules,
            entry: analyzed.entry.clone(),
            initialization_order: analyzed
                .linked
                .initialization_order
                .iter()
                .filter(|id| analyzed.runtime_modules.contains(*id))
                .cloned()
                .collect(),
            semantic_metadata: Some(metadata_bundle),
        })
    }

    /// Compiles an entry selection by analyzing it first and compiling the analyzed result.
    pub fn compile_entry_selection(entry: EntrySelection) -> Result<CompiledProgram, ProgramCompileError> {
        let analyzed = ProgramAnalyzer::analyze_entry_selection(entry)?;
        Self::compile_analyzed(&analyzed)
    }
}

fn relative_path_to_module_path(rel_path: &Path) -> Result<ModulePath, ProgramCompileError> {
    let mut components = Vec::new();
    if let Some(parent) = rel_path.parent() {
        for comp in parent.components() {
            if let Component::Normal(os_str) = comp {
                let s = os_str.to_str().unwrap_or_default();
                if !s.is_empty() {
                    components.push(ModuleComponent::from_kebab(s).map_err(|e| ProgramCompileError::Io(e.to_string()))?);
                }
            }
        }
    }
    let file_stem = rel_path.file_stem().and_then(|s| s.to_str()).unwrap_or_default();
    if file_stem != "package" && !file_stem.is_empty() {
        components.push(ModuleComponent::from_kebab(file_stem).map_err(|e| ProgramCompileError::Io(e.to_string()))?);
    }
    Ok(ModulePath::from_components(components))
}

fn compile_module(
    id: ModuleId,
    module: &LinkedModule,
    source: Option<SourceLocation>,
    source_text: Option<Arc<str>>,
    lowering: Arc<super::semantic_lowering::ModuleLoweringSemantics>,
) -> CompiledModule {
    let plan = ModuleMaterializationPlan::empty(module);
    CompiledModule {
        id,
        kind: module.interface.kind,
        source,
        source_text,
        interface: Arc::new(module.interface.clone()),
        linked_reads: module.linked_reads.clone(),
        plan,
        lowering,
    }
}

#[cfg(test)]
mod tests {
    use crate::typing::RuntimeTypeRecipe;
    use phalcom_modules::{ModuleComponent, ModuleId, ModulePath};

    #[test]
    fn canonical_range_lower_option_closes_number_at_is_none_call_edge() {
        let canonical = crate::modules::canonical_universe_program().expect("canonical Universe should build");
        let range_id = ModuleId::universe(ModulePath::from_components(vec![
            ModuleComponent::from_identifier("collections").expect("collections component"),
            ModuleComponent::from_identifier("range").expect("range component"),
        ]));
        let source = canonical
            .source_index()
            .unit(&range_id)
            .map(|module| module.text.clone())
            .expect("canonical Range source text");
        let source = source.as_ref();
        let lower_offset = source.find("let lowerOpt = _range.rawLower").expect("RangeIterator lower binding");
        let is_none_offset = source[lower_offset..]
            .find("lowerOpt.isNone")
            .map(|offset| lower_offset + offset)
            .expect("RangeIterator lower isNone observation");
        let is_none_end = is_none_offset + "lowerOpt.isNone".len();
        let module = canonical.program().modules.get(&range_id).expect("canonical Range module artifact");
        let (range, recipe) = module
            .lowering
            .call_environment_recipes
            .iter()
            .find(|(range, recipe)| range.start >= is_none_offset && range.end <= is_none_end && recipe.callable.selector.as_ref() == "isNone")
            .expect("RangeIterator lowerOpt.isNone call recipe");
        assert!(
            matches!(recipe.bindings.as_ref(), [(_, RuntimeTypeRecipe::Closed(_))]),
            "RangeIterator lowerOpt.isNone must close Option.T as Number at {range:?}, got {recipe:?}"
        );
    }
}
