//! Shared stabilization policies for compiler, resolver and tooling.

use crate::{BuiltinProject, ModuleId, ProjectIdentity, SourceId};

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ResolverGeneration(pub u64);

pub use crate::dunder::{DunderCategory, DunderPolicy, DunderPolicyError, DunderRole};

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
