//! Bounded `METADATA` section wrapper for `RuntimePublic` `SemanticMetadataBundle` (`PERF003.C2.P7.T10`).

use phalcom_type_meta::header::{MetadataIdentityMode, MetadataProfile};

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
    pub identity_mode: MetadataIdentityMode,
    pub package_digest: [u8; 32],
    pub preflight_tokens: u32,
    pub preflight_containers: u32,
    pub preflight_nesting: u16,
    pub encoded_bundle: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MetadataSectionError {
    NonRuntimePublicProfile(MetadataProfile),
    NonStandardIdentityMode(MetadataIdentityMode),
    UnsupportedVersion { wrapper: u16, inner: u16 },
    PackageDigestMismatch,
    ByteBudgetExceeded { limit: usize, actual: usize },
    TokenBudgetExceeded { limit: u32, actual: u32 },
    ContainerBudgetExceeded { limit: u32, actual: u32 },
    NestingBudgetExceeded { limit: u16, actual: u16 },
    SharedArtifactBudgetExhausted { remaining: usize, requested: usize },
}

pub fn validate_and_charge_metadata_preflight(
    section: &MetadataSectionV1,
    limits: &MetadataPreflightLimits,
    expected_package_digest: &[u8; 32],
    shared_remaining_bytes: &mut usize,
) -> Result<(), MetadataSectionError> {
    if section.profile != MetadataProfile::RuntimePublic {
        return Err(MetadataSectionError::NonRuntimePublicProfile(section.profile));
    }
    if section.identity_mode != MetadataIdentityMode::V1Standard {
        return Err(MetadataSectionError::NonStandardIdentityMode(section.identity_mode));
    }
    if section.wrapper_version != 1 || section.inner_schema_version != 1 {
        return Err(MetadataSectionError::UnsupportedVersion {
            wrapper: section.wrapper_version,
            inner: section.inner_schema_version,
        });
    }
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
    if *shared_remaining_bytes < len {
        return Err(MetadataSectionError::SharedArtifactBudgetExhausted {
            remaining: *shared_remaining_bytes,
            requested: len,
        });
    }
    *shared_remaining_bytes -= len;
    Ok(())
}
