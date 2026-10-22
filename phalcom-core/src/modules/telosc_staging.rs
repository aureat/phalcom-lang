//! Package-neutral, allocation-free transport staging for compiler-capable
//! `.telosc` containers. Staging is not semantic authority: imported native
//! products and package/Universe policy remain with their respective owners.
//!
//! Borrowing container sections is deliberate. In particular, the canonical
//! Universe loader's allocation census is not affected by this separation.

use phalcom_compiler_artifact::container::TeloscContainerView;
use phalcom_compiler_artifact::digest::{
    ArtifactDigest, DOMAIN_METADATA, DOMAIN_MODULE_WORLD, DOMAIN_QUERY_IMAGE, DOMAIN_SEMANTIC_WORLD,
};
use phalcom_compiler_artifact::header::{
    MODULE_SEMANTIC_ABI_V3, QUERY_FINGERPRINT_ABI_V3, SEMANTIC_ARTIFACT_ABI_V3, STABLE_IDENTITY_SCHEMA_V1,
    TeloscCapabilities, TeloscHeaderV2,
};
use phalcom_compiler_artifact::section::SectionKind;
use phalcom_compiler_artifact::{ArtifactError, ValidationError};
use phalcom_native_meta::NATIVE_SURFACE_SCHEMA_VERSION;
use phalcom_type_meta::{SEMANTIC_MODEL_VERSION, TYPE_METADATA_SCHEMA_VERSION};

#[derive(Debug, thiserror::Error)]
pub(crate) enum TeloscStagingError {
    #[error("artifact validation error: {0}")]
    Artifact(#[from] ArtifactError),
    #[error("capability error: {0}")]
    Capability(&'static str),
    #[error("metadata section error: {0}")]
    Metadata(#[from] phalcom_compiler_artifact::metadata::MetadataSectionError),
}

/// Non-owning transport section views. All required sections have passed
/// container bounds/digest checks plus cross-section provenance validation.
/// Wire decoding, import/relocation and publication are deliberately separate.
pub(crate) struct StagedCompilerSections<'a> {
    pub module_world: &'a [u8],
    pub semantic_world: &'a [u8],
    pub query_image: &'a [u8],
    pub metadata: &'a [u8],
    pub executable: Option<&'a [u8]>,
}

impl<'a> StagedCompilerSections<'a> {
    /// Generic entry point for compiler-capable package or builtin artifacts.
    /// The caller must separately prove exact package ownership and any
    /// stronger bootstrapping/executable policy before publishing authority.
    pub fn from_container(container: &TeloscContainerView<'a>) -> Result<Self, TeloscStagingError> {
        validate_compiler_header_abi(container.header())?;
        let compiler = TeloscCapabilities::CANONICAL_COMPILER_IMAGE_V1 | TeloscCapabilities::RUNTIME_PUBLIC_METADATA;
        if !container.header().capabilities.contains(compiler) {
            return Err(TeloscStagingError::Capability("missing compiler artifact capabilities"));
        }
        Self::from_compatible_container(container)
    }

    /// Stage sections after an enclosing policy has already verified the header.
    /// The Universe reader uses this path to retain its exact capability policy
    /// and prior error ordering without rechecking ABI fields or allocating.
    pub fn from_compatible_container(container: &TeloscContainerView<'a>) -> Result<Self, TeloscStagingError> {
        let header = container.header();
        let module_world = container
            .require_section(SectionKind::ModuleWorld)
            .map_err(|_| TeloscStagingError::Capability("missing MODULE_WORLD section"))?;
        let semantic_world = container
            .require_section(SectionKind::SemanticWorld)
            .map_err(|_| TeloscStagingError::Capability("missing SEMANTIC_WORLD section"))?;
        let query_image = container
            .require_section(SectionKind::QueryImage)
            .map_err(|_| TeloscStagingError::Capability("missing QUERY_IMAGE section"))?;
        let metadata = container
            .require_section(SectionKind::Metadata)
            .map_err(|_| TeloscStagingError::Capability("missing METADATA section"))?;
        let executable = header.capabilities.contains(TeloscCapabilities::PORTABLE_EXECUTABLE)
            .then(|| container.require_section(SectionKind::Executable))
            .transpose()
            .map_err(|_| TeloscStagingError::Capability("missing EXECUTABLE section"))?;
        validate_section_provenance(header, module_world, semantic_world, query_image, metadata)?;
        Ok(Self { module_world, semantic_world, query_image, metadata, executable })
    }
}

/// Shared ABI compatibility checks; package identity and availability
/// contracts are policy decisions, not transport decisions.
pub(crate) fn validate_compiler_header_abi(header: &TeloscHeaderV2) -> Result<(), TeloscStagingError> {
    for (kind, found, expected) in [
        ("semantic_model_version", header.semantic_model_version, SEMANTIC_MODEL_VERSION),
        ("stable_identity_schema", header.stable_identity_schema, STABLE_IDENTITY_SCHEMA_V1),
        ("module_semantic_abi", header.module_semantic_abi, MODULE_SEMANTIC_ABI_V3),
        ("semantic_artifact_abi", header.semantic_artifact_abi, SEMANTIC_ARTIFACT_ABI_V3),
        ("query_fingerprint_abi", header.query_fingerprint_abi, QUERY_FINGERPRINT_ABI_V3),
        ("native_surface_schema_version", header.native_surface_schema_version, NATIVE_SURFACE_SCHEMA_VERSION),
        ("type_metadata_schema_version", header.type_metadata_schema_version, TYPE_METADATA_SCHEMA_VERSION),
    ] {
        if found != expected {
            return Err(ArtifactError::from(ValidationError::IncompatibleAbi { kind, found, expected }).into());
        }
    }
    let expected_catalog = phalcom_native_surface::catalog_fingerprint().0;
    if header.native_catalog_fingerprint != expected_catalog {
        return Err(ArtifactError::from(ValidationError::NativeCatalogFingerprintMismatch {
            found: header.native_catalog_fingerprint,
            expected: expected_catalog,
        }).into());
    }
    Ok(())
}

/// Source bytes are intentionally absent; require nonzero committed provenance
/// and independently recompute each portable section digest.
fn validate_section_provenance(
    header: &TeloscHeaderV2,
    module_world: &[u8],
    semantic_world: &[u8],
    query_image: &[u8],
    metadata: &[u8],
) -> Result<(), TeloscStagingError> {
    if header.source_digest == ArtifactDigest::ZERO {
        return Err(ArtifactError::from(ValidationError::IncompatibleWorld(
            "source provenance digest is missing".into(),
        )).into());
    }
    let expected = [
        ("interface", header.interface_digest, ArtifactDigest::digest_domain(DOMAIN_MODULE_WORLD, module_world)),
        ("semantic", header.semantic_digest, ArtifactDigest::digest_domain(DOMAIN_SEMANTIC_WORLD, semantic_world)),
        ("query", header.query_digest, ArtifactDigest::digest_domain(DOMAIN_QUERY_IMAGE, query_image)),
        ("metadata", header.metadata_digest, ArtifactDigest::digest_domain(DOMAIN_METADATA, metadata)),
    ];
    for (name, found, actual) in expected {
        if found != actual {
            return Err(ArtifactError::from(ValidationError::IncompatibleWorld(
                format!("{name} provenance digest mismatch"),
            )).into());
        }
    }
    Ok(())
}
