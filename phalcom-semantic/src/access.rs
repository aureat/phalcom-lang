//! Canonical declaration access provenance and authorization.

use crate::identity::{CallableId, DeclarationId, ModuleId};
use crate::types::relation::TypeHierarchy;
use phalcom_common::modifier::Visibility;
use phalcom_modules::ProjectIdentity;

/// Exact declaration provenance for an ordinary module-backed declaration.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DeclarationAccessOrigin {
    pub module: ModuleId,
    pub package: ModuleId,
    pub project: ProjectIdentity,
}

/// Canonical caller provenance used by checker, editor, compiler, and runtime projections.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AccessContext {
    pub enclosing_declaration: Option<DeclarationId>,
    pub enclosing_callable: Option<CallableId>,
    pub caller_module: Option<ModuleId>,
    pub caller_package: Option<ModuleId>,
    pub caller_project: Option<ProjectIdentity>,
}

impl AccessContext {
    pub fn new(enclosing_declaration: Option<DeclarationId>, enclosing_callable: Option<CallableId>) -> Self {
        let caller_module = enclosing_declaration.as_ref().map(|declaration| declaration.module.clone());
        let caller_project = caller_module.as_ref().map(|module| module.project);
        Self {
            enclosing_declaration,
            enclosing_callable,
            caller_module,
            caller_package: None,
            caller_project,
        }
    }

    pub fn with_provenance(
        enclosing_declaration: Option<DeclarationId>,
        enclosing_callable: Option<CallableId>,
        caller_module: Option<ModuleId>,
        caller_package: Option<ModuleId>,
        caller_project: Option<ProjectIdentity>,
    ) -> Self {
        Self {
            enclosing_declaration,
            enclosing_callable,
            caller_module,
            caller_package,
            caller_project,
        }
    }
}

/// Canonical identity-based access relation shared by semantic consumers.
pub fn is_member_accessible(
    visibility: Visibility,
    owner: &DeclarationId,
    declaration_package: Option<&ModuleId>,
    access: &AccessContext,
    hierarchy: &dyn TypeHierarchy,
) -> bool {
    match visibility {
        Visibility::Public => true,
        Visibility::Private => access.enclosing_declaration.as_ref() == Some(owner),
        Visibility::Protected => access
            .enclosing_declaration
            .as_ref()
            .is_some_and(|context| hierarchy.is_subclass(context, owner)),
        Visibility::Module => access
            .caller_module
            .as_ref()
            .or_else(|| access.enclosing_declaration.as_ref().map(|declaration| &declaration.module))
            == Some(&owner.module),
        Visibility::Package => matches!(
            (access.caller_package.as_ref(), declaration_package),
            (Some(caller), Some(declaration)) if caller == declaration
        ),
        Visibility::Internal => access
            .caller_project
            .or_else(|| access.caller_module.as_ref().map(|module| module.project))
            .or_else(|| access.enclosing_declaration.as_ref().map(|declaration| declaration.module.project))
            == Some(owner.module.project),
    }
}
