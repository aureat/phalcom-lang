//! Portable cross-process package and compatibility identities.

use crate::digest::{ArtifactDigest, ArtifactHasher};
use crate::error::ValidationError;
use phalcom_type_meta::identity::StableProjectRef;
use serde::{Deserialize, Serialize};

/// Portable package identity serialized into `.telosc` artifacts.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
pub enum TeloscPackageIdentity {
    Builtin {
        namespace: Box<str>,
        version: Box<str>,
    },
    Package {
        package: Box<str>,
        version: Box<str>,
    },
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
    pub native_surface_schema_version: u32,
    pub type_metadata_schema_version: u32,
}

impl CanonicalCompatibilityIdentity {
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
            return Err(ValidationError::IncompatibleAbi { kind: "stable_identity_schema", found: self.stable_identity_schema, expected: other.stable_identity_schema });
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
        if self.native_surface_schema_version != other.native_surface_schema_version {
            return Err(ValidationError::NativeSurfaceSchemaMismatch {
                found: self.native_surface_schema_version,
                expected: other.native_surface_schema_version,
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
