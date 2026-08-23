//! Read-only queries over canonical module products.
//!
//! This facade deliberately owns no project lifecycle, revision counter,
//! source cache, or invalidation graph. A compiler semantic session can expose
//! its current `ProjectUniverse`, interfaces, resolutions, and provenance
//! through this view; LSP consumers can then query those products without
//! reconstructing module meaning.

use crate::identity::{ImportRootTarget, ModuleComponent, ModuleId, ModulePath, ProjectIdentity, SourceLocation};
use crate::interface::{LinkedExport, LinkedModuleInterface, UnlinkedModuleInterface};
use crate::project::ProjectUniverse;
use std::collections::{BTreeMap, BTreeSet};

/// Immutable view of one canonical module-resolution generation.
pub struct ModuleQueryFacade<'a> {
    universe: &'a ProjectUniverse,
    unlinked: &'a BTreeMap<ModuleId, UnlinkedModuleInterface>,
    linked: &'a BTreeMap<ModuleId, LinkedModuleInterface>,
    resolved_imports: &'a BTreeMap<(ModuleId, String), ModuleId>,
    sources: &'a BTreeMap<ModuleId, SourceLocation>,
}

impl<'a> ModuleQueryFacade<'a> {
    /// Creates a query view over already-produced canonical module products.
    pub fn new(
        universe: &'a ProjectUniverse,
        unlinked: &'a BTreeMap<ModuleId, UnlinkedModuleInterface>,
        linked: &'a BTreeMap<ModuleId, LinkedModuleInterface>,
        resolved_imports: &'a BTreeMap<(ModuleId, String), ModuleId>,
        sources: &'a BTreeMap<ModuleId, SourceLocation>,
    ) -> Self {
        Self {
            universe,
            unlinked,
            linked,
            resolved_imports,
            sources,
        }
    }

    /// Returns canonical import roots available to an importer.
    pub fn import_roots(&self, importer: &ModuleId) -> BTreeMap<ModuleComponent, ImportRootTarget> {
        let mut roots = BTreeMap::from([
            (
                ModuleComponent::from_identifier("std").expect("std is canonical"),
                ImportRootTarget::Builtin(crate::identity::BuiltinProject::Std),
            ),
            (
                ModuleComponent::from_identifier("universe").expect("universe is canonical"),
                ImportRootTarget::Builtin(crate::identity::BuiltinProject::Universe),
            ),
        ]);

        if let ProjectIdentity::Resolved(project) = importer.project
            && let Some(project) = self.universe.get_project(project)
        {
            roots.extend(project.import_roots().iter().map(|(name, (target, _))| (name.clone(), *target)));
        }
        roots
    }

    /// Returns direct module children of a canonical project-relative prefix for any project.
    /// Package exposure is enforced from the source-owned unlinked interface.
    pub fn import_children_in_project(&self, project: ProjectIdentity, prefix: &ModulePath) -> Vec<ModuleId> {
        let parent = ModuleId {
            project,
            path: prefix.clone(),
        };
        let exposed = self.unlinked.get(&parent).map(|interface| &interface.exposed_children);

        self.linked
            .keys()
            .filter(|candidate| candidate.project == project)
            .filter_map(|candidate| {
                let components = candidate.path.components();
                if components.len() != prefix.components().len() + 1 || &components[..prefix.components().len()] != prefix.components() {
                    return None;
                }
                let child = components.last()?.clone();
                if exposed.is_some_and(|children| !children.contains(&child)) {
                    return None;
                }
                Some(candidate.clone())
            })
            .collect()
    }

    /// Returns direct module children of a canonical project-relative prefix for an importer's project.
    /// Package exposure is enforced from the source-owned unlinked interface.
    pub fn import_children(&self, importer: &ModuleId, prefix: &ModulePath) -> Vec<ModuleId> {
        self.import_children_in_project(importer.project, prefix)
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
