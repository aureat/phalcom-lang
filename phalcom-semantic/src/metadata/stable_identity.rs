//! Conversions from semantic/module identities to durable metadata stable identities.

use crate::identity::{CallableId, CallableOwnerId, DeclarationId, DispatchSide, FieldId};
use phalcom_modules::{ModuleId, ProjectIdentity, ProjectUniverse};
use phalcom_type_meta::fingerprint::Fingerprint128;
use phalcom_type_meta::identity::{
    StableCallableOwnerRef, StableCallableRef, StableDeclarationRef, StableDispatchSide, StableFieldRef, StableImplRef, StableModuleRef, StableProjectRef,
    StableVariantRef,
};

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
        owner: stabilize_callable_owner(&callable.owner, |module| to_stable_module_with_context(module, context)),
        side: to_stable_dispatch_side(callable.side),
        selector: callable.selector.encode().into_boxed_str(),
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
        owner: stabilize_callable_owner(&callable.owner, to_stable_module),
        side: to_stable_dispatch_side(callable.side),
        selector: callable.selector.encode().into_boxed_str(),
    }
}

pub fn to_stable_field(field: &FieldId) -> StableFieldRef {
    StableFieldRef {
        owner: to_stable_declaration(&field.owner),
        side: to_stable_dispatch_side(field.side),
        name: field.name.clone(),
    }
}

fn stabilize_callable_owner(owner: &CallableOwnerId, module: impl Fn(&ModuleId) -> StableModuleRef) -> StableCallableOwnerRef {
    let declaration = |decl: &DeclarationId| StableDeclarationRef {
        module: module(&decl.module),
        path: Box::new([decl.name.clone()]),
    };
    match owner {
        CallableOwnerId::Declaration(decl) => StableCallableOwnerRef::Declaration(declaration(decl)),
        CallableOwnerId::Variant(variant) => StableCallableOwnerRef::Variant(StableVariantRef {
            owner: declaration(&variant.owner),
            selector: variant.selector.encode().into_boxed_str(),
        }),
        CallableOwnerId::Conformance(implementation) => StableCallableOwnerRef::Conformance(StableImplRef {
            module: module(&implementation.module),
            local_id: implementation.local.0,
        }),
    }
}

pub fn to_stable_impl_with_context(r#impl: &crate::identity::ImplId, context: &StableIdentityContext<'_>) -> phalcom_type_meta::identity::StableImplRef {
    phalcom_type_meta::identity::StableImplRef {
        module: to_stable_module_with_context(&r#impl.module, context),
        local_id: r#impl.local.0,
    }
}

pub fn to_stable_impl(r#impl: &crate::identity::ImplId) -> phalcom_type_meta::identity::StableImplRef {
    phalcom_type_meta::identity::StableImplRef {
        module: to_stable_module(&r#impl.module),
        local_id: r#impl.local.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::identity::{ImplId, ImplLocalId, VariantId};
    use phalcom_common::selector::Selector;

    #[test]
    fn stable_callable_owners_preserve_all_source_identity_categories() {
        let module = ModuleId {
            project: ProjectIdentity::Universe,
            path: Default::default(),
        };
        let declaration = DeclarationId::new(module.clone(), "Box".into());
        let owners = [
            CallableOwnerId::Declaration(declaration.clone()),
            CallableOwnerId::Variant(VariantId::new(declaration.clone(), Selector::getter("A").unwrap())),
            CallableOwnerId::Variant(VariantId::new(declaration, Selector::getter("B").unwrap())),
            CallableOwnerId::Conformance(ImplId::new(module.clone(), ImplLocalId(1))),
            CallableOwnerId::Conformance(ImplId::new(module, ImplLocalId(2))),
        ];
        let projects = ProjectUniverse::new();
        let context = StableIdentityContext::new(&projects);
        let projected: Vec<_> = owners
            .into_iter()
            .map(|owner| {
                let callable = CallableId {
                    owner,
                    side: DispatchSide::Instance,
                    selector: Selector::getter("foo").unwrap(),
                };
                let stable = to_stable_callable(&callable);
                assert_eq!(stable, to_stable_callable_with_context(&callable, &context));
                assert_eq!(stable.selector.as_ref(), callable.selector.encode());
                stable
            })
            .collect();
        assert_eq!(projected.iter().collect::<std::collections::BTreeSet<_>>().len(), projected.len());
        assert!(projected[0].owner.try_declaration().is_some());
        assert!(projected[1].owner.try_declaration().is_none());
        assert!(projected[1].owner.enclosing_declaration().is_some());
        assert!(projected[3].owner.enclosing_declaration().is_none());
    }
}
