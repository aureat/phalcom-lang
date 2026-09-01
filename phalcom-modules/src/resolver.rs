use crate::builtin::UniverseSourceProvider;
use crate::error::{ModuleLoadError, ModuleResolutionError};
use crate::identity::{ImportRootTarget, ModuleComponent, ModuleId, ModulePath, ProjectIdentity, ResolvedProjectId, SourceLocation};
use crate::interface::{InterfaceBuilder, PackagePathSurface, UnlinkedModuleInterface};
use crate::project::ProjectUniverse;
use crate::source::{ModuleKind, ParsedModuleUnit, SourceProvider, SourceUnit};
use phalcom_ast::ast::{ImportPath, ImportRoot};
use std::collections::{BTreeSet, HashMap};
use std::path::PathBuf;
use std::sync::Arc;

/// Traced import resolution record containing the target source unit and all package interfaces consulted.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ImportResolutionTrace {
    pub target: SourceUnit,
    pub package_interfaces: BTreeSet<ModuleId>,
}

/// Module resolver coordinating `ProjectUniverse` and `SourceProvider`.
pub struct ModuleResolver<'u, P: SourceProvider> {
    pub universe: &'u ProjectUniverse,
    pub source: &'u P,
    parsed_cache: HashMap<ModuleId, Result<Arc<ParsedModuleUnit>, ModuleLoadError>>,
    interface_cache: HashMap<ModuleId, Result<UnlinkedModuleInterface, ModuleLoadError>>,
}

impl<'u, P: SourceProvider> ModuleResolver<'u, P> {
    pub fn new(universe: &'u ProjectUniverse, source: &'u P) -> Self {
        Self {
            universe,
            source,
            parsed_cache: HashMap::new(),
            interface_cache: HashMap::new(),
        }
    }

    /// Resolves an AST `ImportPath` written inside the context of `importer`.
    pub fn resolve_import(&mut self, importer: &ModuleId, syntax: &ImportPath) -> Result<SourceUnit, ModuleResolutionError> {
        self.resolve_import_with_trace(importer, syntax).map(|trace| trace.target)
    }

