//! Metadata header, versions, profile, and features.

use crate::fingerprint::Fingerprint128;
use serde::{Deserialize, Serialize};

pub const MIN_SUPPORTED_TYPE_METADATA_SCHEMA_VERSION: u32 = 1;
/// Schema v13 retains heterogeneous Record keys and explicit remainder key domains.
/// Schema v12 encodes exact static integers as normalized decimal strings.
/// Legacy numeric integer terms remain readable.
/// Schema v11 preserves declaration, variant, and conformance callable ownership.
/// Legacy declaration-shaped callable owners remain readable.
/// Schema v10 adds structural member presence and anonymous Tuple/Record remainders;
/// v9 adds scoped heterogeneous `RecordFields` arguments; v8 adds a
/// typed static-domain catalog and v7 retains selected source member identity.
pub const TYPE_METADATA_SCHEMA_VERSION: u32 = 13;
pub const SEMANTIC_MODEL_VERSION: u32 = 1;
pub const NATIVE_SURFACE_SCHEMA_VERSION: u32 = 2;

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
    #[serde(default)]
    pub static_domains: bool,
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
    /// The original digest scheme on a decode-normalized legacy graph. A
    /// semantic re-export replaces these digests and clears this provenance.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub legacy_fingerprint_schema_version: Option<u32>,
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

impl SemanticMetadataHeader {
    pub fn fingerprint_schema_version(&self) -> u32 {
        self.legacy_fingerprint_schema_version.unwrap_or(self.schema_version)
    }
}
