//! Metadata header, versions, profile, and features.

use crate::fingerprint::Fingerprint128;
use serde::{Deserialize, Serialize};

pub const MIN_SUPPORTED_TYPE_METADATA_SCHEMA_VERSION: u32 = 1;
/// Schema v14 canonicalizes global Union structural fingerprints over stable
/// child fingerprints, independent of process-local TypeId order. Scoped
/// union ordering remains represented.
/// Schema v13 adds FieldKey heterogeneous Record keys, durable StaticValue DAGs, and static constants.
/// Legacy v12 string Record field names remain readable as Symbol FieldKeys.
/// Schema v12 encodes exact static integers as normalized decimal strings.
/// Legacy numeric integer terms remain readable.
/// Schema v11 preserves declaration, variant, and conformance callable ownership.
/// Legacy declaration-shaped callable owners remain readable.
/// Schema v10 adds structural member presence and anonymous Tuple/Record remainders;
/// v9 adds scoped heterogeneous `RecordFields` arguments; v8 adds a
/// typed static-domain catalog and v7 retains selected source member identity.
pub const TYPE_METADATA_SCHEMA_VERSION: u32 = 14;
pub const SEMANTIC_MODEL_VERSION: u32 = 1;
pub const NATIVE_SURFACE_SCHEMA_VERSION: u32 = 2;

#[inline]
pub const fn supports_type_metadata_schema(version: u32) -> bool {
    version >= MIN_SUPPORTED_TYPE_METADATA_SCHEMA_VERSION && version <= TYPE_METADATA_SCHEMA_VERSION
}

/// Metadata retention profile.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize, phalcom_type_meta_derive::BudgetDecode)]
pub enum MetadataProfile {
    RuntimeMinimal,
    RuntimePublic,
    ToolingDebug,
    Proof,
}

/// Feature section identifier.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize, phalcom_type_meta_derive::BudgetDecode)]
pub struct FeatureSectionId(pub Box<str>);

/// Enabled features in this metadata artifact.
#[derive(Clone, Debug, Default, Eq, Hash, PartialEq, Deserialize, Serialize, phalcom_type_meta_derive::BudgetDecode)]
pub struct MetadataFeatures {
    pub type_lambdas: bool,
    pub record_rows: bool,
    #[serde(default)]
    pub static_domains: bool,
    pub runtime_type_constants: bool,
    pub source_occurrences: bool,
    #[serde(default)]
    pub static_values: bool,
    #[serde(default)]
    pub static_constants: bool,
    #[serde(default)]
    pub static_definitions: bool,
    pub advanced_sections: Box<[FeatureSectionId]>,
}

/// Producer identity (e.g. phalcomc).
#[derive(Clone, Debug, Eq, Hash, PartialEq, Deserialize, Serialize, phalcom_type_meta_derive::BudgetDecode)]
pub struct ProducerIdentity(pub Box<str>);

/// Artifact identity scheme.
#[derive(Clone, Debug, Eq, Hash, PartialEq, Deserialize, Serialize, phalcom_type_meta_derive::BudgetDecode)]
pub enum ArtifactIdentityScheme {
    V1Standard,
    SessionLocal,
}

/// Root metadata header.
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize, phalcom_type_meta_derive::BudgetDecode)]
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
