//! Read-side semantic query types.

use super::facts::FileRevision;

/// Coherent published semantic generation.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SemanticGeneration(pub u64);

/// Revision plus generation used by request handlers to describe a snapshot.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SnapshotStamp {
    /// File revision.
    pub revision: FileRevision,
    /// Semantic generation containing that revision.
    pub generation: SemanticGeneration,
}
