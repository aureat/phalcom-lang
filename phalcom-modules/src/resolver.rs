use crate::error::{ModuleLoadError, ModuleResolutionError};
use crate::identity::{
    BuiltinProject, ImportRootTarget, ModuleComponent, ModuleId, ModulePath, ProjectIdentity,
};
use crate::interface::{InterfaceBuilder, PackagePathSurface, UnlinkedModuleInterface};
use crate::metadata::{MetadataTarget, ModuleMetadata};
use crate::project::ProjectUniverse;
use crate::source::{ModuleKind, ResolverGeneration, SourceProvider, SourceUnit};
use phalcom_ast::ast::{ImportPath, ImportRoot};
use std::collections::HashMap;

#[derive(Clone)]
struct CachedInterface {
    generation: ResolverGeneration,
    value: Result<UnlinkedModuleInterface, ModuleLoadError>,
}

pub struct ModuleResolver<'u, P: SourceProvider + ?Sized> {
    pub universe: &'u ProjectUniverse,
    pub source: &'u P,
    interface_cache: HashMap<ModuleId, CachedInterface>,
}

impl<'u, P: SourceProvider + ?Sized> ModuleResolver<'u, P> {
    pub fn new(universe: &'u ProjectUniverse, source: &'u P) -> Self {
        Self {
            universe,
            source,
            interface_cache: HashMap::new(),
        }
    }

    pub fn resolve_import(
        &mut self,
        importer: &ModuleId,
        syntax: &ImportPath,
    ) -> Result<SourceUnit, ModuleLoadError> {
        match &syntax.root {
            ImportRoot::Absolute(root_seg) => {
                let root_comp = ModuleComponent::from_identifier(&root_seg.name).map_err(|e| {
                    ModuleResolutionError::InvalidModuleName(root_seg.name.clone(), e)
                })?;
                let (target_identity, external_display, check_exposure) =
                    self.absolute_root_target(importer, &root_comp)?;

                let mut components = Vec::new();
                for seg in &syntax.segments {
                    components.push(ModuleComponent::from_identifier(&seg.name).map_err(|e| {
                        ModuleResolutionError::InvalidModuleName(seg.name.clone(), e)
                    })?);
                }
                let target_path = ModulePath::from_components(components);
                if check_exposure {
                    self.validate_external_path(
                        target_identity,
                        &external_display,
                        &target_path,
                    )?;
                }
                self.source.locate(&ModuleId {
                    project: target_identity,
                    path: target_path,
                })
            }
            ImportRoot::Relative { dots, range: _ } => {
                let dots = *dots as usize;
                if dots == 0 {
                    return Err(ModuleResolutionError::InvalidModuleLayout(
                        "Relative import must have at least one leading dot".to_string(),
                    )
                    .into());
                }

                let importer_unit = self.source.locate(importer)?;
                let package_path = match importer_unit.kind {
                    ModuleKind::Package => importer.path.clone(),
                    ModuleKind::Module => {
                        importer.path.parent().unwrap_or_else(ModulePath::root)
                    }
                };
                let pkg_components = package_path.components();
                let ascend_count = dots - 1;
                if ascend_count > pkg_components.len() {
                    return Err(ModuleResolutionError::RelativeImportBeyondRoot {
                        dots,
                        depth: pkg_components.len(),
                    }
                    .into());
                }

                let base_len = pkg_components.len() - ascend_count;
                let mut resolved_components = pkg_components[..base_len].to_vec();
                for seg in &syntax.segments {
                    resolved_components.push(
                        ModuleComponent::from_identifier(&seg.name).map_err(|e| {
                            ModuleResolutionError::InvalidModuleName(seg.name.clone(), e)
                        })?,
                    );
                }
                self.source.locate(&ModuleId {
                    project: importer.project,
                    path: ModulePath::from_components(resolved_components),
                })
            }
        }
    }

