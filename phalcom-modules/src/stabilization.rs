//! Shared stabilization policies for compiler, resolver and tooling.

use crate::{BuiltinProject, ModuleId, ProjectIdentity, SourceId};

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ResolverGeneration(pub u64);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DunderRole {
    Forbidden,
    ContextIntrinsic,
    ReflectionIntrinsic,
    OverridableHook,
}

pub fn dunder_role(name: &str) -> Option<DunderRole> {
    if !(name.starts_with("__") && name.ends_with("__") && name.len() > 4) {
        return None;
    }
    Some(match name {
        "__module__" | "__package__" | "__project__" => DunderRole::ContextIntrinsic,
        "__selector__" | "__exports__" => DunderRole::ReflectionIntrinsic,
        "__intercept__" => DunderRole::OverridableHook,
        _ => DunderRole::Forbidden,
    })
}

pub fn builtin_module_uri(module: &ModuleId) -> Option<String> {
    let ProjectIdentity::Builtin(project) = module.project else {
        return None;
    };
    let host = match project {
        BuiltinProject::Universe => "universe",
        BuiltinProject::Std => "std",
    };
    if module.path.is_root() {
        return Some(format!("phalcom://{host}/"));
    }
    Some(format!(
        "phalcom://{host}/{}",
        module.path.components().iter().map(|c| c.as_str()).collect::<Vec<_>>().join("/")
    ))
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResolvedDocumentIdentity {
    pub source: SourceId,
    pub module: ModuleId,
    pub generation: ResolverGeneration,
}