    /// Resolves an AST `ImportPath` tracking all package exposure interfaces consulted during hierarchical validation.
    pub fn resolve_import_with_trace(&mut self, importer: &ModuleId, syntax: &ImportPath) -> Result<ImportResolutionTrace, ModuleResolutionError> {
        let mut package_interfaces = BTreeSet::new();
        let importer_project = match importer.project {
            crate::identity::ProjectIdentity::Resolved(pid) => self.universe.get_project(pid),
            _ => None,
        };

        match &syntax.root {
            ImportRoot::Absolute(root_seg) => {
                if root_seg.name == "core" {
                    return Err(ModuleResolutionError::LegacyCoreImportRemoved);
                }
                if root_seg.name == "std" {
                    return Err(ModuleResolutionError::LegacyStdImportRemoved);
                }
                let root_comp =
                    ModuleComponent::from_identifier(&root_seg.name).map_err(|e| ModuleResolutionError::InvalidModuleName(root_seg.name.clone(), e))?;

                let (target_root, is_self) = if root_seg.name == "universe" {
                    (ImportRootTarget::Universe, false)
                } else if let Some(proj) = importer_project {
                    let roots = proj.import_roots();
                    roots
                        .get(&root_comp)
                        .copied()
                        .ok_or_else(|| ModuleResolutionError::UnknownImportRoot(root_seg.name.clone()))?
                } else {
                    return Err(ModuleResolutionError::ModuleNotFound(format!(
                        "standalone module {} cannot import user dependency '{}' without a project context",
                        importer, root_seg.name
                    )));
                };

                // Build target relative path from segments before selecting a provider.
                let mut components = Vec::new();
                for seg in &syntax.segments {
                    let comp = ModuleComponent::from_identifier(&seg.name).map_err(|e| ModuleResolutionError::InvalidModuleName(seg.name.clone(), e))?;
                    components.push(comp);
                }
                let target_path = ModulePath::from_components(components);

                let target_project_id = match target_root {
                    ImportRootTarget::Universe => {
                        self.validate_path_with_trace(ProjectIdentity::Universe, &target_path, &mut package_interfaces)?;
                        let provider = UniverseSourceProvider::new();
                        let kind = provider
                            .kind(&target_path)
                            .ok_or_else(|| ModuleResolutionError::ModuleNotFound(format!("Universe module universe.{target_path} not found")))?;
                        let id = ModuleId::universe(target_path.clone());
                        let source_id = provider.source_id(&id).map_err(|e| match e {
                            ModuleLoadError::Resolution(r) => r,
                            _ => ModuleResolutionError::ModuleNotFound(format!("{e}")),
                        })?;
                        let uri_path = if target_path.is_root() {
                            String::new()
                        } else {
                            target_path.components().iter().map(|c| c.as_str()).collect::<Vec<_>>().join("/")
                        };
                        return Ok(ImportResolutionTrace {
                            target: SourceUnit {
                                id,
                                kind,
                                source: SourceLocation {
                                    source_id,
                                    display_path: PathBuf::from(format!("<universe>/{uri_path}")),
                                },
                            },
                            package_interfaces,
                        });
                    }
                    ImportRootTarget::Resolved(id) => id,
                };

                let target_project = self
                    .universe
                    .get_project(target_project_id)
                    .ok_or_else(|| ModuleResolutionError::ModuleNotFound(format!("Target project {:?} not found", target_project_id)))?;

                // If cross-project import, perform external path exposure check
                if !is_self {
                    self.validate_external_path_with_trace(target_project_id, &target_path, &mut package_interfaces)?;
                }

                let target = self.source.locate(target_project, &target_path)?;
                Ok(ImportResolutionTrace { target, package_interfaces })
            }
            ImportRoot::Relative { dots, range: _ } => {
                let dots = *dots as usize;
                if dots == 0 {
                    return Err(ModuleResolutionError::InvalidModuleLayout(
                        "Relative import must have at least one leading dot".to_string(),
                    ));
                }

                // Determine importer package depth
                let importer_kind = self
                    .load_parsed(importer)
                    .map_err(|error| ModuleResolutionError::ModuleNotFound(format!("cannot load importer {importer}: {error}")))?
                    .kind;
                let package_path = match importer_kind {
                    ModuleKind::Package => importer.path.clone(),
                    ModuleKind::Module => importer.path.parent().unwrap_or_else(ModulePath::root),
                };

                let pkg_components = package_path.components();
                let ascend_count = dots - 1;

                if ascend_count > pkg_components.len() {
                    return Err(ModuleResolutionError::RelativeImportBeyondRoot {
                        dots,
                        depth: pkg_components.len(),
                    });
                }

                let base_len = pkg_components.len() - ascend_count;
                let mut resolved_components = pkg_components[..base_len].to_vec();

                for seg in &syntax.segments {
                    let comp = ModuleComponent::from_identifier(&seg.name).map_err(|e| ModuleResolutionError::InvalidModuleName(seg.name.clone(), e))?;
                    resolved_components.push(comp);
                }

                let target_path = ModulePath::from_components(resolved_components);
                let target_project = importer.project;
                if !matches!(target_project, ProjectIdentity::Universe | ProjectIdentity::Resolved(_)) {
                    return Err(ModuleResolutionError::ModuleNotFound(format!(
                        "standalone module {} cannot perform relative imports without a project context",
                        importer
                    )));
                }
                let target = self.locate_project_module(target_project, &target_path)?;
                Ok(ImportResolutionTrace { target, package_interfaces })
            }
        }
    }

    /// Validates that an external module path is exposed hierarchically by each intermediate package.
    pub fn validate_external_path(&mut self, target_project_id: ResolvedProjectId, path: &ModulePath) -> Result<(), ModuleResolutionError> {
        let mut trace = BTreeSet::new();
        self.validate_external_path_with_trace(target_project_id, path, &mut trace)
    }

