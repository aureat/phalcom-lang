//! `.telosc` container header specification and codec.

use crate::codec::{BinaryReader, BinaryWriter};
use crate::digest::ArtifactDigest;
use crate::error::{CodecError, ContainerError};
use crate::identity::TeloscPackageIdentity;
use crate::limits::ValidationLimits;
use phalcom_type_meta::allocation::{AllocationBudget, AllocationError};

pub const TELOSC_MAGIC: &[u8; 8] = b"TELOSC\x01\x00";
pub const TELOSC_CONTAINER_SCHEMA_V2: u32 = 2;
pub const TELOSC_MIN_READER_SCHEMA_V2: u32 = 2;

pub const MODULE_SEMANTIC_ABI_V2: u32 = 2;
pub const QUERY_FINGERPRINT_ABI_V2: u32 = 2;
pub const SEMANTIC_ARTIFACT_ABI_V2: u32 = 2;
pub const STABLE_IDENTITY_SCHEMA_V1: u32 = 1;

bitflags::bitflags! {
    /// Capabilities declared by the `.telosc` artifact.
    #[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash)]
    pub struct TeloscCapabilities: u32 {
        /// Fully materialized compiler image capable of zero-analysis compilation.
        const CANONICAL_COMPILER_IMAGE_V1 = 1 << 0;
        /// Contains canonical RuntimePublic metadata.
        const RUNTIME_PUBLIC_METADATA    = 1 << 1;
        /// Presentation data (docstrings, syntax presentation).
        const PRESENTATION               = 1 << 2;
        /// Source map tables.
        const SOURCE_MAPS                = 1 << 3;
        /// Reserved for future P8 executable bytecodes.
        const PORTABLE_EXECUTABLE        = 1 << 4;
    }
}

/// The v2 container header for `.telosc` artifacts.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TeloscHeaderV2 {
    pub container_schema: u32,
    pub minimum_reader_schema: u32,
    pub package_identity: TeloscPackageIdentity,

    pub semantic_model_version: u32,
    pub stable_identity_schema: u32,
    pub module_semantic_abi: u32,
    pub semantic_artifact_abi: u32,
    pub query_fingerprint_abi: u32,

    pub native_surface_schema_version: u32,
    /// Structural compatibility identity for the complete trusted native catalog.
    pub native_catalog_fingerprint: [u8; 16],
    pub type_metadata_schema_version: u32,

    pub capabilities: TeloscCapabilities,

    pub source_digest: ArtifactDigest,
    pub interface_digest: ArtifactDigest,
    pub semantic_digest: ArtifactDigest,
    pub query_digest: ArtifactDigest,
    pub metadata_digest: ArtifactDigest,
    pub artifact_digest: ArtifactDigest,

    pub section_count: u32,
}

impl TeloscHeaderV2 {
    pub fn clone_with_allocation_budget(&self, allocations: &mut AllocationBudget) -> Result<Self, AllocationError> {
        Ok(Self {
            container_schema: self.container_schema,
            minimum_reader_schema: self.minimum_reader_schema,
            package_identity: self.package_identity.clone_with_allocation_budget(allocations)?,
            semantic_model_version: self.semantic_model_version,
            stable_identity_schema: self.stable_identity_schema,
            module_semantic_abi: self.module_semantic_abi,
            semantic_artifact_abi: self.semantic_artifact_abi,
            query_fingerprint_abi: self.query_fingerprint_abi,
            native_surface_schema_version: self.native_surface_schema_version,
            native_catalog_fingerprint: self.native_catalog_fingerprint,
            type_metadata_schema_version: self.type_metadata_schema_version,
            capabilities: self.capabilities,
            source_digest: self.source_digest,
            interface_digest: self.interface_digest,
            semantic_digest: self.semantic_digest,
            query_digest: self.query_digest,
            metadata_digest: self.metadata_digest,
            artifact_digest: self.artifact_digest,
            section_count: self.section_count,
        })
    }

