//! Read-only queries over canonical module products.
//!
//! This facade deliberately owns no project lifecycle, revision counter,
//! source cache, or invalidation graph. A compiler semantic session can expose
//! its current `ProjectUniverse`, interfaces, resolutions, and provenance
//! through this view; LSP consumers can then query those products without
//! reconstructing module meaning.

use crate::identity::{ImportRootTarget, ModuleComponent, ModuleId, ModulePath, ProjectIdentity, SourceId, SourceLocation};
use crate::interface::{LinkedExport, LinkedModuleInterface, UnlinkedModuleInterface};
use crate::project::ProjectUniverse;
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

/// Query target for an import root with self-package distinction.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ImportRootQueryTarget {
    pub target: ImportRootTarget,
    pub is_self: bool,
}

/// Errors occurring during pure relative prefix resolution.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RelativeQueryError {
    InvalidDots,
    ImporterInterfaceMissing(ModuleId),
    BeyondRoot { dots: usize, depth: usize },
}

/// Immutable view of one canonical module-resolution generation.
pub struct ModuleQueryFacade<'a> {
    universe: &'a ProjectUniverse,
    unlinked: &'a BTreeMap<ModuleId, UnlinkedModuleInterface>,
    linked: &'a BTreeMap<ModuleId, LinkedModuleInterface>,
    resolved_imports: &'a BTreeMap<(ModuleId, String), ModuleId>,
    sources: &'a BTreeMap<ModuleId, SourceLocation>,
    source_modules: &'a BTreeMap<SourceId, ModuleId>,
    display_path_modules: &'a BTreeMap<std::path::PathBuf, ModuleId>,
}

impl<'a> ModuleQueryFacade<'a> {
    /// Creates a query view over already-produced canonical module products.
    pub fn new(
        universe: &'a ProjectUniverse,
        unlinked: &'a BTreeMap<ModuleId, UnlinkedModuleInterface>,
        linked: &'a BTreeMap<ModuleId, LinkedModuleInterface>,
        resolved_imports: &'a BTreeMap<(ModuleId, String), ModuleId>,
        sources: &'a BTreeMap<ModuleId, SourceLocation>,
        source_modules: &'a BTreeMap<SourceId, ModuleId>,
        display_path_modules: &'a BTreeMap<std::path::PathBuf, ModuleId>,
    ) -> Self {
        Self {
            universe,
            unlinked,
            linked,
            resolved_imports,
            sources,
            source_modules,
            display_path_modules,
        }
    }

    /// Returns canonical import root entries including `is_self` indicators.
    pub fn import_root_entries(&self, importer: &ModuleId) -> BTreeMap<ModuleComponent, ImportRootQueryTarget> {
        let mut roots = BTreeMap::from([(
            ModuleComponent::from_identifier("universe").expect("universe is canonical"),
            ImportRootQueryTarget {
                target: ImportRootTarget::Universe,
                is_self: false,
            },
        )]);

        if let ProjectIdentity::Resolved(project) = importer.project
            && let Some(project) = self.universe.get_project(project)
        {
            roots.extend(project.import_roots().iter().map(|(name, (target, is_self))| {
                (
                    name.clone(),
                    ImportRootQueryTarget {
                        target: *target,
                        is_self: *is_self,
                    },
                )
            }));
        }
        roots
    }

    /// Returns canonical import roots available to an importer.
    pub fn import_roots(&self, importer: &ModuleId) -> BTreeMap<ModuleComponent, ImportRootTarget> {
        self.import_root_entries(importer).into_iter().map(|(k, v)| (k, v.target)).collect()
    }

    /// Returns direct module children of a canonical project-relative prefix with no exposure filtering.
    pub fn module_children(&self, project: ProjectIdentity, prefix: &ModulePath) -> Vec<ModuleId> {
        let mut children = BTreeSet::new();
        for candidate in self.linked.keys().chain(self.unlinked.keys()) {
            if candidate.project == project {
                let components = candidate.path.components();
                if components.len() == prefix.components().len() + 1 && &components[..prefix.components().len()] == prefix.components() {
                    children.insert(candidate.clone());
                }
            }
        }
        children.into_iter().collect()
    }

