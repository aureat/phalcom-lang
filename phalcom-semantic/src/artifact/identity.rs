//! Exact stable-owner authority for one validated compiler artifact.

use crate::identity::{CallableId, CallableOwnerId, DeclarationId, DispatchSide, ImplId, ImplLocalId, ModuleId, VariantId};
use phalcom_common::selector::Selector;
use phalcom_compiler_artifact::module_world_v5::ModuleWorldV5;
use phalcom_compiler_artifact::{ArtifactError, ValidationError};
use phalcom_modules::ProjectUniverse;
use phalcom_modules::identity::{ModuleComponent, ModulePath};
use phalcom_type_meta::allocation::{AllocationBudget, AllocationError};
use phalcom_type_meta::identity::{
    StableCallableOwnerRef, StableCallableRef, StableDeclarationRef, StableDispatchSide, StableImplRef, StableModuleRef, StableProjectRef, StableVariantRef,
};

#[cfg(test)]
pub(crate) fn empty_owner_inputs() -> (ModuleWorldV5, ProjectUniverse) {
    (
        phalcom_compiler_artifact::module_world::ModuleWorldV2 {
            entry: phalcom_compiler_artifact::module_world::ArtifactModuleId(0),
            initialization_order: Vec::new(),
            modules: Vec::new(),
            semantic_rows: Vec::new(),
            external_projects: Vec::new(),
            topology_fingerprint: 0,
        }
        .into(),
        ProjectUniverse::new(),
    )
}

/// Resolves artifact owners only through module rows or the explicit external
/// project allowlist and the caller's already populated project universe.
pub struct StableOwnerCatalog<'a> {
    module_world: &'a ModuleWorldV5,
    projects: &'a ProjectUniverse,
}

impl<'a> StableOwnerCatalog<'a> {
    pub fn new(module_world: &'a ModuleWorldV5, projects: &'a ProjectUniverse) -> Result<Self, ArtifactError> {
        for external in &module_world.external_projects {
            if projects.resolve_stable_project(external).is_none() {
                return Err(referential("external project has no exact persistent workspace binding"));
            }
        }
        Ok(Self { module_world, projects })
    }

    pub(crate) fn module(&self, stable: &StableModuleRef) -> Result<ModuleId, ArtifactError> {
        self.module_parts(&stable.project, &stable.path)
    }

    pub(crate) fn module_parts(&self, project_ref: &StableProjectRef, path: &[Box<str>]) -> Result<ModuleId, ArtifactError> {
        let project = self.project_for_module_parts(project_ref, path)?;
        let components = path
            .iter()
            .map(|segment| ModuleComponent::from_identifier(segment).map_err(|_| referential("invalid stable module path component")))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(ModuleId {
            project,
            path: ModulePath::from_components(components),
        })
    }

    pub(crate) fn module_parts_with_allocation_budget(
        &self,
        project_ref: &StableProjectRef,
        path: &[Box<str>],
        allocations: &mut AllocationBudget,
    ) -> Result<ModuleId, ArtifactError> {
        let project = self.project_for_module_parts(project_ref, path)?;
        let mut components = allocations.vector::<ModuleComponent>(path.len()).map_err(allocation_error)?;
        for segment in path {
            let segment = allocations.boxed_str(segment).map_err(allocation_error)?;
            components.push(ModuleComponent::from_owned_identifier(segment).map_err(|_| referential("invalid stable module path component"))?);
        }
        Ok(ModuleId {
            project,
            path: ModulePath::from_components(allocations.boxed_slice(components).map_err(allocation_error)?),
        })
    }

    fn project_for_module_parts(&self, project_ref: &StableProjectRef, path: &[Box<str>]) -> Result<crate::identity::ProjectIdentity, ArtifactError> {
        if matches!(project_ref, StableProjectRef::Session { .. }) {
            return Err(referential("session project owner is forbidden"));
        }

        let project_is_internal = self.module_world.modules.iter().any(|module| module.identity.project == *project_ref);
        if project_is_internal {
            if !self
                .module_world
                .modules
                .iter()
                .any(|module| module.identity.project == *project_ref && module.identity.path.as_ref() == path)
            {
                return Err(referential("internal stable module owner is absent from the module world"));
            }
        } else if !self.module_world.external_projects.contains(project_ref) {
            return Err(referential("stable module owner project is neither internal nor allowlisted"));
        }

        let project = self
            .projects
            .resolve_stable_project(project_ref)
            .ok_or_else(|| referential("stable project identity has no exact persistent workspace binding"))?;
        Ok(project)
    }

