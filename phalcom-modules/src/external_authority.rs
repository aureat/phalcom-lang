//! Exact immutable module authority over one or more completed artifacts.
//!
//! Module consumers borrow native interfaces and linked exports. Runtime
//! initialization schedules remain with each artifact's executable authority.

use crate::{CanonicalExternalModuleWorld, LinkedModule, ModuleId, ProjectIdentity, ProjectUniverse, UnlinkedModuleInterface};
use std::collections::BTreeMap;
use std::fmt::Debug;
use std::sync::Arc;

/// Source-free module products, independent of the artifact's package policy.
pub trait ExternalModuleAuthority: Debug + Send + Sync {
    /// Project catalog in which all native module identities were relocated.
    fn projects(&self) -> &Arc<ProjectUniverse>;
    /// Exact module identities in this authority.
    fn module_ids(&self) -> Box<dyn Iterator<Item = &ModuleId> + '_>;
    /// Whether this immutable base owns the project's complete module surface.
    fn owns_project(&self, project: ProjectIdentity) -> bool;
    /// Exact unlinked interface; absent rows must never trigger source fallback.
    fn interface(&self, module: &ModuleId) -> Option<&Arc<UnlinkedModuleInterface>>;
    /// Exact linked exports, without scheduling the dependency for local linking.
    fn linked_module(&self, module: &ModuleId) -> Option<&LinkedModule>;
    /// Whether this module belongs to the external base.
    fn contains(&self, module: &ModuleId) -> bool {
        self.interface(module).is_some()
    }
}

impl ExternalModuleAuthority for CanonicalExternalModuleWorld {
    fn projects(&self) -> &Arc<ProjectUniverse> { &self.linked().universe }
    fn module_ids(&self) -> Box<dyn Iterator<Item = &ModuleId> + '_> { Box::new(self.interfaces().keys()) }
    fn owns_project(&self, project: ProjectIdentity) -> bool { self.interfaces().keys().any(|module| module.project == project) }
    fn interface(&self, module: &ModuleId) -> Option<&Arc<UnlinkedModuleInterface>> { self.interfaces().get(module) }
    fn linked_module(&self, module: &ModuleId) -> Option<&LinkedModule> { self.linked().modules.get(module) }
}

impl<T: ExternalModuleAuthority + ?Sized> ExternalModuleAuthority for Arc<T> {
    fn projects(&self) -> &Arc<ProjectUniverse> { (**self).projects() }
    fn module_ids(&self) -> Box<dyn Iterator<Item = &ModuleId> + '_> { (**self).module_ids() }
    fn owns_project(&self, project: ProjectIdentity) -> bool { (**self).owns_project(project) }
    fn interface(&self, module: &ModuleId) -> Option<&Arc<UnlinkedModuleInterface>> { (**self).interface(module) }
    fn linked_module(&self, module: &ModuleId) -> Option<&LinkedModule> { (**self).linked_module(module) }
}

/// Set-local origin index. This is provenance, not a persistent artifact identity.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ExternalModuleOrigin(usize);

impl ExternalModuleOrigin {
    /// Stable index of the owning artifact within the composed authority set.
    pub fn index(self) -> usize { self.0 }
}

/// A closed immutable collection of completed native module worlds.
///
/// No products are copied into mutable source maps. Construction checks every
/// ownership collision before returning an authority that can be published.
#[derive(Debug)]
pub struct ExternalModuleAuthoritySet {
    projects: Arc<ProjectUniverse>,
    worlds: Vec<Arc<CanonicalExternalModuleWorld>>,
    owners: BTreeMap<ModuleId, ExternalModuleOrigin>,
    project_owners: BTreeMap<ProjectIdentity, ExternalModuleOrigin>,
}

impl ExternalModuleAuthoritySet {
    /// Composes worlds already relocated into a compatible caller-owned catalog.
    /// A project belongs to exactly one artifact; embedded duplicate ownership
    /// is rejected even when the products happen to compare equal.
    pub fn from_worlds(projects: Arc<ProjectUniverse>, worlds: Vec<Arc<CanonicalExternalModuleWorld>>) -> Result<Self, String> {
        let mut owners = BTreeMap::new();
        let mut project_owners = BTreeMap::new();
        for (index, world) in worlds.iter().enumerate() {
            if !projects.preserves_authority_of(world.projects()) {
                return Err("external module world uses different project bindings or package receipts".into());
            }
            let origin = ExternalModuleOrigin(index);
            for (module, interface) in world.interfaces().iter() {
                if interface.id != *module {
                    return Err(format!("external interface identity disagrees with its owner: {module}"));
                }
                if matches!(module.project, ProjectIdentity::Synthetic(_)) {
                    return Err(format!("session-only project cannot own persistent artifact modules: {module}"));
                }
                if let ProjectIdentity::Resolved(project) = module.project {
                    if !projects.get_project(project).is_some_and(|project| project.persistent_project) {
                        return Err(format!("external module has no persistent project binding: {module}"));
                    }
                }
                if owners.insert(module.clone(), origin).is_some() {
                    return Err(format!("duplicate external module ownership: {module}"));
                }
                if let Some(prior) = project_owners.insert(module.project, origin) {
                    if prior != origin {
                        return Err(format!("project is owned by multiple external artifacts: {}", module.project));
                    }
                }
            }
        }
        Ok(Self { projects, worlds, owners, project_owners })
    }

    /// Returns the exact artifact origin within this authority set.
    pub fn origin(&self, module: &ModuleId) -> Option<ExternalModuleOrigin> { self.owners.get(module).copied() }
}

impl ExternalModuleAuthority for ExternalModuleAuthoritySet {
    fn projects(&self) -> &Arc<ProjectUniverse> { &self.projects }
    fn module_ids(&self) -> Box<dyn Iterator<Item = &ModuleId> + '_> { Box::new(self.owners.keys()) }
    fn owns_project(&self, project: ProjectIdentity) -> bool { self.project_owners.contains_key(&project) }
    fn interface(&self, module: &ModuleId) -> Option<&Arc<UnlinkedModuleInterface>> {
        self.worlds.get(self.origin(module)?.0)?.interface(module)
    }
    fn linked_module(&self, module: &ModuleId) -> Option<&LinkedModule> {
        self.worlds.get(self.origin(module)?.0)?.linked_module(module)
    }
}
