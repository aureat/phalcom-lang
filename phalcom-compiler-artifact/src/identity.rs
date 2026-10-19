//! Portable cross-process package and compatibility identities.

use crate::digest::{ArtifactDigest, ArtifactHasher};
use crate::error::ValidationError;
use phalcom_type_meta::allocation::{AllocationBudget, AllocationError};
use phalcom_type_meta::identity::StableProjectRef;
use serde::{Deserialize, Serialize};

/// Portable package identity serialized into `.telosc` artifacts.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
pub enum TeloscPackageIdentity {
    Builtin { namespace: Box<str>, version: Box<str> },
    Package { package: Box<str>, version: Box<str> },
}

impl TeloscPackageIdentity {
    /// Stable, domain-separated digest used by sections bound to one package.
    pub fn digest(&self) -> ArtifactDigest {
        fn write_text(hasher: &mut ArtifactHasher, value: &str) {
            hasher.write_u64(value.len() as u64);
            hasher.write(value.as_bytes());
        }

        let mut hasher = ArtifactHasher::new_domain("telos.telosc.v1.package-identity");
        match self {
            Self::Builtin { namespace, version } => {
                hasher.write_u8(0);
                write_text(&mut hasher, namespace);
                write_text(&mut hasher, version);
            }
            Self::Package { package, version } => {
                hasher.write_u8(1);
                write_text(&mut hasher, package);
                write_text(&mut hasher, version);
            }
        }
        hasher.finalize()
    }

    pub fn universe() -> Self {
        Self::Builtin {
            namespace: "universe".into(),
            version: "0.1.0".into(),
        }
    }

    pub fn is_universe(&self) -> bool {
        match self {
            Self::Builtin { namespace, .. } => namespace.as_ref() == "universe",
            _ => false,
        }
    }

    pub fn is_exact_universe(&self) -> bool {
        matches!(
            self,
            Self::Builtin { namespace, version }
                if namespace.as_ref() == "universe" && version.as_ref() == "0.1.0"
        )
    }

    pub fn clone_with_allocation_budget(&self, allocations: &mut AllocationBudget) -> Result<Self, AllocationError> {
        match self {
            Self::Builtin { namespace, version } => Ok(Self::Builtin {
                namespace: allocations.boxed_str(namespace)?,
                version: allocations.boxed_str(version)?,
            }),
            Self::Package { package, version } => Ok(Self::Package {
                package: allocations.boxed_str(package)?,
                version: allocations.boxed_str(version)?,
            }),
        }
    }
}

impl From<&StableProjectRef> for TeloscPackageIdentity {
    fn from(pref: &StableProjectRef) -> Self {
        match pref {
            StableProjectRef::Builtin { namespace, version } => Self::Builtin {
                namespace: namespace.clone(),
                version: version.clone(),
            },
            StableProjectRef::Package { package, version, .. } => Self::Package {
                package: package.clone(),
                version: version.clone(),
            },
            StableProjectRef::SourceArtifact { logical_uri, .. } => Self::Package {
                package: logical_uri.clone(),
                version: "0.1.0".into(),
            },
            StableProjectRef::Session { .. } => Self::Builtin {
                namespace: "session".into(),
                version: "0.1.0".into(),
            },
        }
    }
}

/// Portable semantic compatibility structure comparing two images across processes.
#[derive(Clone, Debug, Eq, PartialEq, Hash, Deserialize, Serialize)]
pub struct CanonicalCompatibilityIdentity {
    pub package: TeloscPackageIdentity,
    pub source_digest: ArtifactDigest,
    pub interface_digest: ArtifactDigest,
    pub semantic_digest: ArtifactDigest,
    pub semantic_model_version: u32,
    pub stable_identity_schema: u32,
    pub module_semantic_abi: u32,
    pub semantic_artifact_abi: u32,
    pub query_fingerprint_abi: u32,
    pub native_surface_schema_version: u32,
    pub native_catalog_fingerprint: [u8; 16],
    pub type_metadata_schema_version: u32,
}

impl CanonicalCompatibilityIdentity {
    pub fn clone_with_allocation_budget(&self, allocations: &mut AllocationBudget) -> Result<Self, AllocationError> {
        Ok(Self {
            package: self.package.clone_with_allocation_budget(allocations)?,
            source_digest: self.source_digest,
            interface_digest: self.interface_digest,
            semantic_digest: self.semantic_digest,
            semantic_model_version: self.semantic_model_version,
            stable_identity_schema: self.stable_identity_schema,
            module_semantic_abi: self.module_semantic_abi,
            semantic_artifact_abi: self.semantic_artifact_abi,
            query_fingerprint_abi: self.query_fingerprint_abi,
            native_surface_schema_version: self.native_surface_schema_version,
            native_catalog_fingerprint: self.native_catalog_fingerprint,
            type_metadata_schema_version: self.type_metadata_schema_version,
        })
    }

