//! Coverage subject representation pairing canonical TypeId and query-local LocalType.

use crate::types::id::TypeId;
use crate::types::rigid::LocalType;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CoverageSubject {
    pub canonical: TypeId,
    pub local: LocalType,
}

impl CoverageSubject {
    pub(crate) fn canonical(canonical: TypeId) -> Self {
        Self {
            canonical,
            local: LocalType::Canonical(canonical),
        }
    }

    pub(crate) fn from_parts(canonical: TypeId, local: LocalType) -> Self {
        Self { canonical, local }
    }

    #[allow(dead_code)]
    pub(crate) fn canonical_type(&self) -> TypeId {
        self.canonical
    }

    #[allow(dead_code)]
    pub(crate) fn local_type(&self) -> &LocalType {
        &self.local
    }

    #[allow(dead_code)]
    pub(crate) fn contains_local_rigids(&self) -> bool {
        !self.local.free_rigids().is_empty()
    }
}
