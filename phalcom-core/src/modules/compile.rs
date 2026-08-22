//! Program-level compiler seam for closed linked module plans.

use super::artifact::ModuleMaterializationPlan;
use phalcom_modules::{
    BuiltinProject, BuiltinProjectSourceProvider, FilesystemSourceProvider, InterfaceBuilder, InterfaceError, LinkError, LinkedModule, LinkedProgram,
    ModuleComponent, ModuleId, ModuleKind, ModuleLinker, ModulePath, ModuleResolutionError, ModuleResolver, ProjectError, ProjectUniverse, SourceError,
    SourceId, SourceLocation, discover_owning_project,
};
use phalcom_semantic::SemanticDiagnostic;
use std::collections::{BTreeMap, HashSet};
use std::path::{Component, Path, PathBuf};
use std::sync::Arc;
use thiserror::Error;

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
#[derive(Clone, Debug, Default)]
pub struct ProgramSemanticDiagnostics {
    pub by_module: BTreeMap<ModuleId, Vec<SemanticDiagnostic>>,
}

impl ProgramSemanticDiagnostics {
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
        "standalone module execution cannot resolve import '{import_name}' (standalone modules only support builtin 'universe' and 'std' roots; sibling modules require a Project or Package)"
    )]
    StandaloneImportRequiresPackageContext { import_name: Box<str> },
    /// Context-free REPL cannot import modules without project context.
    #[error("context-free inline/REPL execution does not support module import '{import_name}' (requires project context)")]
    ReplImportRequiresProjectContext { import_name: Box<str> },
    /// Semantic type checking errors.
    #[error("semantic error: {0:?}")]
    Semantic(ProgramSemanticDiagnostics),
    /// Generic I/O error.
    #[error("io error: {0}")]
    Io(String),
}

/// Fully analyzed program snapshot ready for code generation.
#[derive(Clone, Debug)]
pub struct AnalyzedProgram {
    pub project_universe: Arc<ProjectUniverse>,
    pub linked: Arc<LinkedProgram>,
    pub semantic: Arc<phalcom_semantic::SemanticSnapshot>,
    pub sources: BTreeMap<ModuleId, Arc<phalcom_modules::source::ParsedModuleUnit>>,
    pub entry: ModuleId,
}