    /// Validates external module path recording all package interfaces consulted.
    pub fn validate_external_path_with_trace(
        &mut self,
        target_project_id: ResolvedProjectId,
        path: &ModulePath,
        package_interfaces: &mut BTreeSet<ModuleId>,
    ) -> Result<(), ModuleResolutionError> {
        self.validate_path_with_trace(ProjectIdentity::Resolved(target_project_id), path, package_interfaces)
    }

    /// Validates hierarchical package exposure for either a filesystem project
    /// or the provider-backed Universe. Provider choice must not change import
    /// visibility semantics.
    fn validate_path_with_trace(
        &mut self,
        target_project: ProjectIdentity,
        path: &ModulePath,
        package_interfaces: &mut BTreeSet<ModuleId>,
    ) -> Result<(), ModuleResolutionError> {
        let components = path.components();
        // Root package `[]` is always addressable
        if components.is_empty() {
            return Ok(());
        }

        let target_name = match target_project {
            ProjectIdentity::Universe => "universe".to_owned(),
            ProjectIdentity::Resolved(project_id) => self
                .universe
                .get_project(project_id)
                .map(|project| project.name.clone())
                .ok_or_else(|| ModuleResolutionError::ModuleNotFound(format!("Target project {:?} not found", project_id)))?,
            ProjectIdentity::Synthetic(project_id) => {
                return Err(ModuleResolutionError::ModuleNotFound(format!(
                    "synthetic project {project_id} has no import provider"
                )));
            }
        };

        // Hierarchical exposure check: start at root package `[]`
        let mut current_pkg_path = ModulePath::root();

        for comp in components {
            let pkg_mod_id = ModuleId {
                project: target_project,
                path: current_pkg_path.clone(),
            };
            package_interfaces.insert(pkg_mod_id);
            let surface = self.load_package_surface_for(target_project, &current_pkg_path)?;
            if !surface.exposed_children.contains(comp) {
                let exposed_names = surface.exposed_children.iter().map(|c| c.as_str().to_string()).collect();
                return Err(ModuleResolutionError::ModulePathNotExposed {
                    path: path.to_string(),
                    project: target_name.clone(),
                    exposed: exposed_names,
                });
            }
            current_pkg_path = current_pkg_path.join(comp.clone());
        }

        Ok(())
    }

    /// Loads package exposure surface for a given package module.
    pub fn load_package_surface(&mut self, project_id: ResolvedProjectId, package_path: &ModulePath) -> Result<PackagePathSurface, ModuleResolutionError> {
        self.load_package_surface_for(ProjectIdentity::Resolved(project_id), package_path)
    }

    fn load_package_surface_for(&mut self, project: ProjectIdentity, package_path: &ModulePath) -> Result<PackagePathSurface, ModuleResolutionError> {
        let module_id = ModuleId {
            project,
            path: package_path.clone(),
        };

        let interface = self
            .load_interface(&module_id)
            .map_err(|error| ModuleResolutionError::PackageSurface(Box::new(error)))?;

        if !interface.kind.is_package_like() {
            return Err(ModuleResolutionError::PackageNotFoundError(format!("{}", module_id)));
        }

        Ok(PackagePathSurface {
            exposed_children: interface.exposed_children.clone(),
        })
    }