    /// Returns direct module children of a canonical project-relative prefix with external hierarchical exposure filtering.
    pub fn external_import_children(&self, target_project: ProjectIdentity, prefix: &ModulePath) -> Vec<ModuleId> {
        let components = prefix.components();
        let mut current_pkg_path = ModulePath::root();

        for comp in components {
            let parent_id = ModuleId {
                project: target_project,
                path: current_pkg_path.clone(),
            };
            let Some(iface) = self.unlinked.get(&parent_id) else {
                return Vec::new();
            };
            if !iface.exposed_children.contains(comp) {
                return Vec::new();
            }
            current_pkg_path = current_pkg_path.join(comp.clone());
        }

        let current_parent_id = ModuleId {
            project: target_project,
            path: prefix.clone(),
        };
        let Some(current_iface) = self.unlinked.get(&current_parent_id) else {
            return Vec::new();
        };

        let mut result = Vec::new();
        for child in self.module_children(target_project, prefix) {
            if let Some(last_comp) = child.path.components().last() {
                if current_iface.exposed_children.contains(last_comp) {
                    result.push(child);
                }
            }
        }
        result
    }

    /// Pure relative prefix resolution matching canonical module resolver rules.
    pub fn resolve_relative_prefix(&self, importer: &ModuleId, dots: usize, suffix: &[ModuleComponent]) -> Result<ModulePath, RelativeQueryError> {
        if dots == 0 {
            return Err(RelativeQueryError::InvalidDots);
        }
        let Some(iface) = self.unlinked.get(importer) else {
            return Err(RelativeQueryError::ImporterInterfaceMissing(importer.clone()));
        };

        let package_path = match iface.kind {
            crate::source::ModuleKind::Package => importer.path.clone(),
            crate::source::ModuleKind::Module => importer.path.parent().unwrap_or_else(ModulePath::root),
        };

        let pkg_components = package_path.components();
        let ascend_count = dots - 1;

        if ascend_count > pkg_components.len() {
            return Err(RelativeQueryError::BeyondRoot {
                dots,
                depth: pkg_components.len(),
            });
        }

        let base_len = pkg_components.len() - ascend_count;
        let mut resolved_components = pkg_components[..base_len].to_vec();
        resolved_components.extend_from_slice(suffix);

        Ok(ModulePath::from_components(resolved_components))
    }

    /// Returns direct module children of a canonical project-relative prefix for any project.
    /// Package exposure is enforced from the source-owned unlinked interface.
    pub fn import_children_in_project(&self, project: ProjectIdentity, prefix: &ModulePath) -> Vec<ModuleId> {
        self.external_import_children(project, prefix)
    }

    /// Returns direct module children of a canonical project-relative prefix for an importer's project.
    pub fn import_children(&self, importer: &ModuleId, prefix: &ModulePath) -> Vec<ModuleId> {
        self.external_import_children(importer.project, prefix)
    }

    /// Returns linked public exports for a module.
    pub fn public_exports(&self, module: &ModuleId) -> Option<&BTreeMap<Box<str>, LinkedExport>> {
        self.linked.get(module).map(|interface| &interface.exports)
    }

    /// Returns a previously computed canonical resolution for an import path.
    /// The facade never guesses or derives logical meaning from URI spelling.
    pub fn resolved_import_target(&self, importer: &ModuleId, path: &str) -> Option<&ModuleId> {
        self.resolved_imports.get(&(importer.clone(), path.to_string()))
    }

    /// Returns canonical source provenance for a module definition.
    pub fn definition_source(&self, module: &ModuleId) -> Option<&SourceLocation> {
        self.sources.get(module)
    }

    /// Returns the canonical module for an exact source-provider identity.
    pub fn module_for_source(&self, source: &SourceId) -> Option<&ModuleId> {
        self.source_modules.get(source)
    }

    /// Returns the canonical module for an already-produced display path.
    ///
    /// This is a pure snapshot lookup. It does not canonicalize or read the
    /// filesystem.
    pub fn module_for_display_path(&self, path: &Path) -> Option<&ModuleId> {
        self.display_path_modules.get(path)
    }

    /// Returns importers that consumed a resolved module target.
    pub fn reverse_importers(&self, module: &ModuleId) -> Vec<ModuleId> {
        let mut importers = BTreeSet::new();
        for ((importer, _), target) in self.resolved_imports {
            if target == module {
                importers.insert(importer.clone());
            }
        }
        importers.into_iter().collect()
    }
}
