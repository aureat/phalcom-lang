//! `.telosc` persisted semantic compiler image artifact container and codec.


pub mod codec;
pub mod container;
pub mod digest;
pub mod error;
pub mod header;
pub mod identity;
pub mod limits;
pub mod metadata;
pub mod module_world;
pub mod query;
pub mod section;
pub mod semantic;

pub use codec::{BinaryReader, BinaryWriter};
pub use container::{TeloscContainerBuilder, TeloscContainerView};
pub use digest::{
    ArtifactDigest, ArtifactHasher, DOMAIN_ARTIFACT, DOMAIN_METADATA, DOMAIN_MODULE_WORLD,
    DOMAIN_QUERY_IMAGE, DOMAIN_SECTION, DOMAIN_SEMANTIC_WORLD, DOMAIN_SOURCE,
};
pub use error::{ArtifactError, CodecError, ContainerError, ValidationError};
pub use header::{
    TeloscCapabilities, TeloscHeaderV2, TELOSC_CONTAINER_SCHEMA_V2, TELOSC_MAGIC,
    TELOSC_MIN_READER_SCHEMA_V2,
};
pub use identity::{CanonicalCompatibilityIdentity, TeloscPackageIdentity};
pub use limits::ValidationLimits;
pub use section::{
    SectionDirectoryEntryV1, SectionFlags, SectionKind, SECTION_DIRECTORY_ENTRY_SIZE,
};