    fn absolute_root_target(
        &self,
        importer: &ModuleId,
        root: &ModuleComponent,
    ) -> Result<(ProjectIdentity, String, bool), ModuleLoadError> {
        match root.as_str() {
            "universe" | "core" => {
                return Ok((
                    ProjectIdentity::Builtin(BuiltinProject::Universe),
                    "universe".to_string(),
                    true,
                ));
            }
            "std" => {
                return Ok((
                    ProjectIdentity::Builtin(BuiltinProject::Std),
                    "std".to_string(),
                    true,
                ));
            }
            _ => {}
        }

        let ProjectIdentity::Resolved(importer_project_id) = importer.project else {
            return Err(ModuleResolutionError::UnknownImportRoot(root.as_str().to_string()).into());
        };
        let importer_project = self.universe.get_project(importer_project_id).ok_or_else(|| {
            ModuleResolutionError::ModuleNotFound(format!(
                "Project {importer_project_id} not found"
            ))
        })?;
        let (target, is_self) = importer_project
            .import_roots()
            .get(root)
            .copied()
            .ok_or_else(|| ModuleResolutionError::UnknownImportRoot(root.as_str().to_string()))?;

        match target {
            ImportRootTarget::Builtin(project) => Ok((
                ProjectIdentity::Builtin(project),
                project.root_name().to_string(),
                true,
            )),
            ImportRootTarget::Resolved(project_id) => {
                let display = self
                    .universe
                    .get_project(project_id)
                    .map(|p| p.name.clone())
                    .unwrap_or_else(|| project_id.to_string());
                Ok((ProjectIdentity::Resolved(project_id), display, !is_self))
            }
        }
    }

    pub fn validate_external_path(
        &mut self,
        target_project: ProjectIdentity,
        display_name: &str,
        path: &ModulePath,
    ) -> Result<(), ModuleLoadError> {
        if path.is_root() {
            return Ok(());
        }

        let mut current_pkg_path = ModulePath::root();
        for comp in path.components() {
            let surface = self.load_package_surface(target_project, &current_pkg_path)?;
            if !surface.exposed_children.contains(comp) {
                let exposed_names = surface
                    .exposed_children
                    .iter()
                    .map(|c| c.as_str().to_string())
                    .collect();
                return Err(ModuleResolutionError::ModulePathNotExposed {
                    path: path.to_string(),
                    project: display_name.to_string(),
                    exposed: exposed_names,
                }
                .into());
            }
            current_pkg_path = current_pkg_path.join(comp.clone());
        }
        Ok(())
    }

    pub fn load_package_surface(
        &mut self,
        project: ProjectIdentity,
        package_path: &ModulePath,
    ) -> Result<PackagePathSurface, ModuleLoadError> {
        let module_id = ModuleId {
            project,
            path: package_path.clone(),
        };
        let interface = self.load_interface(&module_id)?;
        if interface.kind != ModuleKind::Package {
            return Err(ModuleResolutionError::PackageNotFoundError(module_id.to_string()).into());
        }
        Ok(PackagePathSurface {
            exposed_children: interface.exposed_children.clone(),
        })
    }

    pub fn load_interface(
        &mut self,
        module_id: &ModuleId,
    ) -> Result<UnlinkedModuleInterface, ModuleLoadError> {
        let generation = self.source.generation();
        if let Some(cached) = self.interface_cache.get(module_id) {
            if cached.generation == generation {
                return cached.value.clone();
            }
        }

        let result = self.load_interface_uncached(module_id);
        self.interface_cache.insert(
            module_id.clone(),
            CachedInterface {
                generation,
                value: result.clone(),
            },
        );
        result
    }

    fn load_interface_uncached(
        &self,
        module_id: &ModuleId,
    ) -> Result<UnlinkedModuleInterface, ModuleLoadError> {
        let unit = self.source.locate(module_id)?;
        let source_text = self
            .source
            .read(&unit.source.source_id)
            .map_err(|error| ModuleLoadError::Io {
                module: Some(module_id.clone()),
                error,
            })?;
        let parse_result = phalcom_ast::parse(&source_text, 0);
        if let Some(error) = parse_result.errors.first() {
            return Err(ModuleLoadError::Parse {
                module: module_id.clone(),
                location: unit.source.clone(),
                error: error.clone(),
            });
        }

        let mut interface = InterfaceBuilder::build(
            module_id.clone(),
            unit.kind,
            &parse_result.program,
        )
        .map_err(|error| ModuleLoadError::Interface {
            module: module_id.clone(),
            error,
        })?;

        // The persistent project's root package is the Project object itself.
        // Metadata attaches once to that semantic owner rather than to a
        // duplicate package facet.
        if unit.kind == ModuleKind::Package
            && module_id.path.is_root()
            && matches!(module_id.project, ProjectIdentity::Resolved(_))
        {
            interface.metadata = ModuleMetadata::with_target(
                &parse_result.program.preamble.metadata,
                MetadataTarget::Project,
            );
        }
        Ok(interface)
    }
}
