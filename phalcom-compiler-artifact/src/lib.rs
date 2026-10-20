//! `.telosc` persisted semantic compiler image artifact container and codec.

pub mod codec;
pub mod container;
pub mod digest;
pub mod error;
pub mod executable;
pub mod header;
pub mod identity;
pub mod limits;
pub mod metadata;
pub mod module_world;
pub mod payload;
pub mod query;
pub mod section;
pub mod semantic;

pub use codec::{BinaryReader, BinaryWriter};
pub use container::{TeloscContainerBuilder, TeloscContainerView};
pub use digest::{
    ArtifactDigest, ArtifactHasher, DOMAIN_ARTIFACT, DOMAIN_METADATA, DOMAIN_MODULE_WORLD, DOMAIN_QUERY_IMAGE, DOMAIN_SECTION, DOMAIN_SEMANTIC_WORLD,
    DOMAIN_SOURCE,
};
pub use error::{ArtifactError, CodecError, ContainerError, ValidationError};
pub use header::{TELOSC_CONTAINER_SCHEMA_V2, TELOSC_MAGIC, TELOSC_MIN_READER_SCHEMA_V2, TeloscCapabilities, TeloscHeaderV2};
pub use identity::{CanonicalCompatibilityIdentity, TeloscPackageIdentity};
pub use limits::ValidationLimits;
pub use payload::CompilerArtifactPayload;
pub use section::{SECTION_DIRECTORY_ENTRY_SIZE, SectionDirectoryEntryV1, SectionFlags, SectionKind};