    pub(crate) fn declaration(&self, stable: &StableDeclarationRef) -> Result<DeclarationId, ArtifactError> {
        if stable.path.len() != 1 {
            return Err(referential("stable declaration owner is not a single canonical path component"));
        }
        Ok(DeclarationId::new(self.module(&stable.module)?, stable.path[0].clone()))
    }

    pub(crate) fn declaration_with_allocation_budget(
        &self,
        stable: &StableDeclarationRef,
        allocations: &mut AllocationBudget,
    ) -> Result<DeclarationId, ArtifactError> {
        if stable.path.len() != 1 {
            return Err(referential("stable declaration owner is not a single canonical path component"));
        }
        let module = self.module_parts_with_allocation_budget(&stable.module.project, &stable.module.path, allocations)?;
        let name = allocations.boxed_str(&stable.path[0]).map_err(allocation_error)?;
        Ok(DeclarationId::new(module, name))
    }

    pub(crate) fn variant(&self, stable: &StableVariantRef) -> Result<VariantId, ArtifactError> {
        Ok(VariantId::new(self.declaration(&stable.owner)?, canonical_selector(&stable.selector)?))
    }

    pub(crate) fn implementation(&self, stable: &StableImplRef) -> Result<ImplId, ArtifactError> {
        Ok(ImplId::new(self.module(&stable.module)?, ImplLocalId(stable.local_id)))
    }

    pub(crate) fn callable(&self, stable: &StableCallableRef) -> Result<CallableId, ArtifactError> {
        let owner = match &stable.owner {
            StableCallableOwnerRef::Declaration(owner) => CallableOwnerId::Declaration(self.declaration(owner)?),
            StableCallableOwnerRef::Variant(owner) => CallableOwnerId::Variant(self.variant(owner)?),
            StableCallableOwnerRef::Conformance(owner) => CallableOwnerId::Conformance(self.implementation(owner)?),
        };
        Ok(CallableId::new(owner, canonical_selector(&stable.selector)?, dispatch_side(stable.side)))
    }

    /// Resolve a query callable against exact artifact owner authority while
    /// charging its native identity and selector allocations to the caller.
    pub(crate) fn callable_with_allocation_budget(
        &self,
        stable: &StableCallableRef,
        allocations: &mut AllocationBudget,
    ) -> Result<CallableId, ArtifactError> {
        let owner = match &stable.owner {
            StableCallableOwnerRef::Declaration(declaration) => {
                CallableOwnerId::Declaration(self.declaration_with_allocation_budget(declaration, allocations)?)
            }
            StableCallableOwnerRef::Variant(variant) => {
                let declaration = self.declaration_with_allocation_budget(&variant.owner, allocations)?;
                let selector = crate::artifact::import::import_stable_selector_with_allocation_budget(&variant.selector, allocations)?;
                CallableOwnerId::Variant(VariantId::new(declaration, selector))
            }
            StableCallableOwnerRef::Conformance(implementation) => {
                let module = self.module_parts_with_allocation_budget(
                    &implementation.module.project, &implementation.module.path, allocations,
                )?;
                CallableOwnerId::Conformance(ImplId::new(module, ImplLocalId(implementation.local_id)))
            }
        };
        let selector = crate::artifact::import::import_stable_selector_with_allocation_budget(&stable.selector, allocations)?;
        Ok(CallableId::new(owner, selector, dispatch_side(stable.side)))
    }
}

pub(crate) fn dispatch_side(side: StableDispatchSide) -> DispatchSide {
    match side {
        StableDispatchSide::Instance => DispatchSide::Instance,
        StableDispatchSide::Class => DispatchSide::Class,
    }
}

pub(crate) fn canonical_selector(encoded: &str) -> Result<Selector, ArtifactError> {
    let selector = Selector::decode(encoded);
    if selector.encode() != encoded {
        return Err(referential("stable selector is not canonical"));
    }
    Ok(selector)
}

fn referential(message: impl Into<String>) -> ArtifactError {
    ArtifactError::Validation(ValidationError::ReferentialIntegrity(message.into()))
}

fn allocation_error(error: AllocationError) -> ArtifactError {
    ArtifactError::Allocation(error)
}