    /// Encodes the header into the writer, returning the byte offset of `artifact_digest`.
    pub fn encode(&self, writer: &mut BinaryWriter) -> usize {
        writer.write_fixed(TELOSC_MAGIC);
        writer.write_u32(self.container_schema);
        writer.write_u32(self.minimum_reader_schema);

        match &self.package_identity {
            TeloscPackageIdentity::Builtin { namespace, version } => {
                writer.write_u8(0);
                writer.write_str(namespace);
                writer.write_str(version);
            }
            TeloscPackageIdentity::Package { package, version } => {
                writer.write_u8(1);
                writer.write_str(package);
                writer.write_str(version);
            }
        }

        writer.write_u32(self.semantic_model_version);
        writer.write_u32(self.stable_identity_schema);
        writer.write_u32(self.module_semantic_abi);
        writer.write_u32(self.semantic_artifact_abi);
        writer.write_u32(self.query_fingerprint_abi);

        writer.write_u32(self.native_surface_schema_version);
        writer.write_fixed(&self.native_catalog_fingerprint);
        writer.write_u32(self.type_metadata_schema_version);

        writer.write_u32(self.capabilities.bits());

        writer.write_fixed(self.source_digest.as_bytes());
        writer.write_fixed(self.interface_digest.as_bytes());
        writer.write_fixed(self.semantic_digest.as_bytes());
        writer.write_fixed(self.query_digest.as_bytes());
        writer.write_fixed(self.metadata_digest.as_bytes());

        let artifact_digest_offset = writer.len();
        writer.write_fixed(self.artifact_digest.as_bytes());

        writer.write_u32(self.section_count);

        artifact_digest_offset
    }

    /// Decodes the header from reader, returning the header and the byte offset of `artifact_digest`.
    pub fn decode(reader: &mut BinaryReader<'_>, limits: &ValidationLimits) -> Result<(Self, usize), ContainerError> {
        let mut allocations = AllocationBudget::new(usize::MAX);
        Self::decode_with_allocation_budget(reader, limits, &mut allocations)
    }

