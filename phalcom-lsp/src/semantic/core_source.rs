//! Core-source integration seam.
//!
//! Core source indexing is deliberately separate from native metadata. The
//! first slice records this seam without making the generated table disappear
//! before live core declarations and opaque-native contracts are available.

/// Describes an opaque native member until a canonical semantic contract is
/// supplied by the live-core/native slice.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NativeReturnKnowledge {
    /// Native implementation has no semantic return contract.
    Unknown,
    /// Native declaration supplies a known return shape.
    Declared,
}
