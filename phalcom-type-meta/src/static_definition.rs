//! Stable metadata models for static definitions.

use crate::generic::GenericParameterModeRef;
use crate::identity::StableDeclarationRef;
use crate::static_value::StaticValueDomainRef;
use serde::{Deserialize, Serialize};

/// Stable parameter record of a declared static definition.
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct StaticParameterRecord {
    pub index: u32,
    pub name: Box<str>,
    pub domain: StaticValueDomainRef,
    pub mode: GenericParameterModeRef,
}

/// Durable static definition record in the metadata bundle.
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct StaticDefinitionRecord {
    pub id: StableDeclarationRef,
    pub parameters: Box<[StaticParameterRecord]>,
    pub result_domain: StaticValueDomainRef,
    pub semantic_fingerprint: crate::fingerprint::Fingerprint128,
}
