//! Shared stabilization policies for compiler, resolver and tooling.

use crate::{ModuleId, SourceId};

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ResolverGeneration(pub u64);

pub use crate::dunder::{DunderCategory, DunderPolicy, DunderPolicyError, DunderRole};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResolvedDocumentIdentity {
    pub source: SourceId,
    pub module: ModuleId,
    pub generation: ResolverGeneration,
}
