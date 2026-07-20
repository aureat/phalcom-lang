//! A native, fixed-length, mutable octet buffer.
//!
//! Realizes [PDR-0011](../../../docs/decisions/0011-admit-bytes-native-octet-buffer.md)
//! ruling 1: `Bytes` is a dedicated [`crate::heap::Object::Bytes`] heap variant
//! (the [ADR-0020](../../../docs/adr/accepted/0020-kernel-list-native-array-protocol.md)
//! kernel pattern — native storage, `.ph` protocol above), **not** an
//! [`crate::heap::InstanceObject`]. The backing store is `Box<[u8]>`, not
//! `Vec<u8>`: length is fixed at construction (`bytes.md` law 3) because a
//! realloc would strand copies of secret contents in the arena beyond the
//! reach of `zeroize` (`bytes.md` §7).

/// A fixed-length, mutable native octet buffer
/// ([PDR-0011](../../../docs/decisions/0011-admit-bytes-native-octet-buffer.md) ruling 1).
///
/// Contents are mutable through [`Self::set`]/[`Self::as_mut_slice`]; the
/// length never changes after construction. The eleven floor primitives
/// (`phalcom-core/src/primitive/bytes.rs`) operate directly on this buffer;
/// the surfaced protocol (`at(_)`/`set(_,_)`/`each(_)`/`slice(_,_)`/…) is
/// `.ph` over them (`docs/spec/v0.2/core/bytes.md` §4).
#[derive(Debug, Clone, PartialEq)]
pub struct BytesObject {
    /// The octets. `Box<[u8]>` — never reallocates, which is what makes the
    /// `.ph` `zeroize` contract complete (`bytes.md` §7 / PDR-0011 ruling 7).
    data: Box<[u8]>,
}

impl BytesObject {
    /// Builds a zero-filled buffer of `len` octets (`bytes.md` law 4:
    /// no constructor exposes uninitialized memory).
    pub fn new_zeroed(len: usize) -> Self {
        Self { data: vec![0u8; len].into_boxed_slice() }
    }

    /// Builds a buffer by taking ownership of `data` — one move, no copy.
    pub fn from_vec(data: Vec<u8>) -> Self {
        Self { data: data.into_boxed_slice() }
    }

    /// Returns the octet count. Constant for the buffer's whole lifetime
    /// (`bytes.md` law 3).
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Returns `true` iff the buffer holds no octets.
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Returns the octet at `index`, or `None` out of bounds — total, never
    /// a panic (`bytes.md` law 1).
    pub fn get(&self, index: usize) -> Option<u8> {
        self.data.get(index).copied()
    }

    /// Writes `value` at `index`. Returns `false` (and writes nothing) out
    /// of bounds.
    pub fn set(&mut self, index: usize, value: u8) -> bool {
        match self.data.get_mut(index) {
            Some(slot) => {
                *slot = value;
                true
            }
            None => false,
        }
    }

    /// Borrows the octets.
    pub fn as_slice(&self) -> &[u8] {
        &self.data
    }

    /// Mutably borrows the octets. Mutation never changes the length —
    /// a slice has no way to grow.
    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        &mut self.data
    }
}
