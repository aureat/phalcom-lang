//! Bounded `METADATA` section wrapper for `RuntimePublic` `SemanticMetadataBundle` (`PERF003.C2.P7.T10`).
//!
//! Content limits and envelope agreement are validated here. Decoding uses the
//! shared allocation ledger for the owned payload and the typed metadata graph.

use crate::codec::{BinaryReader, BinaryWriter};
use crate::error::{ArtifactError, ValidationError};
use crate::limits::ValidationLimits;
use phalcom_type_meta::header::{ArtifactIdentityScheme, MetadataProfile};
use phalcom_type_meta::{SemanticMetadataBundle, TYPE_METADATA_SCHEMA_VERSION};
use phalcom_type_meta::allocation::{AllocationBudget, AllocationError};

mod preflight;
pub use preflight::{MetadataPreflightCounts, inspect_metadata_json};

pub type MetadataIdentityMode = ArtifactIdentityScheme;

pub const METADATA_SECTION_SCHEMA_V1: u32 = 1;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MetadataPreflightLimits {
    pub max_metadata_bytes: usize,
    pub max_token_count: u32,
    pub max_container_count: u32,
    pub max_nesting_depth: u16,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MetadataSectionV1 {
    pub wrapper_version: u16,
    pub inner_schema_version: u16,
    pub profile: MetadataProfile,
    pub identity_mode: ArtifactIdentityScheme,
    pub package_digest: [u8; 32],
    pub preflight_tokens: u32,
    pub preflight_containers: u32,
    pub preflight_nesting: u16,
    pub encoded_bundle: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MetadataSectionError {
    NonRuntimePublicProfile(MetadataProfile),
    NonStandardIdentityMode(ArtifactIdentityScheme),
    UnsupportedVersion {
        wrapper: u16,
        inner: u16,
    },
    PackageDigestMismatch,
    ByteBudgetExceeded {
        limit: usize,
        actual: usize,
    },
    TokenBudgetExceeded {
        limit: u32,
        actual: u32,
    },
    ContainerBudgetExceeded {
        limit: u32,
        actual: u32,
    },
    NestingBudgetExceeded {
        limit: u16,
        actual: u16,
    },
    SharedArtifactBudgetExhausted {
        remaining: usize,
        requested: usize,
    },
    InvalidEncodedBundle(String),
    PreflightCountsMismatch {
        advertised: MetadataPreflightCounts,
        actual: MetadataPreflightCounts,
    },
}

impl std::fmt::Display for MetadataSectionError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{self:?}")
    }
}

impl std::error::Error for MetadataSectionError {}

fn validate_envelope(wrapper: u16, inner: u16, profile: MetadataProfile, identity: &ArtifactIdentityScheme) -> Result<(), MetadataSectionError> {
    if profile != MetadataProfile::RuntimePublic {
        return Err(MetadataSectionError::NonRuntimePublicProfile(profile));
    }
    if *identity != ArtifactIdentityScheme::V1Standard {
        return Err(MetadataSectionError::NonStandardIdentityMode(identity.clone()));
    }
    if u32::from(wrapper) != METADATA_SECTION_SCHEMA_V1 || u32::from(inner) != TYPE_METADATA_SCHEMA_VERSION {
        return Err(MetadataSectionError::UnsupportedVersion { wrapper, inner });
    }
    Ok(())
}

