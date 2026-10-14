//! Error types for artifact container parsing, codec, and validation.

use crate::digest::ArtifactDigest;
use crate::section::SectionKind;
use thiserror::Error;

/// Top-level error for artifact processing.
#[derive(Debug, Error)]
pub enum ArtifactError {
    #[error("metadata envelope: {0}")]
    MetadataEnvelope(#[from] crate::metadata::MetadataSectionError),

    #[error("container error: {0}")]
    Container(#[from] ContainerError),

    #[error("codec error: {0}")]
    Codec(#[from] CodecError),

    #[error("limit exceeded: {0}")]
    LimitExceeded(String),

    #[error("validation error: {0}")]
    Validation(#[from] ValidationError),

    #[error("io error: {0}")]
    Io(String),
}

/// Errors related to container format, header, directory, and section layout.
#[derive(Debug, Error)]
pub enum ContainerError {
    #[error("invalid magic bytes: {0:?}")]
    BadMagic([u8; 8]),

    #[error("unsupported container schema version: found {found}, max supported {supported}")]
    UnsupportedContainerSchema { found: u32, supported: u32 },

    #[error("unsupported minimum reader schema: found {found}, current version {supported}")]
    UnsupportedReaderSchema { found: u32, supported: u32 },

    #[error("section count overflow: found {count}, limit {limit}")]
    SectionCountOverflow { count: usize, limit: usize },

    #[error("missing required section: {0}")]
    MissingRequiredSection(SectionKind),

    #[error("unsupported section schema for {section}: {schema}")]
    UnsupportedSectionSchema { section: SectionKind, schema: u32 },

    #[error("unsupported section flags: {0:#x}")]
    UnsupportedSectionFlags(u32),

    #[error("duplicate section: {0}")]
    DuplicateSection(SectionKind),

    #[error("offset overflow for section {section}: offset {offset} + len {len}")]
    OffsetOverflow {
        section: SectionKind,
        offset: u64,
        len: u64,
    },

    #[error("length overflow for section {section}: len {len}, total {total}")]
    LengthOverflow {
        section: SectionKind,
        len: u64,
        total: u64,
    },

    #[error("section {section} extends beyond file bounds: end {end}, file length {file_len}")]
    OutOfFileSection {
        section: SectionKind,
        end: u64,
        file_len: u64,
    },

    #[error("overlapping sections: {first} and {second}")]
    OverlappingSections {
        first: SectionKind,
        second: SectionKind,
    },

    #[error("unknown required section with tag: {tag}")]
    UnknownRequiredSection { tag: u32 },

    #[error("section {section} exceeds size budget: size {size}, budget {budget}")]
    SectionBudgetViolation {
        section: SectionKind,
        size: u64,
        budget: u64,
    },

    #[error("section {section} digest mismatch: expected {expected}, actual {actual}")]
    SectionDigestMismatch {
        section: SectionKind,
        expected: ArtifactDigest,
        actual: ArtifactDigest,
    },

    #[error("artifact digest mismatch: expected {expected}, actual {actual}")]
    ArtifactDigestMismatch {
        expected: ArtifactDigest,
        actual: ArtifactDigest,
    },
}

/// Errors during low-level binary reading and decoding.
#[derive(Debug, Error)]
pub enum CodecError {
    #[error("unexpected end of file/buffer")]
    UnexpectedEof,

    #[error("invalid utf-8 string")]
    InvalidUtf8,

    #[error("invalid boolean byte: {0}")]
    InvalidBool(u8),

    #[error("invalid variant tag for {type_name}: {tag}")]
    InvalidTag {
        type_name: &'static str,
        tag: u32,
    },

    #[error("count for {item} ({count}) exceeds validation limit ({limit})")]
    LimitExceeded {
        item: &'static str,
        count: usize,
        limit: usize,
    },

    #[error("corrupted binary payload: {0}")]
    CorruptedData(String),
}

/// Semantic and compatibility validation errors.
#[derive(Debug, Error)]
pub enum ValidationError {
    #[error("incompatible world: {0}")]
    IncompatibleWorld(String),

    #[error("incompatible semantic model version: found {found}, expected {expected}")]
    IncompatibleModelVersion { found: u32, expected: u32 },

    #[error("incompatible ABI version for {kind}: found {found}, expected {expected}")]
    IncompatibleAbi {
        kind: &'static str,
        found: u32,
        expected: u32,
    },

    #[error("native surface schema mismatch: found {found}, expected {expected}")]
    NativeSurfaceSchemaMismatch { found: u32, expected: u32 },

    #[error("type metadata schema mismatch: found {found}, expected {expected}")]
    TypeMetadataSchemaMismatch { found: u32, expected: u32 },

    #[error("referential integrity error: {0}")]
    ReferentialIntegrity(String),
}