    /// Validates cross-process compatibility between two environments.
    pub fn check_compatible(&self, other: &Self) -> Result<(), ValidationError> {
        if self.package != other.package {
            return Err(ValidationError::IncompatibleWorld(format!(
                "package mismatch: {:?} vs {:?}",
                self.package, other.package
            )));
        }
        if self.semantic_model_version != other.semantic_model_version {
            return Err(ValidationError::IncompatibleModelVersion {
                found: self.semantic_model_version,
                expected: other.semantic_model_version,
            });
        }
        if self.stable_identity_schema != other.stable_identity_schema {
            return Err(ValidationError::IncompatibleAbi {
                kind: "stable_identity_schema",
                found: self.stable_identity_schema,
                expected: other.stable_identity_schema,
            });
        }
        if self.module_semantic_abi != other.module_semantic_abi {
            return Err(ValidationError::IncompatibleAbi {
                kind: "module_semantic_abi",
                found: self.module_semantic_abi,
                expected: other.module_semantic_abi,
            });
        }
        if self.semantic_artifact_abi != other.semantic_artifact_abi {
            return Err(ValidationError::IncompatibleAbi {
                kind: "semantic_artifact_abi",
                found: self.semantic_artifact_abi,
                expected: other.semantic_artifact_abi,
            });
        }
        if self.query_fingerprint_abi != other.query_fingerprint_abi {
            return Err(ValidationError::IncompatibleAbi {
                kind: "query_fingerprint_abi",
                found: self.query_fingerprint_abi,
                expected: other.query_fingerprint_abi,
            });
        }
        if self.native_surface_schema_version != other.native_surface_schema_version {
            return Err(ValidationError::NativeSurfaceSchemaMismatch {
                found: self.native_surface_schema_version,
                expected: other.native_surface_schema_version,
            });
        }
        if self.native_catalog_fingerprint != other.native_catalog_fingerprint {
            return Err(ValidationError::NativeCatalogFingerprintMismatch {
                found: self.native_catalog_fingerprint,
                expected: other.native_catalog_fingerprint,
            });
        }
        if self.type_metadata_schema_version != other.type_metadata_schema_version {
            return Err(ValidationError::TypeMetadataSchemaMismatch {
                found: self.type_metadata_schema_version,
                expected: other.type_metadata_schema_version,
            });
        }
        if self.interface_digest != other.interface_digest {
            return Err(ValidationError::IncompatibleWorld(format!(
                "interface digest mismatch: {:?} vs {:?}",
                self.interface_digest, other.interface_digest
            )));
        }
        if self.semantic_digest != other.semantic_digest {
            return Err(ValidationError::IncompatibleWorld(format!(
                "semantic digest mismatch: {:?} vs {:?}",
                self.semantic_digest, other.semantic_digest
            )));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn native_catalog_fingerprint_participates_in_compatibility() {
        let identity = CanonicalCompatibilityIdentity {
            package: TeloscPackageIdentity::universe(),
            source_digest: ArtifactDigest::ZERO,
            interface_digest: ArtifactDigest::ZERO,
            semantic_digest: ArtifactDigest::ZERO,
            semantic_model_version: 1,
            stable_identity_schema: 1,
            module_semantic_abi: 1,
            semantic_artifact_abi: 1,
            query_fingerprint_abi: 1,
            native_surface_schema_version: 1,
            native_catalog_fingerprint: [1; 16],
            type_metadata_schema_version: 1,
        };
        assert!(identity.check_compatible(&identity).is_ok());

        let mut foreign_query_abi = identity.clone();
        foreign_query_abi.query_fingerprint_abi = 2;
        assert!(matches!(
            identity.check_compatible(&foreign_query_abi),
            Err(ValidationError::IncompatibleAbi {
                kind: "query_fingerprint_abi",
                ..
            })
        ));

        let mut foreign = identity.clone();
        foreign.native_catalog_fingerprint = [2; 16];
        assert!(matches!(
            identity.check_compatible(&foreign),
            Err(ValidationError::NativeCatalogFingerprintMismatch { .. })
        ));
    }
}