    /// Decodes owned package identity strings through the caller's load budget.
    pub fn decode_with_allocation_budget(
        reader: &mut BinaryReader<'_>,
        limits: &ValidationLimits,
        allocations: &mut AllocationBudget,
    ) -> Result<(Self, usize), ContainerError> {
        let magic = reader.read_fixed::<8>().map_err(|_| ContainerError::BadMagic([0; 8]))?;
        if &magic != TELOSC_MAGIC {
            return Err(ContainerError::BadMagic(magic));
        }

        let container_schema = reader.read_u32().map_err(|_| ContainerError::UnsupportedContainerSchema {
            found: 0,
            supported: TELOSC_CONTAINER_SCHEMA_V2,
        })?;
        if container_schema != TELOSC_CONTAINER_SCHEMA_V2 {
            return Err(ContainerError::UnsupportedContainerSchema {
                found: container_schema,
                supported: TELOSC_CONTAINER_SCHEMA_V2,
            });
        }

        let minimum_reader_schema = reader.read_u32().map_err(|_| ContainerError::UnsupportedReaderSchema {
            found: 0,
            supported: TELOSC_CONTAINER_SCHEMA_V2,
        })?;
        if minimum_reader_schema > TELOSC_CONTAINER_SCHEMA_V2 {
            return Err(ContainerError::UnsupportedReaderSchema {
                found: minimum_reader_schema,
                supported: TELOSC_CONTAINER_SCHEMA_V2,
            });
        }

        let pkg_tag = reader
            .read_u8()
            .map_err(|_| CodecError::UnexpectedEof)
            .map_err(|_| ContainerError::BadMagic([0; 8]))?;

        let package_identity = match pkg_tag {
            0 => {
                let namespace = reader
                    .read_str(limits.max_string_bytes)
                    .map_err(|_| CodecError::CorruptedData("invalid package namespace".into()))
                    .map_err(|_| ContainerError::BadMagic([0; 8]))?;
                let version = reader
                    .read_str(limits.max_string_bytes)
                    .map_err(|_| CodecError::CorruptedData("invalid package version".into()))
                    .map_err(|_| ContainerError::BadMagic([0; 8]))?;
                TeloscPackageIdentity::Builtin {
                    namespace: allocations.boxed_str(namespace)?,
                    version: allocations.boxed_str(version)?,
                }
            }
            1 => {
                let package = reader
                    .read_str(limits.max_string_bytes)
                    .map_err(|_| CodecError::CorruptedData("invalid package name".into()))
                    .map_err(|_| ContainerError::BadMagic([0; 8]))?;
                let version = reader
                    .read_str(limits.max_string_bytes)
                    .map_err(|_| CodecError::CorruptedData("invalid package version".into()))
                    .map_err(|_| ContainerError::BadMagic([0; 8]))?;
                TeloscPackageIdentity::Package {
                    package: allocations.boxed_str(package)?,
                    version: allocations.boxed_str(version)?,
                }
            }
            other => {
                return Err(ContainerError::BadMagic([other; 8]));
            }
        };

        let semantic_model_version = reader
            .read_u32()
            .map_err(|_| CodecError::UnexpectedEof)
            .map_err(|_| ContainerError::BadMagic([0; 8]))?;
        let stable_identity_schema = reader
            .read_u32()
            .map_err(|_| CodecError::UnexpectedEof)
            .map_err(|_| ContainerError::BadMagic([0; 8]))?;
        let module_semantic_abi = reader
            .read_u32()
            .map_err(|_| CodecError::UnexpectedEof)
            .map_err(|_| ContainerError::BadMagic([0; 8]))?;
        let semantic_artifact_abi = reader
            .read_u32()
            .map_err(|_| CodecError::UnexpectedEof)
            .map_err(|_| ContainerError::BadMagic([0; 8]))?;
        let query_fingerprint_abi = reader
            .read_u32()
            .map_err(|_| CodecError::UnexpectedEof)
            .map_err(|_| ContainerError::BadMagic([0; 8]))?;

        let native_surface_schema_version = reader
            .read_u32()
            .map_err(|_| CodecError::UnexpectedEof)
            .map_err(|_| ContainerError::BadMagic([0; 8]))?;
        let native_catalog_fingerprint = reader
            .read_fixed::<16>()
            .map_err(|_| CodecError::UnexpectedEof)
            .map_err(|_| ContainerError::BadMagic([0; 8]))?;
        let type_metadata_schema_version = reader
            .read_u32()
            .map_err(|_| CodecError::UnexpectedEof)
            .map_err(|_| ContainerError::BadMagic([0; 8]))?;

        let cap_bits = reader
            .read_u32()
            .map_err(|_| CodecError::UnexpectedEof)
            .map_err(|_| ContainerError::BadMagic([0; 8]))?;
        let capabilities = TeloscCapabilities::from_bits_truncate(cap_bits);

        let source_digest = ArtifactDigest(
            reader
                .read_fixed::<32>()
                .map_err(|_| CodecError::UnexpectedEof)
                .map_err(|_| ContainerError::BadMagic([0; 8]))?,
        );
        let interface_digest = ArtifactDigest(
            reader
                .read_fixed::<32>()
                .map_err(|_| CodecError::UnexpectedEof)
                .map_err(|_| ContainerError::BadMagic([0; 8]))?,
        );
        let semantic_digest = ArtifactDigest(
            reader
                .read_fixed::<32>()
                .map_err(|_| CodecError::UnexpectedEof)
                .map_err(|_| ContainerError::BadMagic([0; 8]))?,
        );
        let query_digest = ArtifactDigest(
            reader
                .read_fixed::<32>()
                .map_err(|_| CodecError::UnexpectedEof)
                .map_err(|_| ContainerError::BadMagic([0; 8]))?,
        );
        let metadata_digest = ArtifactDigest(
            reader
                .read_fixed::<32>()
                .map_err(|_| CodecError::UnexpectedEof)
                .map_err(|_| ContainerError::BadMagic([0; 8]))?,
        );

        let artifact_digest_offset = reader.position();
        let artifact_digest = ArtifactDigest(
            reader
                .read_fixed::<32>()
                .map_err(|_| CodecError::UnexpectedEof)
                .map_err(|_| ContainerError::BadMagic([0; 8]))?,
        );

        let section_count = reader
            .read_u32()
            .map_err(|_| CodecError::UnexpectedEof)
            .map_err(|_| ContainerError::BadMagic([0; 8]))?;

        if section_count as usize > limits.max_section_count {
            return Err(ContainerError::SectionCountOverflow {
                count: section_count as usize,
                limit: limits.max_section_count,
            });
        }

        Ok((
            TeloscHeaderV2 {
                container_schema,
                minimum_reader_schema,
                package_identity,
                semantic_model_version,
                stable_identity_schema,
                module_semantic_abi,
                semantic_artifact_abi,
                query_fingerprint_abi,
                native_surface_schema_version,
                native_catalog_fingerprint,
                type_metadata_schema_version,
                capabilities,
                source_digest,
                interface_digest,
                semantic_digest,
                query_digest,
                metadata_digest,
                artifact_digest,
                section_count,
            },
            artifact_digest_offset,
        ))
    }
}
