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

/// Stable callable reference.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
pub struct StableCallableRef {
    pub owner: StableDeclarationRef,
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

/// Stable source span reference.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
pub struct SourceSpanRef {
    pub start: u32,
    pub end: u32,
}