    fn locate_project_module(&self, project: ProjectIdentity, path: &ModulePath) -> Result<SourceUnit, ModuleResolutionError> {
        let module_id = ModuleId { project, path: path.clone() };
        match project {
            ProjectIdentity::Universe => {
                let provider = UniverseSourceProvider::new();
                let kind = provider
                    .kind(path)
                    .ok_or_else(|| ModuleResolutionError::ModuleNotFound(format!("Universe module universe.{path} not found")))?;
                let source_id = provider.source_id(&module_id).map_err(|error| match error {
                    ModuleLoadError::Resolution(resolution) => resolution,
                    other => ModuleResolutionError::ModuleNotFound(format!("{other}")),
                })?;
                Ok(SourceUnit {
                    id: module_id,
                    kind,
                    source: SourceLocation {
                        source_id,
                        display_path: PathBuf::from(format!(
                            "<universe>/{}",
                            path.components().iter().map(|c| c.as_str()).collect::<Vec<_>>().join("/")
                        )),
                    },
                })
            }
            ProjectIdentity::Resolved(project_id) => {
                let project = self
                    .universe
                    .get_project(project_id)
                    .ok_or_else(|| ModuleResolutionError::ModuleNotFound(format!("Target project {:?} not found", project_id)))?;
                self.source.locate(project, path)
            }
            ProjectIdentity::Synthetic(project_id) => Err(ModuleResolutionError::ModuleNotFound(format!(
                "synthetic project {project_id} has no import provider"
            ))),
        }
    }

    /// Loads and parses a module unit, caching the parsed AST and source artifact.
    pub fn load_parsed(&mut self, module_id: &ModuleId) -> Result<Arc<ParsedModuleUnit>, ModuleLoadError> {
        if let Some(res) = self.parsed_cache.get(module_id) {
            return res.clone();
        }

        if module_id.project == ProjectIdentity::Universe {
            let result = UniverseSourceProvider::new().load_parsed(module_id);
            self.parsed_cache.insert(module_id.clone(), result.clone());
            return result;
        }

        let project_id = module_id
            .project
            .as_resolved()
            .ok_or_else(|| ModuleResolutionError::ModuleNotFound(format!("{} is not filesystem-backed by this provider", module_id.project)))?;
        let project = self
            .universe
            .get_project(project_id)
            .ok_or_else(|| ModuleResolutionError::ModuleNotFound(format!("Project {:?} not found", project_id)))?;

        let unit = self.source.locate(project, &module_id.path)?;
        let source_text = self.source.read(&unit.source.source_id).map_err(ModuleResolutionError::Source)?;

        let parse_result = phalcom_ast::parse(&source_text, 0);
        if !parse_result.errors.is_empty() {
            let err = &parse_result.errors[0];
            let load_err = ModuleLoadError::Parse {
                module: module_id.clone(),
                source: unit.source.display_path.clone(),
                error: err.clone(),
            };
            self.parsed_cache.insert(module_id.clone(), Err(load_err.clone()));
            return Err(load_err);
        }

        let parsed = Arc::new(ParsedModuleUnit {
            id: module_id.clone(),
            kind: unit.kind,
            source: Some(unit.source),
            text: source_text,
            program: Arc::new(parse_result.program),
        });

        self.parsed_cache.insert(module_id.clone(), Ok(parsed.clone()));
        Ok(parsed)
    }

    /// Loads and parses the unlinked interface of a module.
    pub fn load_interface(&mut self, module_id: &ModuleId) -> Result<UnlinkedModuleInterface, ModuleLoadError> {
        if let Some(res) = self.interface_cache.get(module_id) {
            return res.clone();
        }

        let parsed = self.load_parsed(module_id)?;

        if module_id.project == ProjectIdentity::Universe {
            let provider = UniverseSourceProvider::new();
            let result = crate::builtin_interface::BuiltinInterfaceBuilder::build_from_parsed(&provider, &parsed);
            self.interface_cache.insert(module_id.clone(), result.clone());
            return result;
        }

        let unlinked = match InterfaceBuilder::build(module_id.clone(), parsed.kind, &parsed.program) {
            Ok(u) => u,
            Err(e) => {
                let load_err = ModuleLoadError::Interface {
                    module: module_id.clone(),
                    error: e,
                };
                self.interface_cache.insert(module_id.clone(), Err(load_err.clone()));
                return Err(load_err);
            }
        };

        self.interface_cache.insert(module_id.clone(), Ok(unlinked.clone()));
        Ok(unlinked)
    }
}
