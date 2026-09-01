//! Conversions from semantic/module identities to durable metadata stable identities.

use crate::identity::{CallableId, DeclarationId, DispatchSide, FieldId};
use phalcom_modules::{ModuleId, ProjectIdentity};
use phalcom_type_meta::fingerprint::Fingerprint128;
use phalcom_type_meta::identity::{StableCallableRef, StableDeclarationRef, StableDispatchSide, StableFieldRef, StableModuleRef, StableProjectRef};

pub fn to_stable_project(proj: &ProjectIdentity) -> StableProjectRef {
    match proj {
        ProjectIdentity::Universe => StableProjectRef::Builtin {
            namespace: "universe".into(),
            version: "0.1.0".into(),
        },
        ProjectIdentity::Resolved(res_id) => StableProjectRef::SourceArtifact {
            logical_uri: res_id.to_string().into_boxed_str(),
            source_fingerprint: Fingerprint128::ZERO,
        },
        ProjectIdentity::Synthetic(syn_id) => StableProjectRef::Session {
            session_fingerprint: Fingerprint128::from_u128(syn_id.raw() as u128),
        },
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
