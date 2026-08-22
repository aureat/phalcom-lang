//! Metadata header, versions, profile, and features.

use crate::fingerprint::Fingerprint128;
use serde::{Deserialize, Serialize};

pub const MIN_SUPPORTED_TYPE_METADATA_SCHEMA_VERSION: u32 = 1;
pub const TYPE_METADATA_SCHEMA_VERSION: u32 = 2;
pub const SEMANTIC_MODEL_VERSION: u32 = 1;
pub const NATIVE_SURFACE_SCHEMA_VERSION: u32 = 1;

#[inline]
pub const fn supports_type_metadata_schema(version: u32) -> bool {
    version >= MIN_SUPPORTED_TYPE_METADATA_SCHEMA_VERSION && version <= TYPE_METADATA_SCHEMA_VERSION
}

/// Metadata retention profile.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
pub enum MetadataProfile {
    RuntimeMinimal,
    RuntimePublic,
    ToolingDebug,
    Proof,
}

/// Feature section identifier.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
pub struct FeatureSectionId(pub Box<str>);

/// Enabled features in this metadata artifact.
#[derive(Clone, Debug, Default, Eq, Hash, PartialEq, Deserialize, Serialize)]
pub struct MetadataFeatures {
    pub type_lambdas: bool,
    pub record_rows: bool,
    pub runtime_type_constants: bool,
    pub source_occurrences: bool,
    pub advanced_sections: Box<[FeatureSectionId]>,
}

/// Producer identity (e.g. phalcomc).
#[derive(Clone, Debug, Eq, Hash, PartialEq, Deserialize, Serialize)]
pub struct ProducerIdentity(pub Box<str>);

/// Artifact identity scheme.
#[derive(Clone, Debug, Eq, Hash, PartialEq, Deserialize, Serialize)]
pub enum ArtifactIdentityScheme {
    V1Standard,
    SessionLocal,
}

/// Root metadata header.
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct SemanticMetadataHeader {
    pub schema_version: u32,
    pub semantic_model_version: u32,
    pub producer: ProducerIdentity,
    pub producer_version: Box<str>,
    pub native_surface_schema_version: u32,
    pub profile: MetadataProfile,
    pub features: MetadataFeatures,
    pub identity_scheme: ArtifactIdentityScheme,
    pub source_fingerprint: Fingerprint128,
    pub interface_fingerprint: Fingerprint128,
}
