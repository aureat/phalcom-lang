//! Stable artifact/project/module/declaration/member identities.

use crate::fingerprint::Fingerprint128;
use serde::{Deserialize, Serialize};

/// Stable project reference.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
pub enum StableProjectRef {
    Builtin {
        namespace: Box<str>,
        version: Box<str>,
    },
    Package {
        package: Box<str>,
        version: Box<str>,
        artifact_fingerprint: Fingerprint128,
    },
    SourceArtifact {
        logical_uri: Box<str>,
        source_fingerprint: Fingerprint128,
    },
    Session {
        session_fingerprint: Fingerprint128,
    },
}

/// Stable module reference within a project.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
pub struct StableModuleRef {
    pub project: StableProjectRef,
    pub path: Box<[Box<str>]>,
}

/// Stable declaration reference.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
pub struct StableDeclarationRef {
    pub module: StableModuleRef,
    pub path: Box<[Box<str>]>,
}

/// Stable dispatch side (instance or class).
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
pub enum StableDispatchSide {
    Instance,
    Class,
}

/// Stable exact-case owner, retaining the canonical encoded case selector.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
pub struct StableVariantRef {
    pub owner: StableDeclarationRef,
    pub selector: Box<str>,
}

/// Source ownership of a callable, independent of exact specialization evidence.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub enum StableCallableOwnerRef {
    Declaration(StableDeclarationRef),
    Variant(StableVariantRef),
    Conformance(StableImplRef),
}

impl<'de> Deserialize<'de> for StableCallableOwnerRef {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        enum Tagged {
            Declaration(StableDeclarationRef),
            Variant(StableVariantRef),
            Conformance(StableImplRef),
        }
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum Compatible {
            Tagged(Tagged),
            Legacy(StableDeclarationRef),
        }
        Ok(match Compatible::deserialize(deserializer)? {
            Compatible::Tagged(Tagged::Declaration(owner)) | Compatible::Legacy(owner) => Self::Declaration(owner),
            Compatible::Tagged(Tagged::Variant(owner)) => Self::Variant(owner),
            Compatible::Tagged(Tagged::Conformance(owner)) => Self::Conformance(owner),
        })
    }
}

impl StableCallableOwnerRef {
    /// Returns only a declaration-owned callable's lexical owner.
    pub fn try_declaration(&self) -> Option<&StableDeclarationRef> {
        match self {
            Self::Declaration(owner) => Some(owner),
            Self::Variant(_) | Self::Conformance(_) => None,
        }
    }

    /// Returns the enclosing nominal declaration, when ownership provides one.
    pub fn enclosing_declaration(&self) -> Option<&StableDeclarationRef> {
        match self {
            Self::Declaration(owner) => Some(owner),
            Self::Variant(owner) => Some(&owner.owner),
            Self::Conformance(_) => None,
        }
    }
}

/// Stable callable reference.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
pub struct StableCallableRef {
    pub owner: StableCallableOwnerRef,
    pub side: StableDispatchSide,
    pub selector: Box<str>,
}

/// Stable field reference.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
pub struct StableFieldRef {
    pub owner: StableDeclarationRef,
    pub side: StableDispatchSide,
    pub name: Box<str>,
}

/// Stable source type-member declaration identity.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
pub struct StableTypeMemberRef {
    pub owner: StableDeclarationRef,
    pub selector: Box<str>,
}

/// Stable associated static-domain declaration identity, independent of owner
/// specialization and implementation-fragment location.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
pub struct StableAssociatedStaticDomainDeclarationRef {
    pub owner: StableDeclarationRef,
    pub name: Box<str>,
}

/// Stable identity of a declared static domain, independent of any concrete
/// associated-owner specialization.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
pub enum StableStaticDomainDeclarationRef {
    Nominal(StableDeclarationRef),
    Associated(StableAssociatedStaticDomainDeclarationRef),
}

/// Stable impl reference.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
pub struct StableImplRef {
    pub module: StableModuleRef,
    pub local_id: u32,
}

/// Stable source span reference.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
pub struct SourceSpanRef {
    pub start: u32,
    pub end: u32,
}
