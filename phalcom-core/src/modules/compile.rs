//! Program-level compiler seam for closed linked module plans.

use super::artifact::ModuleMaterializationPlan;
use phalcom_modules::{
    BuiltinProject, BuiltinProjectSourceProvider, FilesystemSourceProvider, InterfaceBuilder, InterfaceError, LinkError, LinkedModule, LinkedProgram,
    ModuleComponent, ModuleId, ModuleKind, ModuleLinker, ModulePath, ModuleResolutionError, ModuleResolver, ProjectError, ProjectUniverse, SourceError,
    SourceId, SourceLocation, SourceProvider, discover_owning_project,
};
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
    #[error("type error: {0:?}")]
    Type(Vec<phalcom_semantic::SemanticDiagnostic>),
    /// Generic I/O error.
    #[error("io error: {0}")]
    Io(String),
}

/// Compiler facade over an already-linked program or source project.
pub struct ProgramCompiler {
    linked: Option<Arc<LinkedProgram>>,
}

impl ProgramCompiler {
    /// Creates a compiler for a closed linked plan.
    pub fn new(linked: Arc<LinkedProgram>) -> Self {
        Self { linked: Some(linked) }
    }

    /// Compiles an entry selection into a fully linked `CompiledProgram`.
    pub fn compile_entry_selection(entry: EntrySelection) -> Result<CompiledProgram, ProgramCompileError> {
        match entry {
            EntrySelection::ModuleId(entry_id) => Err(ProgramCompileError::Io(format!(
                "EntrySelection::ModuleId({entry_id}) cannot be compiled without an existing linked project universe; use ProgramCompiler::new(linked).compile_entry(...)"
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
                Self::discover_and_link(Arc::new(universe), provider, entry_id)
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
                Self::discover_and_link(Arc::new(universe), provider, entry_id)
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
                    Self::discover_and_link(Arc::new(universe), provider, entry_id)
                } else {
                    Self::compile_standalone_module(file_path)
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

                run_semantic_typecheck(&entry_id, &parsed.program)?;

                let plan = ModuleMaterializationPlan::empty(&linked.modules[&entry_id]);
                let compiled_mod = CompiledModule {
                    id: entry_id.clone(),
                    kind: ModuleKind::Module,
                    source: None,
                    source_text: Some(source_text.clone()),
                    interface: Arc::new(linked.modules[&entry_id].interface.clone()),
                    plan,
                    linked_reads: linked.modules[&entry_id].linked_reads.clone(),
                };
                let mut modules = BTreeMap::new();
                modules.insert(entry_id.clone(), compiled_mod);

                Ok(CompiledProgram {
                    project_universe: universe,
                    linked: linked.clone(),
                    modules,
                    entry: entry_id.clone(),
                    initialization_order: linked.initialization_order.clone(),
                })
            }
        }
    }

    fn compile_standalone_module(file_path: PathBuf) -> Result<CompiledProgram, ProgramCompileError> {
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

        // Standalone modules have no sibling/package authority. They may still
        // import the two toolchain builtin roots because those are provider-
        // backed and do not depend on a filesystem project context.
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
            interfaces.entry(target_id.clone()).or_insert(target_interface);
            resolved.insert((entry_id.clone(), path.to_string()), target_id);
        }

        let linked = Arc::new(ModuleLinker::new(universe.clone(), interfaces).link(entry_id.clone(), &resolved)?);
        let mut modules = BTreeMap::new();
        for (id, linked_module) in &linked.modules {
            let (source, text) = if id == &entry_id {
                (
                    Some(SourceLocation {
                        source_id: SourceId(canonical.to_string_lossy().into()),
                        display_path: canonical.clone(),
                    }),
                    Some(source_text.clone()),
                )
            } else if let Some(builtin) = id.project.as_builtin() {
                let builtin_provider = BuiltinProjectSourceProvider::new(builtin);
                let text = builtin_provider.source_text(id).ok();
                let source = builtin_provider.source_id(id).ok().map(|source_id| SourceLocation {
                    source_id,
                    display_path: PathBuf::from(format!("<builtin:{builtin}>/{}", id.path)),
                });
                (source, text)
            } else {
                (None, None)
            };
            modules.insert(
                id.clone(),
                CompiledModule {
                    id: id.clone(),
                    kind: linked_module.interface.kind,
                    source,
                    source_text: text,
                    interface: Arc::new(linked_module.interface.clone()),
                    plan: ModuleMaterializationPlan::empty(linked_module),
                    linked_reads: linked_module.linked_reads.clone(),
                },
            );
        }
        run_semantic_typecheck(&entry_id, &parsed.program)?;

        Ok(CompiledProgram {
            project_universe: universe,
            linked: linked.clone(),
            modules,
            entry: entry_id,
            initialization_order: linked.initialization_order.clone(),
        })
    }

    /// Discovers all transitively reachable modules, parses interfaces, links, and builds a `CompiledProgram`.
    fn discover_and_link(
        universe: Arc<ProjectUniverse>,
        source_provider: FilesystemSourceProvider,
        entry: ModuleId,
    ) -> Result<CompiledProgram, ProgramCompileError> {
        let mut resolver = ModuleResolver::new(&universe, &source_provider);
        let mut interfaces = BTreeMap::new();
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

        let mut source_locations = BTreeMap::new();
        let mut source_texts = BTreeMap::new();

        while let Some(current_id) = pending.pop() {
            let interface = resolver.load_interface(&current_id)?;
            interfaces.insert(current_id.clone(), interface.clone());

            if let Some(project_id) = current_id.project.as_resolved() {
                if let Some(proj) = universe.get_project(project_id) {
                    let mut curr_parent = current_id.path.parent();
                    while let Some(parent) = curr_parent {
                        let pkg_id = ModuleId::resolved(project_id, parent.clone());
                        if visited.insert(pkg_id.clone()) {
                            pending.push(pkg_id);
                        }
                        curr_parent = parent.parent();
                    }

                    if let Ok(unit) = source_provider.locate(proj, &current_id.path) {
                        if let Ok(text) = source_provider.read(&unit.source.source_id) {
                            source_texts.insert(current_id.clone(), text);
                        }
                        source_locations.insert(current_id.clone(), unit.source);
                    }
                }
            } else if let Some(builtin) = current_id.project.as_builtin() {
                let builtin_provider = BuiltinProjectSourceProvider::new(builtin);
                if let Ok(text) = builtin_provider.source_text(&current_id) {
                    source_texts.insert(current_id.clone(), text);
                }
                if let Ok(source_id) = builtin_provider.source_id(&current_id) {
                    let uri_path = if current_id.path.is_root() {
                        String::new()
                    } else {
                        current_id.path.components().iter().map(|c| c.as_str()).collect::<Vec<_>>().join("/")
                    };
                    source_locations.insert(
                        current_id.clone(),
                        SourceLocation {
                            source_id,
                            display_path: PathBuf::from(format!("<builtin:{builtin}>/{uri_path}")),
                        },
                    );
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

        let modules = linked
            .modules
            .iter()
            .map(|(id, module)| {
                let src = source_locations.get(id).cloned();
                let txt = source_texts.get(id).cloned();
                (id.clone(), compile_module(id.clone(), module, src, txt))
            })
            .collect();

        Ok(CompiledProgram {
            project_universe: universe,
            linked: linked.clone(),
            modules,
            entry,
            initialization_order: linked.initialization_order.clone(),
        })
    }

    /// Validates entry selection for an existing linked program.
    pub fn compile_entry(&self, entry: EntrySelection) -> Result<CompiledProgram, ProgramCompileError> {
        if let Some(linked) = &self.linked {
            let entry_id = match entry {
                EntrySelection::ModuleId(id) => id,
                _ => return Self::compile_entry_selection(entry),
            };
            if !linked.modules.contains_key(&entry_id) {
                return Err(ProgramCompileError::MissingEntry(entry_id));
            }
            let modules = linked
                .modules
                .iter()
                .map(|(id, module)| (id.clone(), compile_module(id.clone(), module, None, None)))
                .collect();
            Ok(CompiledProgram {
                project_universe: linked.universe.clone(),
                linked: linked.clone(),
                modules,
                entry: entry_id,
                initialization_order: linked.initialization_order.clone(),
            })
        } else {
            Self::compile_entry_selection(entry)
        }
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

pub fn run_semantic_typecheck(module_id: &ModuleId, program: &phalcom_ast::ast::Program) -> Result<(), ProgramCompileError> {
    use phalcom_semantic::{DeclarationId, MapTypeHierarchy, SimpleTypeResolver, TypeResolver, TypeStore};

    let mut store = TypeStore::new();
    let mut hierarchy = MapTypeHierarchy::new();
    let mut resolver = SimpleTypeResolver::new();

    let core_mod = ModuleId::core();
    let object_decl = DeclarationId::new(core_mod.clone(), "Object".into());
    resolver.insert("Object", object_decl.clone());

    for builtin in &[
        "Int", "Float", "String", "Bool", "Symbol", "Array", "Map", "Set", "Block", "Unit", "Never", "Dynamic",
    ] {
        let decl = DeclarationId::new(core_mod.clone(), (*builtin).into());
        hierarchy.insert(decl.clone(), object_decl.clone());
        resolver.insert(*builtin, decl);
    }

    for stmt in &program.statements {
        if let phalcom_ast::ast::Statement::Class(class_def) = stmt {
            let decl = DeclarationId::new(module_id.clone(), class_def.name.clone().into());
            resolver.insert(class_def.name.clone(), decl.clone());
            if let Some(super_ref) = &class_def.superclass {
                if let Some(super_decl) = resolver.resolve_type_name(module_id, &super_ref.root, &[]) {
                    hierarchy.insert(decl, super_decl);
                } else {
                    hierarchy.insert(decl, object_decl.clone());
                }
            } else {
                hierarchy.insert(decl, object_decl.clone());
            }
        }
    }

    let report = phalcom_semantic::check_program(&mut store, &hierarchy, &resolver, module_id.clone(), program);
    if report.has_errors() {
        return Err(ProgramCompileError::Type(report.diagnostics));
    }
    Ok(())
}
