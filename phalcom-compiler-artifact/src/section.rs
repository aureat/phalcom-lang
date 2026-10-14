//! Artifact section definitions, directory entries, and flags.

use crate::digest::ArtifactDigest;
use std::fmt;

/// Section identity enum supporting standard known sections and arbitrary future extensions.
#[derive(Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub enum SectionKind {
    ModuleWorld,
    SemanticWorld,
    QueryImage,
    Metadata,
    Presentation,
    SourceMaps,
    Executable,
    Unknown(u32),
}

impl SectionKind {
    pub const TAG_MODULE_WORLD: u32 = 1;
    pub const TAG_SEMANTIC_WORLD: u32 = 2;
    pub const TAG_QUERY_IMAGE: u32 = 3;
    pub const TAG_METADATA: u32 = 4;
    pub const TAG_PRESENTATION: u32 = 5;
    pub const TAG_SOURCE_MAPS: u32 = 6;
    pub const TAG_EXECUTABLE: u32 = 7;

    pub const fn tag(self) -> u32 {
        match self {
            Self::ModuleWorld => Self::TAG_MODULE_WORLD,
            Self::SemanticWorld => Self::TAG_SEMANTIC_WORLD,
            Self::QueryImage => Self::TAG_QUERY_IMAGE,
            Self::Metadata => Self::TAG_METADATA,
            Self::Presentation => Self::TAG_PRESENTATION,
            Self::SourceMaps => Self::TAG_SOURCE_MAPS,
            Self::Executable => Self::TAG_EXECUTABLE,
            Self::Unknown(tag) => tag,
        }
    }

    pub const fn from_tag(tag: u32) -> Self {
        match tag {
            Self::TAG_MODULE_WORLD => Self::ModuleWorld,
            Self::TAG_SEMANTIC_WORLD => Self::SemanticWorld,
            Self::TAG_QUERY_IMAGE => Self::QueryImage,
            Self::TAG_METADATA => Self::Metadata,
            Self::TAG_PRESENTATION => Self::Presentation,
            Self::TAG_SOURCE_MAPS => Self::SourceMaps,
            Self::TAG_EXECUTABLE => Self::Executable,
            other => Self::Unknown(other),
        }
    }

    pub const fn is_known(self) -> bool {
        !matches!(self, Self::Unknown(_))
    }

    pub const fn name(self) -> &'static str {
        match self {
            Self::ModuleWorld => "MODULE_WORLD",
            Self::SemanticWorld => "SEMANTIC_WORLD",
            Self::QueryImage => "QUERY_IMAGE",
            Self::Metadata => "METADATA",
            Self::Presentation => "PRESENTATION",
            Self::SourceMaps => "SOURCE_MAPS",
            Self::Executable => "EXECUTABLE",
            Self::Unknown(_) => "UNKNOWN",
        }
    }
}

impl fmt::Debug for SectionKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unknown(tag) => write!(f, "SectionKind::Unknown({})", tag),
            known => write!(f, "SectionKind::{}", known.name()),
        }
    }
}

impl fmt::Display for SectionKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

bitflags::bitflags! {
    /// Section control and storage flags.
    #[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash)]
    pub struct SectionFlags: u32 {
        /// Section is mandatory for reading/processing.
        const REQUIRED   = 1 << 0;
        /// Section payload is compressed.
        const COMPRESSED = 1 << 1;
    }
}

/// Fixed size in bytes of an encoded directory entry on wire.
pub const SECTION_DIRECTORY_ENTRY_SIZE: usize = 4 + 4 + 4 + 8 + 8 + 8 + 32;

/// A v1 directory entry describing one section in the `.telosc` container.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SectionDirectoryEntryV1 {
    pub kind: SectionKind,
    pub schema_version: u32,
    pub flags: SectionFlags,
    pub offset: u64,
    pub encoded_len: u64,
    pub decoded_len: u64,
    pub digest: ArtifactDigest,
}