/// Exact owner authority made ambient for the nested import families that
/// still take no catalog parameter (`LD-C4`).
///
/// Package semantic rows nest stable owners inside parameter owners, `Self`
/// terms, trait references, callable owners and associated members. Those
/// helpers historically resolved through the Universe-only `from_stable_*`
/// functions. While an [`AmbientOwnerScope`] is live, the `scoped_stable_*`
/// resolvers below consult the artifact's exact [`StableOwnerCatalog`] first
/// and fall back to Universe-only resolution only when the catalog refuses.
pub(crate) mod ambient {
    use super::*;
    use std::cell::RefCell;
    use std::rc::Rc;

    type OwnedOwners = Rc<(ModuleWorldV5, ProjectUniverse)>;

    thread_local! {
        static ACTIVE: RefCell<Vec<OwnedOwners>> = const { RefCell::new(Vec::new()) };
    }

    /// Guard installing one artifact's owner catalog for the current thread.
    pub(crate) struct AmbientOwnerScope(());

    impl AmbientOwnerScope {
        /// Installs `module_world` + `projects` until the guard drops.
        pub(crate) fn enter(module_world: &ModuleWorldV5, projects: &ProjectUniverse) -> Self {
            ACTIVE.with(|active| active.borrow_mut().push(Rc::new((module_world.clone(), projects.clone()))));
            Self(())
        }
    }

    impl Drop for AmbientOwnerScope {
        fn drop(&mut self) {
            ACTIVE.with(|active| {
                active.borrow_mut().pop();
            });
        }
    }

    fn with_catalog<T>(resolve: impl FnOnce(&StableOwnerCatalog<'_>) -> Result<T, ArtifactError>) -> Option<Result<T, String>> {
        let owners = ACTIVE.with(|active| active.borrow().last().cloned())?;
        let catalog = match StableOwnerCatalog::new(&owners.0, &owners.1) {
            Ok(catalog) => catalog,
            Err(error) => return Some(Err(error.to_string())),
        };
        Some(resolve(&catalog).map_err(|error| error.to_string()))
    }

    fn scoped<T>(
        catalog: impl FnOnce(&StableOwnerCatalog<'_>) -> Result<T, ArtifactError>,
        universe: impl FnOnce() -> Result<T, String>,
    ) -> Result<T, String> {
        match with_catalog(catalog) {
            Some(Ok(value)) => Ok(value),
            Some(Err(error)) => universe().map_err(|_| error),
            None => universe(),
        }
    }

    /// Resolves a stable module through the ambient catalog, else Universe.
    pub(crate) fn scoped_stable_module(stable: &StableModuleRef) -> Result<ModuleId, String> {
        scoped(|catalog| catalog.module(stable), || crate::metadata::stable_identity::from_stable_module(stable))
    }

    /// Resolves a stable project through the ambient catalog, else Universe.
    pub(crate) fn scoped_stable_project(stable: &StableProjectRef) -> Result<crate::identity::ProjectIdentity, String> {
        scoped(
            |catalog| {
                if catalog.module_world.modules.iter().any(|module| module.identity.project == *stable)
                    || catalog.module_world.external_projects.contains(stable)
                {
                    catalog
                        .projects
                        .resolve_stable_project(stable)
                        .ok_or_else(|| referential("stable project identity has no exact persistent workspace binding"))
                } else {
                    Err(referential("stable project is neither internal nor allowlisted"))
                }
            },
            || crate::metadata::stable_identity::from_stable_project(stable),
        )
    }

    /// Resolves a stable declaration through the ambient catalog, else Universe.
    pub(crate) fn scoped_stable_declaration(stable: &StableDeclarationRef) -> Result<DeclarationId, String> {
        scoped(|catalog| catalog.declaration(stable), || crate::metadata::stable_identity::from_stable_declaration(stable))
    }

    /// Resolves a stable variant through the ambient catalog, else Universe.
    pub(crate) fn scoped_stable_variant(stable: &StableVariantRef) -> Result<VariantId, String> {
        scoped(|catalog| catalog.variant(stable), || crate::metadata::stable_identity::from_stable_variant(stable))
    }

    /// Resolves a stable implementation through the ambient catalog, else Universe.
    pub(crate) fn scoped_stable_impl(stable: &StableImplRef) -> Result<ImplId, String> {
        scoped(|catalog| catalog.implementation(stable), || crate::metadata::stable_identity::from_stable_impl(stable))
    }

    /// Resolves a stable callable through the ambient catalog, else Universe.
    pub(crate) fn scoped_stable_callable(stable: &StableCallableRef) -> Result<CallableId, String> {
        scoped(|catalog| catalog.callable(stable), || crate::metadata::stable_identity::from_stable_callable(stable))
    }
}
