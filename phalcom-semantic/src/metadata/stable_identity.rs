//! Conversions from semantic/module identities to durable metadata stable identities.

use crate::identity::{CallableId, DeclarationId, DispatchSide, FieldId};
use phalcom_modules::{ModuleId, ProjectIdentity, ProjectUniverse};
use phalcom_type_meta::fingerprint::Fingerprint128;
use phalcom_type_meta::identity::{StableCallableRef, StableDeclarationRef, StableDispatchSide, StableFieldRef, StableModuleRef, StableProjectRef};

pub struct StableIdentityContext<'a> {
    pub projects: &'a ProjectUniverse,
}

impl<'a> StableIdentityContext<'a> {
    pub fn new(projects: &'a ProjectUniverse) -> Self {
        Self { projects }
    }
}

pub fn to_stable_project(proj: &ProjectIdentity) -> StableProjectRef {
    match proj {
        ProjectIdentity::Universe => StableProjectRef::Builtin {
            namespace: "universe".into(),
            version: "0.1.0".into(),
        },
        ProjectIdentity::Resolved(_) => panic!("resolved project requires StableIdentityContext"),
        ProjectIdentity::Synthetic(syn_id) => StableProjectRef::Session {
            session_fingerprint: Fingerprint128::from_u128(syn_id.raw() as u128),
        },
    }
}

pub fn to_stable_project_with_context(proj: &ProjectIdentity, context: &StableIdentityContext<'_>) -> StableProjectRef {
    match proj {
        ProjectIdentity::Resolved(id) => context.projects.get_project(*id).map_or_else(
            || to_stable_project(proj),
            |project| StableProjectRef::SourceArtifact {
                logical_uri: project.source_identity.0.to_string_lossy().into_owned().into_boxed_str(),
                source_fingerprint: Fingerprint128(project.revision_fingerprint().0),
            },
        ),
        _ => to_stable_project(proj),
    }
}

pub fn to_stable_module_with_context(module: &ModuleId, context: &StableIdentityContext<'_>) -> StableModuleRef {
    StableModuleRef {
        project: to_stable_project_with_context(&module.project, context),
        path: module
            .path
            .components()
            .iter()
            .map(|c| c.to_string().into_boxed_str())
            .collect::<Vec<_>>()
            .into_boxed_slice(),
    }
}

pub fn to_stable_declaration_with_context(decl: &DeclarationId, context: &StableIdentityContext<'_>) -> StableDeclarationRef {
    StableDeclarationRef {
        module: to_stable_module_with_context(&decl.module, context),
        path: Box::new([decl.name.clone()]),
    }
}

pub fn to_stable_callable_with_context(callable: &CallableId, context: &StableIdentityContext<'_>) -> StableCallableRef {
    StableCallableRef {
        owner: to_stable_declaration_with_context(callable.declaration_owner(), context),
        side: to_stable_dispatch_side(callable.side),
        selector: callable.selector.to_string().into_boxed_str(),
    }
}

pub fn to_stable_field_with_context(field: &FieldId, context: &StableIdentityContext<'_>) -> StableFieldRef {
    StableFieldRef {
        owner: to_stable_declaration_with_context(&field.owner, context),
        side: to_stable_dispatch_side(field.side),
        name: field.name.clone(),
    }
}

pub fn to_stable_module(module: &ModuleId) -> StableModuleRef {
    let project = to_stable_project(&module.project);
    let path_segments: Vec<Box<str>> = module.path.components().iter().map(|c| c.to_string().into_boxed_str()).collect();
    StableModuleRef {
        project,
        path: path_segments.into_boxed_slice(),
    }
}

pub fn to_stable_declaration(decl: &DeclarationId) -> StableDeclarationRef {
    let module = to_stable_module(&decl.module);
    StableDeclarationRef {
        module,
        path: Box::new([decl.name.clone()]),
    }
}

pub fn to_stable_dispatch_side(side: DispatchSide) -> StableDispatchSide {
    match side {
        DispatchSide::Instance => StableDispatchSide::Instance,
        DispatchSide::Class => StableDispatchSide::Class,
    }
}

pub fn to_stable_callable(callable: &CallableId) -> StableCallableRef {
    StableCallableRef {
        owner: to_stable_declaration(callable.declaration_owner()),
        side: to_stable_dispatch_side(callable.side),
        selector: callable.selector.to_string().into_boxed_str(),
    }
}

pub fn to_stable_field(field: &FieldId) -> StableFieldRef {
    StableFieldRef {
        owner: to_stable_declaration(&field.owner),
        side: to_stable_dispatch_side(field.side),
        name: field.name.clone(),
    }
}
