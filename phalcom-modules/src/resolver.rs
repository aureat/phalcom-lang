use crate::builtin::BuiltinProjectSourceProvider;
use crate::error::{ModuleLoadError, ModuleResolutionError};
use crate::identity::{ImportRootTarget, ModuleComponent, ModuleId, ModulePath, ResolvedProjectId, SourceId, SourceLocation};
use crate::interface::{InterfaceBuilder, PackagePathSurface, UnlinkedModuleInterface};
use crate::project::ProjectUniverse;
use crate::source::{ModuleKind, SourceProvider, SourceUnit};
use phalcom_ast::ast::{ImportPath, ImportRoot};
use std::collections::HashMap;
use std::path::PathBuf;

/// Module resolver coordinating `ProjectUniverse` and `SourceProvider`.
pub struct ModuleResolver<'u, P: SourceProvider> {
    pub universe: &'u ProjectUniverse,
    pub source: &'u P,
    interface_cache: HashMap<ModuleId, Result<UnlinkedModuleInterface, ModuleLoadError>>,
}

impl<'u, P: SourceProvider> ModuleResolver<'u, P> {
    pub fn new(universe: &'u ProjectUniverse, source: &'u P) -> Self {
        Self {
            universe,
            source,
            interface_cache: HashMap::new(),
        }
    }

    /// Resolves an AST `ImportPath` written inside the context of `importer`.
    pub fn resolve_import(&mut self, importer: &ModuleId, syntax: &ImportPath) -> Result<SourceUnit, ModuleResolutionError> {
        let importer_project_id = importer
            .project
            .as_resolved()
            .ok_or_else(|| ModuleResolutionError::ModuleNotFound(format!("{} has no resolved user-project source authority", importer.project)))?;
        let importer_project = self
            .universe
            .get_project(importer_project_id)
            .ok_or_else(|| ModuleResolutionError::ModuleNotFound(format!("Project {:?} not found", importer_project_id)))?;

        match &syntax.root {
            ImportRoot::Absolute(root_seg) => {
                if root_seg.name == "core" {
                    return Err(ModuleResolutionError::LegacyCoreImportRemoved);
                }
                let root_comp =
                    ModuleComponent::from_identifier(&root_seg.name).map_err(|e| ModuleResolutionError::InvalidModuleName(root_seg.name.clone(), e))?;

                let roots = importer_project.import_roots();
                let (target_root, is_self) = roots
                    .get(&root_comp)
                    .copied()
                    .ok_or_else(|| ModuleResolutionError::UnknownImportRoot(root_seg.name.clone()))?;

                // Build target relative path from segments before selecting a provider.
                let mut components = Vec::new();
                for seg in &syntax.segments {
                    let comp = ModuleComponent::from_identifier(&seg.name).map_err(|e| ModuleResolutionError::InvalidModuleName(seg.name.clone(), e))?;
                    components.push(comp);
                }
                let target_path = ModulePath::from_components(components);

                let target_project_id = match target_root {
                    ImportRootTarget::Builtin(builtin) => {
                        let uri_path = if target_path.is_root() {
                            String::new()
                        } else {
                            target_path.components().iter().map(|c| c.as_str()).collect::<Vec<_>>().join("/")
                        };
                        return Ok(SourceUnit {
                            id: ModuleId::builtin(builtin, target_path.clone()),
                            kind: if target_path.is_root() { ModuleKind::Package } else { ModuleKind::Module },
                            source: SourceLocation {
                                source_id: SourceId(format!("phalcom://{builtin}/{uri_path}").into()),
                                display_path: PathBuf::from(format!("<builtin:{builtin}>/{uri_path}")),
                            },
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
                    self.validate_external_path(target_project_id, &target_path)?;
                }

                self.source.locate(target_project, &target_path)
            }
            ImportRoot::Relative { dots, range: _ } => {
                let dots = *dots as usize;
                if dots == 0 {
                    return Err(ModuleResolutionError::InvalidModuleLayout(
                        "Relative import must have at least one leading dot".to_string(),
                    ));
                }

                // Determine importer package depth
                let importer_unit = self.source.locate(importer_project, &importer.path)?;
                let package_path = match importer_unit.kind {
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
                self.source.locate(importer_project, &target_path)
            }
        }
    }

    /// Validates that an external module path is exposed hierarchically by each intermediate package.
    pub fn validate_external_path(&mut self, target_project_id: ResolvedProjectId, path: &ModulePath) -> Result<(), ModuleResolutionError> {
        let components = path.components();
        // Root package `[]` is always addressable
        if components.is_empty() {
            return Ok(());
        }

        let target_project = self
            .universe
            .get_project(target_project_id)
            .ok_or_else(|| ModuleResolutionError::ModuleNotFound(format!("Target project {:?} not found", target_project_id)))?;

        // Hierarchical exposure check: start at root package `[]`
        let mut current_pkg_path = ModulePath::root();

        for comp in components {
            let surface = self.load_package_surface(target_project_id, &current_pkg_path)?;
            if !surface.exposed_children.contains(comp) {
                let exposed_names = surface.exposed_children.iter().map(|c| c.as_str().to_string()).collect();
                return Err(ModuleResolutionError::ModulePathNotExposed {
                    path: path.to_string(),
                    project: target_project.name.clone(),
                    exposed: exposed_names,
                });
            }
            current_pkg_path = current_pkg_path.join(comp.clone());
        }

        Ok(())
    }

    /// Loads package exposure surface for a given package module.
    pub fn load_package_surface(&mut self, project_id: ResolvedProjectId, package_path: &ModulePath) -> Result<PackagePathSurface, ModuleResolutionError> {
        let module_id = ModuleId {
            project: project_id.into(),
            path: package_path.clone(),
        };

        let interface = self
            .load_interface(&module_id)
            .map_err(|error| ModuleResolutionError::PackageSurface(Box::new(error)))?;

        if interface.kind != ModuleKind::Package {
            return Err(ModuleResolutionError::PackageNotFoundError(format!("{}", module_id)));
        }

        Ok(PackagePathSurface {
            exposed_children: interface.exposed_children.clone(),
        })
    }

    /// Loads and parses the unlinked interface of a module.
    pub fn load_interface(&mut self, module_id: &ModuleId) -> Result<UnlinkedModuleInterface, ModuleLoadError> {
        if let Some(res) = self.interface_cache.get(module_id) {
            return res.clone();
        }

        if let crate::identity::ProjectIdentity::Builtin(builtin) = module_id.project {
            let result = BuiltinProjectSourceProvider::new(builtin).load_interface(module_id);
            self.interface_cache.insert(module_id.clone(), result.clone());
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
            self.interface_cache.insert(module_id.clone(), Err(load_err.clone()));
            return Err(load_err);
        }

        let unlinked = match InterfaceBuilder::build(module_id.clone(), unit.kind, &parse_result.program) {
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