#[derive(Debug, thiserror::Error)]
pub enum MetadataTransportError {
    #[error("metadata envelope: {0:?}")]
    Envelope(#[from] MetadataSectionError),
    #[error("metadata codec: {0}")]
    Decode(#[from] phalcom_type_meta::MetadataDecodeError),
    #[error("metadata validation: {0}")]
    Validation(#[from] phalcom_type_meta::MetadataValidationError),
    #[error("metadata JSON: {0}")]
    Encode(#[from] serde_json::Error),
    #[error("metadata is not UTF-8: {0}")]
    Utf8(#[from] std::str::Utf8Error),
    #[error("metadata envelope disagrees with decoded bundle: {0}")]
    HeaderMismatch(&'static str),
}

impl MetadataSectionV1 {
    /// Wraps the existing metadata schema without allocating new pool identities.
    pub fn from_bundle(
        bundle: &SemanticMetadataBundle,
        package_digest: [u8; 32],
        limits: &MetadataPreflightLimits,
        bundle_limits: &phalcom_type_meta::ValidationLimits,
    ) -> Result<Self, MetadataTransportError> {
        phalcom_type_meta::validate_metadata_bundle(bundle, bundle_limits)?;
        if bundle.header.schema_version != TYPE_METADATA_SCHEMA_VERSION {
            return Err(MetadataTransportError::HeaderMismatch("writer requires current schema"));
        }
        let encoded_bundle = phalcom_type_meta::encode_metadata_json(bundle)?.into_bytes();
        let counts = inspect_metadata_json(&encoded_bundle, limits).map_err(MetadataTransportError::Envelope)?;
        let section = Self {
            wrapper_version: METADATA_SECTION_SCHEMA_V1 as u16,
            inner_schema_version: TYPE_METADATA_SCHEMA_VERSION as u16,
            profile: bundle.header.profile,
            identity_mode: bundle.header.identity_scheme.clone(),
            package_digest,
            preflight_tokens: counts.tokens,
            preflight_containers: counts.containers,
            preflight_nesting: counts.nesting,
            encoded_bundle,
        };
        let mut budget = limits.max_metadata_bytes;
        validate_and_charge_metadata_preflight(&section, limits, &package_digest, &mut budget).map_err(MetadataTransportError::Envelope)?;
        Ok(section)
    }

    /// Content preflight precedes the existing allocating decoder and validator.
    /// Budget is committed only after envelope, bundle, and agreement validation.
    /// The committed charge covers both encoded payload bytes and native decoded/scratch allocations.
    pub fn decode_bundle(
        &self,
        limits: &MetadataPreflightLimits,
        bundle_limits: &phalcom_type_meta::ValidationLimits,
        expected_package_digest: &[u8; 32],
        shared_remaining_bytes: &mut usize,
    ) -> Result<SemanticMetadataBundle, MetadataTransportError> {
        let mut transaction = AllocationBudget::new(*shared_remaining_bytes);
        let bundle = self.decode_bundle_with_allocation_budget(
            limits,
            bundle_limits,
            expected_package_digest,
            &mut transaction,
        )?;
        *shared_remaining_bytes = transaction.remaining();
        Ok(bundle)
    }

    pub fn decode_bundle_with_allocation_budget(
        &self,
        limits: &MetadataPreflightLimits,
        bundle_limits: &phalcom_type_meta::ValidationLimits,
        expected_package_digest: &[u8; 32],
        transaction: &mut AllocationBudget,
    ) -> Result<SemanticMetadataBundle, MetadataTransportError> {
        validate_metadata_preflight(self, limits, expected_package_digest).map_err(MetadataTransportError::Envelope)?;
        let json = std::str::from_utf8(&self.encoded_bundle)?;
        // Probe only the header before the allocating owner codec: never invoke
        // legacy normalization based on a forged current-schema envelope.
        let probe = phalcom_type_meta::budgeted::probe_header(json)
            .map_err(|e| MetadataTransportError::Decode(phalcom_type_meta::MetadataDecodeError::Budgeted(e)))?;
        if u32::from(self.inner_schema_version) != probe.schema_version {
            return Err(MetadataTransportError::HeaderMismatch("schema"));
        }
        if self.profile != probe.profile {
            return Err(MetadataTransportError::HeaderMismatch("profile"));
        }
        if self.identity_mode != probe.identity_scheme {
            return Err(MetadataTransportError::HeaderMismatch("identity mode"));
        }
        let bundle = phalcom_type_meta::encode::decode_metadata_json_with_allocation_budget(
            json,
            bundle_limits,
            None,
            transaction,
        )?;
        if u32::from(self.inner_schema_version) != bundle.header.schema_version {
            return Err(MetadataTransportError::HeaderMismatch("schema"));
        }
        if self.profile != bundle.header.profile {
            return Err(MetadataTransportError::HeaderMismatch("profile"));
        }
        if self.identity_mode != bundle.header.identity_scheme {
            return Err(MetadataTransportError::HeaderMismatch("identity mode"));
        }
        Ok(bundle)
    }

    pub fn encode(&self) -> Result<Vec<u8>, ArtifactError> {
        let mut out = BinaryWriter::new();
        out.write_u32(METADATA_SECTION_SCHEMA_V1);
        out.write_u16(self.wrapper_version);
        out.write_u16(self.inner_schema_version);
        let profile_tag = match self.profile {
            MetadataProfile::RuntimeMinimal => 0u8,
            MetadataProfile::RuntimePublic => 1u8,
            MetadataProfile::ToolingDebug => 2u8,
            MetadataProfile::Proof => 3u8,
        };
        out.write_u8(profile_tag);
        let identity_mode_tag = match self.identity_mode {
            ArtifactIdentityScheme::V1Standard => 0u8,
            ArtifactIdentityScheme::SessionLocal => 1u8,
        };
        out.write_u8(identity_mode_tag);
        out.write_fixed(&self.package_digest);
        out.write_u32(self.preflight_tokens);
        out.write_u32(self.preflight_containers);
        out.write_u16(self.preflight_nesting);
        out.write_bytes_with_len(&self.encoded_bundle);
        Ok(out.into_vec())
    }

    pub fn decode_with_allocation_budget(
        bytes: &[u8],
        limits: &ValidationLimits,
        transaction: &mut AllocationBudget,
    ) -> Result<Self, ArtifactError> {
        let section_limit = usize::try_from(limits.max_decoded_section_bytes.min(limits.max_metadata_bytes))
            .map_err(|_| allocation_error(AllocationError::Overflow))?;
        let mut section = transaction.child(section_limit);
        let mut reader = BinaryReader::new(bytes);
        let schema = reader.read_u32()?;
        if schema != METADATA_SECTION_SCHEMA_V1 {
            return Err(ValidationError::ReferentialIntegrity("unsupported metadata section schema".into()).into());
        }
        let wrapper_version = reader.read_u16()?;
        let inner_schema_version = reader.read_u16()?;
        let profile = match reader.read_u8()? {
            0 => MetadataProfile::RuntimeMinimal,
            1 => MetadataProfile::RuntimePublic,
            2 => MetadataProfile::ToolingDebug,
            3 => MetadataProfile::Proof,
            other => return Err(ValidationError::ReferentialIntegrity(format!("unknown profile tag {other}")).into()),
        };
        let identity_mode = match reader.read_u8()? {
            0 => ArtifactIdentityScheme::V1Standard,
            1 => ArtifactIdentityScheme::SessionLocal,
            other => return Err(ValidationError::ReferentialIntegrity(format!("unknown identity mode tag {other}")).into()),
        };
        validate_envelope(wrapper_version, inner_schema_version, profile, &identity_mode)?;
        let package_digest = reader.read_fixed::<32>()?;
        let preflight_tokens = reader.read_u32()?;
        let preflight_containers = reader.read_u32()?;
        let preflight_nesting = reader.read_u16()?;
        let byte_limit = usize::try_from(limits.max_metadata_bytes)
            .unwrap_or(usize::MAX)
            .min(section.remaining());
        let encoded = reader.read_bytes_with_len(byte_limit)?;
        if reader.has_remaining() {
            return Err(ValidationError::ReferentialIntegrity("trailing metadata section bytes".into()).into());
        }
        // The bytes remain borrowed until the content bound has been checked.
        let preflight_limits = MetadataPreflightLimits {
            max_metadata_bytes: byte_limit,
            max_token_count: u32::try_from(limits.max_semantic_entries_per_family).unwrap_or(u32::MAX),
            max_container_count: u32::try_from(limits.max_semantic_entries_per_family).unwrap_or(u32::MAX),
            max_nesting_depth: u16::try_from(limits.max_recursion_depth).unwrap_or(u16::MAX),
        };
        let counts = inspect_metadata_json(encoded, &preflight_limits)?;
        if counts
            != (MetadataPreflightCounts {
                tokens: preflight_tokens,
                containers: preflight_containers,
                nesting: preflight_nesting,
            })
        {
            return Err(MetadataSectionError::PreflightCountsMismatch {
                advertised: MetadataPreflightCounts {
                    tokens: preflight_tokens,
                    containers: preflight_containers,
                    nesting: preflight_nesting,
                },
                actual: counts,
            }
            .into());
        }
        let mut encoded_bundle = section.vector::<u8>(encoded.len()).map_err(allocation_error)?;
        encoded_bundle.extend_from_slice(encoded);
        let decoded = Self {
            wrapper_version,
            inner_schema_version,
            profile,
            identity_mode,
            package_digest,
            preflight_tokens,
            preflight_containers,
            preflight_nesting,
            encoded_bundle,
        };
        transaction.commit_child(section).map_err(allocation_error)?;
        Ok(decoded)
    }

    pub fn decode_with_budget(bytes: &[u8], limits: &ValidationLimits, remaining_budget: &mut u64) -> Result<Self, ArtifactError> {
        let available = usize::try_from(*remaining_budget).map_err(|_| allocation_error(AllocationError::Overflow))?;
        let mut transaction = AllocationBudget::new(available);
        let decoded = Self::decode_with_allocation_budget(bytes, limits, &mut transaction)?;
        *remaining_budget = u64::try_from(transaction.remaining()).map_err(|_| allocation_error(AllocationError::Overflow))?;
        Ok(decoded)
    }

    pub fn decode(bytes: &[u8], limits: &ValidationLimits) -> Result<Self, ArtifactError> {
        let mut budget = limits.max_decoded_section_bytes;
        Self::decode_with_budget(bytes, limits, &mut budget)
    }
}

fn allocation_error(error: AllocationError) -> ArtifactError {
    ArtifactError::LimitExceeded(match error {
        AllocationError::Exhausted => "metadata section allocation budget exceeded",
        AllocationError::Overflow => "metadata section allocation layout overflow",
        AllocationError::Failed => "metadata section allocation failed",
    }
    .into())
}

/// Checks actual content and reserves encoded payload bytes transactionally.
/// This is not yet a bound on the owning decoder/validator's total allocations.
pub fn validate_and_charge_metadata_preflight(
    section: &MetadataSectionV1,
    limits: &MetadataPreflightLimits,
    expected_package_digest: &[u8; 32],
    shared_remaining_bytes: &mut usize,
) -> Result<(), MetadataSectionError> {
    validate_metadata_preflight(section, limits, expected_package_digest)?;
    let len = section.encoded_bundle.len();
    if *shared_remaining_bytes < len {
        return Err(MetadataSectionError::SharedArtifactBudgetExhausted {
            remaining: *shared_remaining_bytes,
            requested: len,
        });
    }
    *shared_remaining_bytes -= len;
    Ok(())
}

/// Validates the borrowed metadata payload without charging it a second time.
/// `decode_with_budget` already charges the owned payload copy made for the DTO.
pub fn validate_metadata_preflight(
    section: &MetadataSectionV1,
    limits: &MetadataPreflightLimits,
    expected_package_digest: &[u8; 32],
) -> Result<(), MetadataSectionError> {
    validate_envelope(section.wrapper_version, section.inner_schema_version, section.profile, &section.identity_mode)?;
    if &section.package_digest != expected_package_digest {
        return Err(MetadataSectionError::PackageDigestMismatch);
    }
    let len = section.encoded_bundle.len();
    if len > limits.max_metadata_bytes {
        return Err(MetadataSectionError::ByteBudgetExceeded {
            limit: limits.max_metadata_bytes,
            actual: len,
        });
    }
    if section.preflight_tokens > limits.max_token_count {
        return Err(MetadataSectionError::TokenBudgetExceeded {
            limit: limits.max_token_count,
            actual: section.preflight_tokens,
        });
    }
    if section.preflight_containers > limits.max_container_count {
        return Err(MetadataSectionError::ContainerBudgetExceeded {
            limit: limits.max_container_count,
            actual: section.preflight_containers,
        });
    }
    if section.preflight_nesting > limits.max_nesting_depth {
        return Err(MetadataSectionError::NestingBudgetExceeded {
            limit: limits.max_nesting_depth,
            actual: section.preflight_nesting,
        });
    }
    let actual = inspect_metadata_json(&section.encoded_bundle, limits)?;
    let advertised = MetadataPreflightCounts {
        tokens: section.preflight_tokens,
        containers: section.preflight_containers,
        nesting: section.preflight_nesting,
    };
    if actual != advertised {
        return Err(MetadataSectionError::PreflightCountsMismatch { advertised, actual });
    }
    Ok(())
}