/// Analyzer that coordinates discovery, linking, and whole-workspace semantic analysis.
pub struct ProgramAnalyzer;

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
                let main_file = pkg_dir.join("main.ph");
                if !main_file.exists() {
                    return Err(ProgramCompileError::PackageNotExecutable(pkg_dir.display().to_string()));
                }
                let mut universe = ProjectUniverse::new();
                let name = pkg_dir.file_name().and_then(|s| s.to_str()).unwrap_or("package");
                let root_id = universe.load_synthetic_root(name, &pkg_dir, "main")?;
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
                if let Ok(Some(project_root)) = discover_owning_project(&file_path) {
                    let mut universe = ProjectUniverse::new();
                    let root_id = universe.load_root(project_root.join("project.toml"))?;
                    let project = universe.get_project(root_id).unwrap();
                    let canonical_file = file_path
                        .canonicalize()
                        .map_err(|e| ProgramCompileError::Io(format!("{}: {}", file_path.display(), e)))?;
                    let rel_path = canonical_file.strip_prefix(&project.source_root).map_err(|_| {
                        ProgramCompileError::Io(format!("file {} not under source root {}", file_path.display(), project.source_root.display()))
                    })?;
                    let module_path = relative_path_to_module_path(rel_path)?;
                    let entry_id = ModuleId {
                        project: root_id.into(),
                        path: module_path,
                    };
                    let provider = FilesystemSourceProvider::new();
                    Self::discover_and_analyze(Arc::new(universe), provider, entry_id)
                } else {
                    Self::analyze_standalone_module(file_path)
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
                let linker = ModuleLinker::new(universe.clone(), interfaces);
                let linked = Arc::new(linker.link(entry_id.clone(), &BTreeMap::new())?);

                let parsed_unit = Arc::new(phalcom_modules::source::ParsedModuleUnit::new(
                    entry_id.clone(),
                    ModuleKind::Module,
                    None,
                    source_text,
                    Arc::new(parsed.program),
                ));
                let mut sources = BTreeMap::new();
                sources.insert(entry_id.clone(), parsed_unit);

                let analysis = phalcom_semantic::analyze_workspace(phalcom_semantic::SemanticWorkspaceInput {
                    linked: linked.clone(),
                    sources: sources.clone(),
                    generation: 0,
                });

                if analysis.snapshot.has_errors() {
                    let mut by_module = BTreeMap::new();
                    for (m, d) in analysis.snapshot.diagnostics.iter() {
                        by_module.insert(m.clone(), d.to_vec());
                    }
                    return Err(ProgramCompileError::Semantic(ProgramSemanticDiagnostics { by_module }));
                }

                Ok(AnalyzedProgram {
                    project_universe: universe,
                    linked,
                    semantic: analysis.snapshot,
                    sources,
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
        let mut interfaces = BTreeMap::from([(entry_id.clone(), interface)]);
        let mut resolved = BTreeMap::new();

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

            let builtin = match &path.root {
                phalcom_ast::ast::ImportRoot::Absolute(root) if root.name == "universe" => BuiltinProject::Universe,
                phalcom_ast::ast::ImportRoot::Absolute(root) if root.name == "std" => BuiltinProject::Std,
                _ => {
                    return Err(ProgramCompileError::StandaloneImportRequiresPackageContext {
                        import_name: path.to_string().into(),
                    });
                }
            };
            let target_path = ModulePath::from_components(
                path.segments
                    .iter()
                    .map(|segment| ModuleComponent::from_identifier(&segment.name))
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(|e| ProgramCompileError::Io(e.to_string()))?,
            );
            let target_id = ModuleId::builtin(builtin, target_path);
            let provider = BuiltinProjectSourceProvider::new(builtin);
            let target_interface = provider.load_interface(&target_id)?;
            let target_parsed = provider.load_parsed(&target_id)?;
            interfaces.entry(target_id.clone()).or_insert(target_interface);
            sources.entry(target_id.clone()).or_insert(target_parsed);
            resolved.insert((entry_id.clone(), path.to_string()), target_id);
        }

        let linked = Arc::new(ModuleLinker::new(universe.clone(), interfaces).link(entry_id.clone(), &resolved)?);

        let analysis = phalcom_semantic::analyze_workspace(phalcom_semantic::SemanticWorkspaceInput {
            linked: linked.clone(),
            sources: sources.clone(),
            generation: 0,
        });

        if analysis.snapshot.has_errors() {
            let mut by_module = BTreeMap::new();
            for (m, d) in analysis.snapshot.diagnostics.iter() {
                by_module.insert(m.clone(), d.to_vec());
            }
            return Err(ProgramCompileError::Semantic(ProgramSemanticDiagnostics { by_module }));
        }

        Ok(AnalyzedProgram {
            project_universe: universe,
            linked,
            semantic: analysis.snapshot,
            sources,
            entry: entry_id,
        })
    }

    fn discover_and_analyze(
        universe: Arc<ProjectUniverse>,
        source_provider: FilesystemSourceProvider,
        entry: ModuleId,
    ) -> Result<AnalyzedProgram, ProgramCompileError> {
        let mut resolver = ModuleResolver::new(&universe, &source_provider);
        let mut interfaces = BTreeMap::new();
        let mut sources = BTreeMap::new();
        let mut resolved = BTreeMap::new();
        let mut visited = HashSet::new();
        let mut pending = vec![entry.clone()];
        visited.insert(entry.clone());

        if let Some(project_id) = entry.project.as_resolved() {
            let root_id = ModuleId::resolved(project_id, ModulePath::root());
            if visited.insert(root_id.clone()) {
                pending.push(root_id);
            }
        }

        while let Some(current_id) = pending.pop() {
            let parsed = resolver.load_parsed(&current_id)?;
            let interface = resolver.load_interface(&current_id)?;
            interfaces.insert(current_id.clone(), interface.clone());
            sources.insert(current_id.clone(), parsed);

            if let Some(project_id) = current_id.project.as_resolved() {
                if let Some(_proj) = universe.get_project(project_id) {
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
                let target_unit = resolver.resolve_import(&current_id, import_path)?;
                resolved.insert((current_id.clone(), import_path.to_string()), target_unit.id.clone());

                if visited.insert(target_unit.id.clone()) {
                    pending.push(target_unit.id);
                }
            }
        }

        let linker = ModuleLinker::new(universe.clone(), interfaces);
        let linked = Arc::new(linker.link(entry.clone(), &resolved)?);

        let analysis = phalcom_semantic::analyze_workspace(phalcom_semantic::SemanticWorkspaceInput {
            linked: linked.clone(),
            sources: sources.clone(),
            generation: 0,
        });

        if analysis.snapshot.has_errors() {
            let mut by_module = BTreeMap::new();
            for (m, d) in analysis.snapshot.diagnostics.iter() {
                by_module.insert(m.clone(), d.to_vec());
            }
            return Err(ProgramCompileError::Semantic(ProgramSemanticDiagnostics { by_module }));
        }

        Ok(AnalyzedProgram {
            project_universe: universe,
            linked,
            semantic: analysis.snapshot,
            sources,
            entry,
        })
    }
}

/// Compiler facade over an analyzed semantic program.
pub struct ProgramCompiler;

impl ProgramCompiler {
    /// Compiles an analyzed program into a fully linked `CompiledProgram`.
    pub fn compile_analyzed(analyzed: &AnalyzedProgram) -> Result<CompiledProgram, ProgramCompileError> {
        let mut modules = BTreeMap::new();
        for (id, linked_module) in &analyzed.linked.modules {
            let (source, source_text) = if let Some(parsed_unit) = analyzed.sources.get(id) {
                (parsed_unit.source.clone(), Some(parsed_unit.text.clone()))
            } else {
                (None, None)
            };
            modules.insert(id.clone(), compile_module(id.clone(), linked_module, source, source_text));
        }

        let exporter = phalcom_semantic::metadata::MetadataExporter::new(
            analyzed.semantic.store(),
            Some(analyzed.semantic.declarations()),
            None,
            None,
            phalcom_type_meta::header::MetadataProfile::RuntimePublic,
        );
        let metadata_bundle = exporter.build_bundle(&[]).ok().map(Arc::new);

        Ok(CompiledProgram {
            project_universe: analyzed.project_universe.clone(),
            linked: analyzed.linked.clone(),
            modules,
            entry: analyzed.entry.clone(),
            initialization_order: analyzed.linked.initialization_order.clone(),
            semantic_metadata: metadata_bundle,
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

fn compile_module(id: ModuleId, module: &LinkedModule, source: Option<SourceLocation>, source_text: Option<Arc<str>>) -> CompiledModule {
    let plan = ModuleMaterializationPlan::empty(module);
    CompiledModule {
        id,
        kind: module.interface.kind,
        source,
        source_text,
        interface: Arc::new(module.interface.clone()),
        linked_reads: module.linked_reads.clone(),
        plan,
    }
}
